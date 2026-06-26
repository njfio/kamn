use super::{
    authorize_daemon_outbound_action, build_runtime_wiring, evaluate_daemon_listener_quorum,
    evaluate_daemon_state_divergence, evaluate_daemon_watchdog_anomaly,
    execute_processor_daemon_tick, ApproverAttestation, ApproverQuorumError,
    ApproverQuorumEvaluator, ApproverQuorumInput, AuthenticatedPeerFrame,
    AuthenticatedPeerFrameError, BoundedRuntimeQueue, ConstructLockError, ConstructLockGuard,
    DeterministicBackpressureController, DeterministicProposalPlanner, ListenerAttestation,
    ListenerQuorumError, ListenerQuorumEvaluator, ListenerQuorumInput, PeerFrameAuthenticator,
    PeerLifecycle, PeerLifecycleEvent, PeerLifecycleState, ProposalCandidate, ProposalPlannerError,
    RecoveryGuardError, RecoveryRejoinGuard, RecoveryStatus, RejoinAttempt,
    RuntimeBackpressureAction, RuntimeBackpressureError, RuntimeBackpressureInput,
    RuntimeBackpressurePolicy, RuntimeLifecycleError, RuntimeQueueError, StateDivergenceError,
    StateDivergenceEvaluator, StateDivergenceSeverity, StateDivergenceStatus,
    StateDivergenceWatchInput, WatchdogAnomalyError, WatchdogAnomalyEvaluator, WatchdogAnomalyKind,
    WatchdogAnomalySeverity, WatchdogAnomalyWatchInput,
};
use crate::config::{NodeConfig, NodeRole, SyncMode};
use crate::signature_profile::baseline_signature_for_fields;
use std::time::Instant;

#[path = "runtime_tests/lifecycle_backpressure_contract_tests.rs"]
mod lifecycle_backpressure_contract_tests;
#[path = "runtime_tests/peer_frame_contract_tests.rs"]
mod peer_frame_contract_tests;
#[path = "runtime_tests/planner_recovery_lock_contract_tests.rs"]
mod planner_recovery_lock_contract_tests;
#[path = "runtime_tests/quorum_watchdog_contract_tests.rs"]
mod quorum_watchdog_contract_tests;
#[path = "runtime_tests_network_fault.rs"]
mod runtime_tests_network_fault;
#[path = "runtime_tests_snapshot_store.rs"]
mod runtime_tests_snapshot_store;
#[path = "runtime_tests/runtime_wiring_contract_tests.rs"]
mod runtime_wiring_contract_tests;

fn sample_config(role: NodeRole) -> NodeConfig {
    NodeConfig {
        chain_id: "kamn-devnet".to_owned(),
        chain_version: "v0.1.0".to_owned(),
        role,
        storage_dir: "/tmp/kamn".to_owned(),
        enable_gossip: true,
        sync_mode: SyncMode::Fast,
    }
}
