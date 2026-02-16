//! Deterministic peer discovery and gossip transport adapters for runtime integration.

use crate::config::NodeConfig;
use crate::runtime::{
    PeerLifecycle, PeerLifecycleEvent, PeerLifecycleState, RuntimeLifecycleError,
    RuntimeTransportProfile,
};
#[cfg(feature = "libp2p-live-transport")]
use libp2p::{gossipsub, identify};
use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::error::Error;
use std::fmt::{Display, Formatter};
use std::sync::{Arc, Mutex, OnceLock};

use validation::{
    validate_peer_id, validate_swarm_bootstrap_peer_address, validate_swarm_listen_address,
    validate_topic,
};

const LIBP2P_SWARM_BEHAVIOR_COMPONENTS: [&str; 6] =
    ["tcp", "noise", "yamux", "identify", "kad", "gossipsub"];
const LIBP2P_IDENTIFY_PROTOCOL_ID: &str = "/kamn/libp2p-live/1.0.0";
const LIBP2P_TOPIC_NAMESPACE: &str = "kamn/v1/";

#[cfg(feature = "libp2p-live-transport")]
#[derive(libp2p::swarm::NetworkBehaviour)]
struct Libp2pDeterministicRuntimeBehaviour {
    gossipsub: gossipsub::Behaviour,
    identify: identify::Behaviour,
}

mod adapter;
mod coordinator;
mod error;
#[allow(dead_code)]
mod native_runtime;
mod p2p_transport_live;
mod runtime_event;
#[allow(dead_code)]
mod swarm_stack;
mod validation;

pub use adapter::{
    InMemoryPeerLifecycleTransport, PeerDiscoveryRecord, PeerGossipFrame, PeerLifecycleTransport,
    UdpPeerLifecycleTransport,
};
pub use coordinator::PeerLifecycleTransportCoordinator;
pub use error::{Libp2pRuntimeAdapterOperation, P2pTransportError};
pub use p2p_transport_live::{
    build_libp2p_lifecycle_regression_corpus, build_p2p_swarm_deterministic_config,
    canonical_libp2p_identify_protocol_id, canonical_libp2p_topic_id,
    compose_kademlia_discovery_bootstrap, compose_libp2p_swarm_behavior_stack,
    resolve_libp2p_live_runtime_backend, run_libp2p_lifecycle_regression_case,
    run_libp2p_lifecycle_regression_corpus, KademliaBootstrapSeedSet,
    KademliaDiscoveryBootstrapPlan, Libp2pLivePeerLifecycleTransport, Libp2pLiveRuntimeBackend,
    LiveTransportFaultClass, LiveTransportReconnectDecision, LiveTransportReconnectPolicy,
    P2pSwarmBehaviorStack, P2pSwarmDeterministicConfig, P2pSwarmHarnessMode, P2pSwarmHarnessReport,
    P2pSwarmHarnessTask, PeerLifecycleRegressionCase, PeerLifecycleRegressionError,
    PeerLifecycleRegressionExpectedOutcome, PeerLifecycleRegressionOutcome,
};
pub use runtime_event::{Libp2pBehaviorFailureClass, Libp2pRuntimeEvent, Libp2pRuntimeEventKind};

const PEER_ADAPTER_REASON_TAXONOMY_VERSION: &str = "kamn.runtime.peer-adapter-reason-taxonomy.v1";
const PEER_ADAPTER_REASON_SOURCE_RECONNECT_POLICY: &str = "p2p_live_reconnect_policy";
const PEER_ADAPTER_REASON_SOURCE_ERROR_PROJECTION: &str = "p2p_peer_adapter_error_projection";

/// Deterministic reason classification classes used by peer adapter projections.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PeerAdapterReasonClass {
    /// Retry should continue and the class is timeout-derived.
    RetryTimeout,
    /// Retry should continue and the class is transient/non-timeout.
    RetryTransient,
    /// Retry budget was exhausted.
    RetryBudgetExhausted,
    /// Operation must fail closed.
    FailClosed,
}

/// Deterministic projected reason output for peer adapter policy lanes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PeerAdapterReasonProjection {
    reason_code: &'static str,
    reason_class: PeerAdapterReasonClass,
    source_marker: &'static str,
}

impl PeerAdapterReasonProjection {
    fn new(
        reason_code: &'static str,
        reason_class: PeerAdapterReasonClass,
        source_marker: &'static str,
    ) -> Self {
        Self {
            reason_code,
            reason_class,
            source_marker,
        }
    }

    /// Returns deterministic projected reason code.
    pub fn reason_code(&self) -> &'static str {
        self.reason_code
    }

    /// Returns deterministic projected reason class.
    pub fn reason_class(&self) -> PeerAdapterReasonClass {
        self.reason_class
    }

    /// Returns deterministic projection source marker.
    pub fn source_marker(&self) -> &'static str {
        self.source_marker
    }
}

