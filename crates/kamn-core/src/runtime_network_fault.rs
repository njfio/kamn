use super::runtime_backpressure::{
    DeterministicBackpressureController, RuntimeBackpressureAction, RuntimeBackpressureError,
    RuntimeBackpressurePolicy,
};
use super::runtime_peer_coordination::{
    BoundedRuntimeQueue, PeerLifecycle, PeerLifecycleEvent, PeerLifecycleState,
    RuntimeLifecycleError, RuntimeQueueError,
};
use super::runtime_recovery_guard::is_valid_kamn_did;
use super::runtime_transport_coordination::{
    WatchdogAnomalyError, WatchdogAnomalyEvaluator, WatchdogAnomalyKind, WatchdogAnomalySeverity,
    WatchdogAnomalyWatchInput,
};
use std::error::Error;
use std::fmt::{Display, Formatter};

#[derive(Debug, Clone, PartialEq, Eq)]
/// Network fault simulation input.
pub struct NetworkFaultSimulationInput {
    sample_id: String,
    peer_id: String,
    expected_deliveries: u32,
    delivered_deliveries: u32,
    active_peers: u32,
    healthy_peers: u32,
    sample_window_secs: u64,
    targeted_peer_count: u32,
    queue_capacity: usize,
    queued_events: usize,
}

