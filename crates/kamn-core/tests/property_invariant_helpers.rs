#![allow(dead_code)]

use kamn_core::{PeerLifecycleEvent, PeerLifecycleState, TaskState};
use proptest::test_runner::{
    Config as ProptestConfig, FileFailurePersistence, RngAlgorithm, RngSeed,
};

const PROPTEST_REGRESSION_ROOT: &str = "proptest-regressions";

/// Parses an optional deterministic seed override.
///
/// Accepted formats:
/// - decimal (`1234`)
/// - hexadecimal with `0x` prefix (`0x4d2`)
///
/// Invalid values fall back to `default_seed` to keep runner behavior deterministic.
pub fn parse_seed_override(raw: Option<&str>, default_seed: u64) -> u64 {
    let Some(raw) = raw else {
        return default_seed;
    };
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return default_seed;
    }
    let parsed = if let Some(hex) = trimmed
        .strip_prefix("0x")
        .or_else(|| trimmed.strip_prefix("0X"))
    {
        u64::from_str_radix(hex, 16).ok()
    } else {
        trimmed.parse::<u64>().ok()
    };
    parsed.unwrap_or(default_seed)
}

/// Resolves a deterministic seed from environment with a stable fallback.
pub fn resolve_seed_from_env(env_key: &str, default_seed: u64) -> u64 {
    let raw = std::env::var(env_key).ok();
    parse_seed_override(raw.as_deref(), default_seed)
}

/// Derives a lane-specific deterministic seed from one base seed.
pub fn derive_seed(base_seed: u64, lane_salt: u64) -> u64 {
    base_seed ^ lane_salt
}

/// Creates a deterministic proptest configuration with source-parallel persistence.
pub fn deterministic_proptest_config(
    cases: u32,
    seed: u64,
    source_file: &'static str,
) -> ProptestConfig {
    ProptestConfig {
        cases,
        failure_persistence: Some(Box::new(FileFailurePersistence::SourceParallel(
            PROPTEST_REGRESSION_ROOT,
        ))),
        source_file: Some(source_file),
        rng_algorithm: RngAlgorithm::ChaCha,
        rng_seed: RngSeed::Fixed(seed),
        ..ProptestConfig::default()
    }
}

/// Shared legality projection for task-state transition steps.
pub fn is_legal_task_state_step(from: TaskState, to: TaskState) -> bool {
    matches!(
        (from, to),
        (TaskState::Submitted, TaskState::Accepted)
            | (TaskState::Submitted, TaskState::Cancelled)
            | (TaskState::Accepted, TaskState::Delegated)
            | (TaskState::Accepted, TaskState::InProgress)
            | (TaskState::Accepted, TaskState::Cancelled)
            | (TaskState::Delegated, TaskState::InProgress)
            | (TaskState::Delegated, TaskState::Cancelled)
            | (TaskState::InProgress, TaskState::Blocked)
            | (TaskState::InProgress, TaskState::InputRequired)
            | (TaskState::InProgress, TaskState::Completed)
            | (TaskState::InProgress, TaskState::Failed)
            | (TaskState::InProgress, TaskState::Cancelled)
            | (TaskState::InputRequired, TaskState::InProgress)
            | (TaskState::InputRequired, TaskState::Failed)
            | (TaskState::InputRequired, TaskState::Cancelled)
            | (TaskState::Blocked, TaskState::InProgress)
            | (TaskState::Blocked, TaskState::Failed)
            | (TaskState::Blocked, TaskState::Cancelled)
    )
}

/// Shared legality projection for peer-lifecycle transitions.
pub fn expected_peer_next_state(
    from: PeerLifecycleState,
    event: PeerLifecycleEvent,
) -> Option<PeerLifecycleState> {
    match (from, event) {
        (PeerLifecycleState::Disconnected, PeerLifecycleEvent::StartConnect)
        | (PeerLifecycleState::Disconnected, PeerLifecycleEvent::Rejoin) => {
            Some(PeerLifecycleState::Connecting)
        }
        (PeerLifecycleState::Connecting, PeerLifecycleEvent::HandshakeSucceeded) => {
            Some(PeerLifecycleState::Active)
        }
        (PeerLifecycleState::Connecting, PeerLifecycleEvent::Disconnect)
        | (PeerLifecycleState::Active, PeerLifecycleEvent::Disconnect)
        | (PeerLifecycleState::Degraded, PeerLifecycleEvent::Disconnect) => {
            Some(PeerLifecycleState::Disconnected)
        }
        (PeerLifecycleState::Active, PeerLifecycleEvent::HeartbeatMissed) => {
            Some(PeerLifecycleState::Degraded)
        }
        (PeerLifecycleState::Degraded, PeerLifecycleEvent::HeartbeatRestored) => {
            Some(PeerLifecycleState::Active)
        }
        _ => None,
    }
}
