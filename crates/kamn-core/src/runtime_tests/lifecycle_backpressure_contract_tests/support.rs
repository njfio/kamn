use super::super::*;
use crate::RuntimeBackpressureDecision;

pub(super) fn backpressure_controller() -> DeterministicBackpressureController {
    let policy = RuntimeBackpressurePolicy::new(700, 900, true).expect("valid policy");
    DeterministicBackpressureController::new(policy)
}

pub(super) fn backpressure_budget(env_key: &str, default_millis: u128) -> u128 {
    std::env::var(env_key)
        .ok()
        .and_then(|raw| raw.parse::<u128>().ok())
        .unwrap_or(default_millis)
}

pub(super) fn backpressure_input(
    peer_did: &str,
    depth: usize,
    capacity: usize,
    state: PeerLifecycleState,
) -> RuntimeBackpressureInput {
    RuntimeBackpressureInput::new(peer_did, depth, capacity, state).expect("valid input")
}

pub(super) fn evaluated_backpressure_action(
    controller: &DeterministicBackpressureController,
    peer_did: &str,
    depth: usize,
    capacity: usize,
    state: PeerLifecycleState,
) -> RuntimeBackpressureDecision {
    controller
        .evaluate(backpressure_input(peer_did, depth, capacity, state))
        .expect("evaluation should pass")
}