impl NetworkFaultSimulationInput {
    #[allow(clippy::too_many_arguments)]
    /// Handles new.
    pub fn new(
        sample_id: &str,
        peer_id: &str,
        expected_deliveries: u32,
        delivered_deliveries: u32,
        active_peers: u32,
        healthy_peers: u32,
        sample_window_secs: u64,
        targeted_peer_count: u32,
        queue_capacity: usize,
        queued_events: usize,
    ) -> Result<Self, NetworkFaultSimulationError> {
        if sample_id.trim().is_empty() {
            return Err(NetworkFaultSimulationError::InvalidSampleId);
        }
        if peer_id.trim().is_empty() {
            return Err(NetworkFaultSimulationError::InvalidPeerId);
        }
        if queue_capacity == 0 {
            return Err(NetworkFaultSimulationError::InvalidQueueCapacity { capacity: 0 });
        }
        WatchdogAnomalyWatchInput::new(
            sample_id,
            expected_deliveries,
            delivered_deliveries,
            active_peers,
            healthy_peers,
            sample_window_secs,
            targeted_peer_count,
        )
        .map_err(NetworkFaultSimulationError::Watchdog)?;

        Ok(Self {
            sample_id: sample_id.to_owned(),
            peer_id: peer_id.to_owned(),
            expected_deliveries,
            delivered_deliveries,
            active_peers,
            healthy_peers,
            sample_window_secs,
            targeted_peer_count,
            queue_capacity,
            queued_events,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// Network fault simulation report.
pub struct NetworkFaultSimulationReport {
    /// Sample id.
    pub sample_id: String,
    /// Final lifecycle state.
    pub final_lifecycle_state: PeerLifecycleState,
    /// Queue capacity.
    pub queue_capacity: usize,
    /// Queued events.
    pub queued_events: usize,
    /// Queue overflow attempts.
    pub queue_overflow_attempts: usize,
    /// Backpressure action observed most recently during queue mutation.
    pub backpressure_last_action: RuntimeBackpressureAction,
    /// Backpressure reason marker observed most recently during queue mutation.
    pub backpressure_last_reason_code: &'static str,
    /// Number of queue enqueue attempts rejected by deterministic backpressure.
    pub backpressure_rejected_events: usize,
    /// Number of queued entries purged due to stale disconnected peer policy.
    pub backpressure_purged_events: usize,
    /// Number of queue enqueue attempts accepted in slow-producer mode.
    pub backpressure_slow_events: usize,
    /// Watchdog kind.
    pub watchdog_kind: WatchdogAnomalyKind,
    /// Watchdog severity.
    pub watchdog_severity: WatchdogAnomalySeverity,
    /// Watchdog delivery ratio per mille.
    pub watchdog_delivery_ratio_per_mille: u16,
    /// Watchdog liveness ratio per mille.
    pub watchdog_liveness_ratio_per_mille: u16,
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// Network fault simulation error.
pub enum NetworkFaultSimulationError {
    /// Invalid sample id.
    InvalidSampleId,
    /// Invalid peer id.
    InvalidPeerId,
    /// Invalid queue capacity.
    InvalidQueueCapacity {
        /// Queue capacity value.
        capacity: usize,
    },
    /// Lifecycle.
    Lifecycle(RuntimeLifecycleError),
    /// Backpressure.
    Backpressure(RuntimeBackpressureError),
    /// Watchdog.
    Watchdog(WatchdogAnomalyError),
}

impl Display for NetworkFaultSimulationError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidSampleId => {
                write!(f, "network fault simulation sample id cannot be empty")
            }
            Self::InvalidPeerId => write!(f, "network fault simulation peer id cannot be empty"),
            Self::InvalidQueueCapacity { capacity } => write!(
                f,
                "network fault simulation queue capacity must be positive, found {capacity}"
            ),
            Self::Lifecycle(error) => write!(f, "{error}"),
            Self::Backpressure(error) => write!(f, "{error}"),
            Self::Watchdog(error) => write!(f, "{error}"),
        }
    }
}

impl Error for NetworkFaultSimulationError {}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
/// Deterministic network fault simulator.
pub struct DeterministicNetworkFaultSimulator {
    anomaly_evaluator: WatchdogAnomalyEvaluator,
}

impl DeterministicNetworkFaultSimulator {
    /// Handles simulate.
    pub fn simulate(
        &self,
        input: NetworkFaultSimulationInput,
    ) -> Result<NetworkFaultSimulationReport, NetworkFaultSimulationError> {
        let mut lifecycle =
            PeerLifecycle::new(&input.peer_id).map_err(NetworkFaultSimulationError::Lifecycle)?;
        lifecycle
            .transition(PeerLifecycleEvent::StartConnect)
            .map_err(NetworkFaultSimulationError::Lifecycle)?;
        lifecycle
            .transition(PeerLifecycleEvent::HandshakeSucceeded)
            .map_err(NetworkFaultSimulationError::Lifecycle)?;
        if input.healthy_peers < input.active_peers {
            lifecycle
                .transition(PeerLifecycleEvent::HeartbeatMissed)
                .map_err(NetworkFaultSimulationError::Lifecycle)?;
        }
        if input.active_peers > 0 && input.healthy_peers == 0 {
            lifecycle
                .transition(PeerLifecycleEvent::Disconnect)
                .map_err(NetworkFaultSimulationError::Lifecycle)?;
        }

        let backpressure_policy = RuntimeBackpressurePolicy::new(700, 900, true)
            .map_err(NetworkFaultSimulationError::Backpressure)?;
        let backpressure_controller = DeterministicBackpressureController::new(backpressure_policy);
        let backpressure_peer_id = if is_valid_kamn_did(&input.peer_id) {
            input.peer_id.clone()
        } else {
            format!("kamn:did:agent:{}", input.peer_id)
        };

        let mut queue = BoundedRuntimeQueue::new(input.queue_capacity).map_err(|_| {
            NetworkFaultSimulationError::InvalidQueueCapacity {
                capacity: input.queue_capacity,
            }
        })?;
        let mut queue_overflow_attempts = 0usize;
        let mut backpressure_last_action = RuntimeBackpressureAction::Accept;
        let mut backpressure_last_reason_code = "runtime_backpressure_accept";
        let mut backpressure_rejected_events = 0usize;
        let mut backpressure_purged_events = 0usize;
        let mut backpressure_slow_events = 0usize;
        for event_index in 0..input.queued_events {
            match queue.enqueue_with_backpressure(
                format!("fault-event-{event_index}"),
                &backpressure_controller,
                &backpressure_peer_id,
                lifecycle.state(),
            ) {
                Ok(decision) => {
                    if decision.action == RuntimeBackpressureAction::SlowProducer {
                        backpressure_slow_events += 1;
                    }
                    backpressure_last_action = decision.action;
                    backpressure_last_reason_code = decision.reason_code();
                }
                Err(RuntimeQueueError::BackpressureRejected { reason_code, .. }) => {
                    queue_overflow_attempts += 1;
                    backpressure_rejected_events += 1;
                    backpressure_last_action = RuntimeBackpressureAction::RejectNewEnqueue;
                    backpressure_last_reason_code = reason_code;
                }
                Err(RuntimeQueueError::BackpressurePurgedStalePeerQueue {
                    reason_code,
                    purged_entries,
                }) => {
                    queue_overflow_attempts += 1;
                    backpressure_purged_events += purged_entries;
                    backpressure_last_action = RuntimeBackpressureAction::PurgeStalePeerQueue;
                    backpressure_last_reason_code = reason_code;
                }
                Err(RuntimeQueueError::BackpressureInput(error)) => {
                    return Err(NetworkFaultSimulationError::Backpressure(error));
                }
                Err(RuntimeQueueError::Overflow { .. }) => {
                    queue_overflow_attempts += 1;
                    backpressure_last_action = RuntimeBackpressureAction::RejectNewEnqueue;
                    backpressure_last_reason_code = "runtime_queue_overflow";
                }
                Err(RuntimeQueueError::InvalidCapacity { capacity }) => {
                    return Err(NetworkFaultSimulationError::InvalidQueueCapacity { capacity });
                }
            }
        }

        let watchdog_input = WatchdogAnomalyWatchInput::new(
            &input.sample_id,
            input.expected_deliveries,
            input.delivered_deliveries,
            input.active_peers,
            input.healthy_peers,
            input.sample_window_secs,
            input.targeted_peer_count,
        )
        .map_err(NetworkFaultSimulationError::Watchdog)?;
        let watchdog_report = self
            .anomaly_evaluator
            .evaluate(watchdog_input)
            .map_err(NetworkFaultSimulationError::Watchdog)?;

        Ok(NetworkFaultSimulationReport {
            sample_id: input.sample_id,
            final_lifecycle_state: lifecycle.state(),
            queue_capacity: input.queue_capacity,
            queued_events: input.queued_events,
            queue_overflow_attempts,
            backpressure_last_action,
            backpressure_last_reason_code,
            backpressure_rejected_events,
            backpressure_purged_events,
            backpressure_slow_events,
            watchdog_kind: watchdog_report.kind,
            watchdog_severity: watchdog_report.severity,
            watchdog_delivery_ratio_per_mille: watchdog_report.delivery_ratio_per_mille,
            watchdog_liveness_ratio_per_mille: watchdog_report.liveness_ratio_per_mille,
        })
    }
}

/// Handles simulate daemon network fault.
pub fn simulate_daemon_network_fault(
    simulator: &DeterministicNetworkFaultSimulator,
    input: NetworkFaultSimulationInput,
) -> Result<NetworkFaultSimulationReport, NetworkFaultSimulationError> {
    simulator.simulate(input)
}