/// Deterministic multi-process validation hook definition for peer adapter lanes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PeerAdapterMultiProcessValidationHook {
    hook_id: &'static str,
    command: &'static str,
    local_heavy_only: bool,
    reason_taxonomy_version: &'static str,
}

impl PeerAdapterMultiProcessValidationHook {
    fn new(hook_id: &'static str, command: &'static str, local_heavy_only: bool) -> Self {
        Self {
            hook_id,
            command,
            local_heavy_only,
            reason_taxonomy_version: PEER_ADAPTER_REASON_TAXONOMY_VERSION,
        }
    }

    /// Returns deterministic hook identifier.
    pub fn hook_id(&self) -> &'static str {
        self.hook_id
    }

    /// Returns deterministic hook command marker.
    pub fn command(&self) -> &'static str {
        self.command
    }

    /// Returns whether the hook requires local-heavy execution opt-in.
    pub fn local_heavy_only(&self) -> bool {
        self.local_heavy_only
    }

    /// Returns deterministic reason taxonomy marker for this hook.
    pub fn reason_taxonomy_version(&self) -> &'static str {
        self.reason_taxonomy_version
    }
}

fn classify_reconnect_reason_code(
    reason_code: &'static str,
    fallback: PeerAdapterReasonClass,
) -> PeerAdapterReasonClass {
    match reason_code {
        "p2p_live_reconnect_retry_dial_timeout" => PeerAdapterReasonClass::RetryTimeout,
        "p2p_live_reconnect_retry_discovery_unavailable"
        | "p2p_live_reconnect_retry_stream_churn" => PeerAdapterReasonClass::RetryTransient,
        "p2p_live_reconnect_retry_budget_exhausted" => PeerAdapterReasonClass::RetryBudgetExhausted,
        "p2p_live_reconnect_protocol_violation" => PeerAdapterReasonClass::FailClosed,
        _ => fallback,
    }
}

/// Returns deterministic peer adapter reason taxonomy marker.
pub fn peer_adapter_reason_taxonomy_version() -> &'static str {
    PEER_ADAPTER_REASON_TAXONOMY_VERSION
}

/// Projects deterministic reason output for a live transport reconnect decision.
pub fn project_live_transport_reconnect_reason(
    decision: &LiveTransportReconnectDecision,
) -> PeerAdapterReasonProjection {
    match decision {
        LiveTransportReconnectDecision::Retry { reason_code, .. } => {
            PeerAdapterReasonProjection::new(
                reason_code,
                classify_reconnect_reason_code(reason_code, PeerAdapterReasonClass::RetryTransient),
                PEER_ADAPTER_REASON_SOURCE_RECONNECT_POLICY,
            )
        }
        LiveTransportReconnectDecision::FailClosed { reason_code } => {
            PeerAdapterReasonProjection::new(
                reason_code,
                classify_reconnect_reason_code(reason_code, PeerAdapterReasonClass::FailClosed),
                PEER_ADAPTER_REASON_SOURCE_RECONNECT_POLICY,
            )
        }
    }
}

/// Projects deterministic reason output for peer adapter transport errors.
pub fn project_peer_adapter_error_reason(error: &P2pTransportError) -> PeerAdapterReasonProjection {
    let reason_code = error.reason_code();
    let reason_class =
        classify_reconnect_reason_code(reason_code, PeerAdapterReasonClass::FailClosed);
    PeerAdapterReasonProjection::new(
        reason_code,
        reason_class,
        PEER_ADAPTER_REASON_SOURCE_ERROR_PROJECTION,
    )
}

/// Returns deterministic multi-process validation hooks for peer adapter contract lanes.
pub fn deterministic_multi_process_peer_validation_hooks(
) -> Vec<PeerAdapterMultiProcessValidationHook> {
    vec![
        PeerAdapterMultiProcessValidationHook::new(
            "peer_adapter_process_isolated_validation",
            "bash scripts/runtime/validate_libp2p_convergence_process_isolated_live.sh --mode run --lane-profile smoke --ci-fast-gate PASS",
            false,
        ),
        PeerAdapterMultiProcessValidationHook::new(
            "peer_adapter_process_isolated_policy",
            "bash scripts/runtime/check_libp2p_convergence_process_isolated_live_policy.sh --report-file /tmp/libp2p-convergence-process-isolated-live-summary.json --expected-final-decision GO --ci-fast-gate PASS",
            false,
        ),
        PeerAdapterMultiProcessValidationHook::new(
            "peer_adapter_process_isolated_contract_lane",
            "bash scripts/runtime/validate_libp2p_convergence_process_isolated_live_contract_lane.sh --mode run --lane-profile deep --ci-fast-gate FAIL",
            true,
        ),
    ]
}
