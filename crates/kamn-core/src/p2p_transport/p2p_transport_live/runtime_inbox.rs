use super::deterministic_config::P2pSwarmDeterministicConfig;
use super::*;
#[cfg(not(feature = "libp2p-live-transport"))]
use crate::runtime::{
    DeterministicBackpressureController, PeerLifecycleState, RuntimeBackpressureAction,
    RuntimeBackpressureInput, RuntimeBackpressurePolicy,
};
use std::sync::{Arc, Mutex, OnceLock};

#[cfg(not(feature = "libp2p-live-transport"))]
const LIVE_RUNTIME_INBOX_QUEUE_CAPACITY: usize = 128;
#[cfg(not(feature = "libp2p-live-transport"))]
const LIVE_RUNTIME_INBOX_SLOW_THRESHOLD_PER_MILLE: u16 = 700;
#[cfg(not(feature = "libp2p-live-transport"))]
const LIVE_RUNTIME_INBOX_REJECT_THRESHOLD_PER_MILLE: u16 = 900;
#[cfg(not(feature = "libp2p-live-transport"))]
const LIVE_RUNTIME_INBOX_PURGE_DISCONNECTED_WITH_PENDING_QUEUE: bool = true;

#[cfg(not(feature = "libp2p-live-transport"))]
fn build_live_runtime_inbox_backpressure_controller(
) -> Result<DeterministicBackpressureController, P2pTransportError> {
    let policy = RuntimeBackpressurePolicy::new(
        LIVE_RUNTIME_INBOX_SLOW_THRESHOLD_PER_MILLE,
        LIVE_RUNTIME_INBOX_REJECT_THRESHOLD_PER_MILLE,
        LIVE_RUNTIME_INBOX_PURGE_DISCONNECTED_WITH_PENDING_QUEUE,
    )?;
    Ok(DeterministicBackpressureController::new(policy))
}

#[cfg(not(feature = "libp2p-live-transport"))]
pub(crate) fn enqueue_live_runtime_inbox_frame(
    state: &mut Libp2pLiveDataPlaneState,
    recipient_peer_id: &str,
    lifecycle_state: PeerLifecycleState,
    frame: PeerGossipFrame,
) -> Result<(), P2pTransportError> {
    let queue = state
        .inbox_by_peer
        .entry(recipient_peer_id.to_owned())
        .or_default();
    let controller = build_live_runtime_inbox_backpressure_controller()?;
    let backpressure_peer_id = if recipient_peer_id.starts_with("kamn:did:") {
        recipient_peer_id.to_owned()
    } else {
        format!("kamn:did:peer:{}", recipient_peer_id.replace(':', "-"))
    };
    let input = RuntimeBackpressureInput::new(
        backpressure_peer_id.as_str(),
        queue.len(),
        LIVE_RUNTIME_INBOX_QUEUE_CAPACITY,
        lifecycle_state,
    )?;
    let decision = controller.evaluate(input)?;
    match decision.action {
        RuntimeBackpressureAction::Accept | RuntimeBackpressureAction::SlowProducer => {
            queue.push_back(frame);
            Ok(())
        }
        RuntimeBackpressureAction::RejectNewEnqueue => {
            Err(P2pTransportError::RuntimeBackpressureRejected {
                reason_code: decision.reason_code(),
                queue_utilization_per_mille: decision.queue_utilization_per_mille,
            })
        }
        RuntimeBackpressureAction::PurgeStalePeerQueue => {
            let purged_entries = queue.len();
            queue.clear();
            Err(P2pTransportError::RuntimeBackpressurePurgedStalePeerQueue {
                reason_code: decision.reason_code(),
                purged_entries,
            })
        }
    }
}

#[cfg(not(feature = "libp2p-live-transport"))]
fn runtime_backpressure_behavior_failure_class(
    error: &P2pTransportError,
) -> Option<Libp2pBehaviorFailureClass> {
    match error {
        P2pTransportError::RuntimeBackpressureRejected { .. } => {
            Some(Libp2pBehaviorFailureClass::RuntimeBackpressureRejectNewEnqueue)
        }
        P2pTransportError::RuntimeBackpressurePurgedStalePeerQueue { .. } => {
            Some(Libp2pBehaviorFailureClass::RuntimeBackpressurePurgeStalePeerQueue)
        }
        _ => None,
    }
}

#[cfg(not(feature = "libp2p-live-transport"))]
pub(crate) fn emit_backpressure_runtime_event(
    state: &mut Libp2pLiveDataPlaneState,
    peer_id: &str,
    topic: &str,
    error: &P2pTransportError,
) {
    let Some(class) = runtime_backpressure_behavior_failure_class(error) else {
        return;
    };
    if let Ok(event) = Libp2pRuntimeEvent::behavior_failure(class, Some(peer_id), Some(topic)) {
        state.runtime_events.push_back(event);
    }
}

#[derive(Debug, Default)]
pub(crate) struct Libp2pLiveDataPlaneState {
    pub(crate) peers_by_id: BTreeMap<String, PeerDiscoveryRecord>,
    pub(crate) inbox_by_peer: BTreeMap<String, VecDeque<PeerGossipFrame>>,
    pub(crate) runtime_events: VecDeque<Libp2pRuntimeEvent>,
}

#[cfg(not(feature = "libp2p-live-transport"))]
#[derive(Debug, Clone)]
pub(super) struct Libp2pLiveDataPlane {
    pub(super) state: Arc<Mutex<Libp2pLiveDataPlaneState>>,
}

pub(super) fn build_live_data_plane_network_id(config: &P2pSwarmDeterministicConfig) -> String {
    let bootstrap_segment = if config.bootstrap_peers().is_empty() {
        format!("listen={}", config.listen_address())
    } else {
        format!("bootstrap={}", config.bootstrap_peers().join(","))
    };
    let topic_segment = format!("topics={}", config.gossip_topics().join(","));
    format!("{bootstrap_segment}|{topic_segment}")
}

pub(super) fn resolve_live_data_plane_state(
    network_id: &str,
) -> Result<Arc<Mutex<Libp2pLiveDataPlaneState>>, P2pTransportError> {
    let mut registry = libp2p_live_data_plane_registry()
        .lock()
        .map_err(|_| P2pTransportError::StateUnavailable)?;
    Ok(registry
        .entry(network_id.to_owned())
        .or_insert_with(|| Arc::new(Mutex::new(Libp2pLiveDataPlaneState::default())))
        .clone())
}

fn libp2p_live_data_plane_registry(
) -> &'static Mutex<BTreeMap<String, Arc<Mutex<Libp2pLiveDataPlaneState>>>> {
    static REGISTRY: OnceLock<Mutex<BTreeMap<String, Arc<Mutex<Libp2pLiveDataPlaneState>>>>> =
        OnceLock::new();
    REGISTRY.get_or_init(|| Mutex::new(BTreeMap::new()))
}
