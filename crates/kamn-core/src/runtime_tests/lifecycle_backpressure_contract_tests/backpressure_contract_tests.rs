use super::super::*;

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
    let policy = RuntimeBackpressurePolicy::new(700, 900, true).expect("valid policy");
    let controller = DeterministicBackpressureController::new(policy);
    let accept = controller
        .evaluate(
            RuntimeBackpressureInput::new(
                "kamn:did:agent:peer-matrix-accept",
                2,
                10,
                PeerLifecycleState::Active,
            )
            .expect("valid accept input"),
        )
        .expect("accept decision should evaluate");
    assert_eq!(accept.action, RuntimeBackpressureAction::Accept);
    assert_eq!(accept.reason_code(), "runtime_backpressure_accept");

    let slow = controller
        .evaluate(
            RuntimeBackpressureInput::new(
                "kamn:did:agent:peer-matrix-slow",
                8,
                10,
                PeerLifecycleState::Active,
            )
            .expect("valid slow input"),
        )
        .expect("slow decision should evaluate");
    assert_eq!(slow.action, RuntimeBackpressureAction::SlowProducer);
    assert_eq!(
        slow.reason_code(),
        "runtime_backpressure_slow_producer"
    );

    let reject = controller
        .evaluate(
            RuntimeBackpressureInput::new(
                "kamn:did:agent:peer-matrix-reject",
                9,
                10,
                PeerLifecycleState::Active,
            )
            .expect("valid reject input"),
        )
        .expect("reject decision should evaluate");
    assert_eq!(reject.action, RuntimeBackpressureAction::RejectNewEnqueue);
    assert_eq!(
        reject.reason_code(),
        "runtime_backpressure_reject_new_enqueue"
    );

    let purge = controller
        .evaluate(
            RuntimeBackpressureInput::new(
                "kamn:did:agent:peer-matrix-purge",
                2,
                10,
                PeerLifecycleState::Disconnected,
            )
            .expect("valid purge input"),
        )
        .expect("purge decision should evaluate");
    assert_eq!(purge.action, RuntimeBackpressureAction::PurgeStalePeerQueue);
    assert_eq!(
        purge.reason_code(),
        "runtime_backpressure_purge_stale_peer_queue"
    );
}

#[test]
fn integration_runtime_backpressure_purges_stale_disconnected_peer_queue() {
    let policy = RuntimeBackpressurePolicy::new(700, 900, true).expect("valid policy");
    let controller = DeterministicBackpressureController::new(policy);
    let input = RuntimeBackpressureInput::new(
        "kamn:did:agent:peer-b",
        3,
        10,
        PeerLifecycleState::Disconnected,
    )
    .expect("valid input");
    let decision = controller.evaluate(input).expect("evaluation should pass");
    assert_eq!(
        decision.action,
        RuntimeBackpressureAction::PurgeStalePeerQueue
    );
}

#[test]
fn functional_runtime_queue_enforces_reject_action_on_enqueue() {
    let policy = RuntimeBackpressurePolicy::new(700, 900, true).expect("valid policy");
    let controller = DeterministicBackpressureController::new(policy);
    let decision = controller
        .evaluate(
            RuntimeBackpressureInput::new(
                "kamn:did:agent:peer-c",
                9,
                10,
                PeerLifecycleState::Active,
            )
            .expect("valid input"),
        )
        .expect("evaluation should pass");
    assert_eq!(decision.action, RuntimeBackpressureAction::RejectNewEnqueue);
}

#[test]
fn integration_runtime_queue_enforces_stale_peer_purge_action() {
    let policy = RuntimeBackpressurePolicy::new(700, 900, true).expect("valid policy");
    let controller = DeterministicBackpressureController::new(policy);
    let decision = controller
        .evaluate(
            RuntimeBackpressureInput::new(
                "kamn:did:agent:peer-d",
                5,
                10,
                PeerLifecycleState::Disconnected,
            )
            .expect("valid input"),
        )
        .expect("evaluation should pass");
    assert_eq!(
        decision.action,
        RuntimeBackpressureAction::PurgeStalePeerQueue
    );
}

#[test]
fn regression_runtime_queue_backpressure_reason_markers_remain_stable() {
    let policy = RuntimeBackpressurePolicy::new(700, 900, true).expect("valid policy");
    let controller = DeterministicBackpressureController::new(policy);
    let decision = controller
        .evaluate(
            RuntimeBackpressureInput::new(
                "kamn:did:agent:peer-markers",
                9,
                10,
                PeerLifecycleState::Active,
            )
            .expect("valid input"),
        )
        .expect("evaluation should pass");
    assert_eq!(decision.reason_code(), "runtime_backpressure_reject_new_enqueue");
}
