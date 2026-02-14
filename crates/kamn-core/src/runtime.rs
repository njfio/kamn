use runtime_recovery_guard::{is_valid_kamn_did, is_valid_listener_did};

#[path = "runtime_backpressure.rs"]
mod runtime_backpressure;
#[path = "runtime_network_fault.rs"]
mod runtime_network_fault;
#[path = "runtime_peer_coordination.rs"]
mod runtime_peer_coordination;
#[path = "runtime_phase_coordination.rs"]
mod runtime_phase_coordination;
#[path = "runtime_recovery_guard.rs"]
mod runtime_recovery_guard;
#[path = "runtime_snapshot_store.rs"]
mod runtime_snapshot_store;
#[path = "runtime_state_divergence.rs"]
mod runtime_state_divergence;
#[path = "runtime_transport_coordination.rs"]
mod runtime_transport_coordination;

pub use runtime_backpressure::{
    DeterministicBackpressureController, RuntimeBackpressureAction, RuntimeBackpressureDecision,
    RuntimeBackpressureError, RuntimeBackpressureInput, RuntimeBackpressurePolicy,
};
pub use runtime_network_fault::{
    simulate_daemon_network_fault, DeterministicNetworkFaultSimulator, NetworkFaultSimulationError,
    NetworkFaultSimulationInput, NetworkFaultSimulationReport,
};
pub use runtime_peer_coordination::{
    build_runtime_wiring, AuthenticatedPeerFrame, AuthenticatedPeerFrameError, BoundedRuntimeQueue,
    DeterministicProposalPlanner, PeerFrameAuthenticator, PeerLifecycle, PeerLifecycleEvent,
    PeerLifecycleState, ProposalCandidate, ProposalPlan, ProposalPlannerError,
    RuntimeLifecycleError, RuntimeQueueError, RuntimeWiring,
};
pub use runtime_phase_coordination::{
    authorize_daemon_outbound_action, evaluate_daemon_listener_quorum,
    execute_processor_daemon_tick, ApproverAttestation, ApproverQuorumDecision,
    ApproverQuorumError, ApproverQuorumEvaluator, ApproverQuorumInput, ConstructLockError,
    ConstructLockGuard, ConstructLockLease, ListenerAttestation, ListenerQuorumDecision,
    ListenerQuorumError, ListenerQuorumEvaluator, ListenerQuorumInput,
};
pub use runtime_recovery_guard::{
    RecoveryGuardError, RecoveryRejoinGuard, RecoveryStatus, RejoinAttempt,
};
pub use runtime_snapshot_store::{
    FileRuntimeSnapshotStore, InMemoryRuntimeSnapshotStore, RuntimeSnapshot, RuntimeSnapshotStore,
    SnapshotRecoveryResult, SnapshotRestoreError, SnapshotRestoreGuard, SnapshotStoreError,
    SqliteRuntimeSnapshotStore,
};
pub use runtime_state_divergence::{
    evaluate_daemon_state_divergence, StateDivergenceError, StateDivergenceEvaluator,
    StateDivergenceEvidence, StateDivergenceReport, StateDivergenceSeverity, StateDivergenceStatus,
    StateDivergenceWatchInput,
};
pub use runtime_transport_coordination::{
    evaluate_daemon_watchdog_anomaly, WatchdogAnomalyError, WatchdogAnomalyEvaluator,
    WatchdogAnomalyKind, WatchdogAnomalyReport, WatchdogAnomalySeverity, WatchdogAnomalyWatchInput,
};

#[cfg(test)]
#[path = "runtime_tests.rs"]
mod tests;
