use super::super::*;
use super::support::{backpressure_controller, evaluated_backpressure_action};

#[test]
fn unit_runtime_backpressure_policy_rejects_invalid_threshold_order() {
    assert_eq!(
        RuntimeBackpressurePolicy::new(900, 900, true),
        Err(RuntimeBackpressureError::InvalidThresholdOrder {
            slow_threshold_per_mille: 900,
            reject_threshold_per_mille: 900
        })
    );
}

#[test]
fn functional_runtime_backpressure_classifies_queue_saturation() {
    let policy = RuntimeBackpressurePolicy::new(700, 900, true).expect("valid policy");
    let controller = DeterministicBackpressureController::new(policy);
    let input =
        RuntimeBackpressureInput::new("kamn:did:agent:peer-a", 8, 10, PeerLifecycleState::Active)
            .expect("valid input");
    let decision = controller.evaluate(input).expect("evaluation should pass");
    assert_eq!(decision.action, RuntimeBackpressureAction::SlowProducer);
    assert_eq!(decision.queue_utilization_per_mille, 800);
}

#[test]
fn regression_runtime_backpressure_action_reason_matrix_remains_stable() {
    let controller = backpressure_controller();
    for (peer_did, depth, state, action, reason) in backpressure_reason_matrix() {
        assert_backpressure_reason_entry(&controller, peer_did, depth, state, action, reason);
    }
}

fn backpressure_reason_matrix() -> [(
    &'static str,
    usize,
    PeerLifecycleState,
    RuntimeBackpressureAction,
    &'static str,
); 4] {
    [
        (
            "kamn:did:agent:peer-matrix-accept",
            2,
            PeerLifecycleState::Active,
            RuntimeBackpressureAction::Accept,
            "runtime_backpressure_accept",
        ),
        (
            "kamn:did:agent:peer-matrix-slow",
            8,
            PeerLifecycleState::Active,
            RuntimeBackpressureAction::SlowProducer,
            "runtime_backpressure_slow_producer",
        ),
        (
            "kamn:did:agent:peer-matrix-reject",
            9,
            PeerLifecycleState::Active,
            RuntimeBackpressureAction::RejectNewEnqueue,
            "runtime_backpressure_reject_new_enqueue",
        ),
        (
            "kamn:did:agent:peer-matrix-purge",
            2,
            PeerLifecycleState::Disconnected,
            RuntimeBackpressureAction::PurgeStalePeerQueue,
            "runtime_backpressure_purge_stale_peer_queue",
        ),
    ]
}

fn assert_backpressure_reason_entry(
    controller: &DeterministicBackpressureController,
    peer_did: &str,
    depth: usize,
    state: PeerLifecycleState,
    expected_action: RuntimeBackpressureAction,
    expected_reason: &str,
) {
    let decision = evaluated_backpressure_action(controller, peer_did, depth, 10, state);
    assert_eq!(decision.action, expected_action);
    assert_eq!(decision.reason_code(), expected_reason);
}

#[test]
fn integration_runtime_backpressure_purges_stale_disconnected_peer_queue() {
    let controller = backpressure_controller();
    let decision = evaluated_backpressure_action(
        &controller,
        "kamn:did:agent:peer-b",
        3,
        10,
        PeerLifecycleState::Disconnected,
    );
    assert_eq!(
        decision.action,
        RuntimeBackpressureAction::PurgeStalePeerQueue
    );
}

#[test]
fn functional_runtime_queue_enforces_reject_action_on_enqueue() {
    let controller = backpressure_controller();
    let decision = evaluated_backpressure_action(
        &controller,
        "kamn:did:agent:peer-c",
        9,
        10,
        PeerLifecycleState::Active,
    );
    assert_eq!(decision.action, RuntimeBackpressureAction::RejectNewEnqueue);
}

#[test]
fn integration_runtime_queue_enforces_stale_peer_purge_action() {
    let controller = backpressure_controller();
    let decision = evaluated_backpressure_action(
        &controller,
        "kamn:did:agent:peer-d",
        5,
        10,
        PeerLifecycleState::Disconnected,
    );
    assert_eq!(
        decision.action,
        RuntimeBackpressureAction::PurgeStalePeerQueue
    );
}

#[test]
fn regression_runtime_queue_backpressure_reason_markers_remain_stable() {
    let controller = backpressure_controller();
    let decision = evaluated_backpressure_action(
        &controller,
        "kamn:did:agent:peer-markers",
        9,
        10,
        PeerLifecycleState::Active,
    );
    assert_eq!(
        decision.reason_code(),
        "runtime_backpressure_reject_new_enqueue"
    );
}
