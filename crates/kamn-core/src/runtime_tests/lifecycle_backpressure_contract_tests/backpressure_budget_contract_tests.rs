use super::super::*;
use super::support::{backpressure_budget, backpressure_controller, backpressure_input};

#[test]
fn performance_runtime_queue_backpressure_enforcement_stays_within_ci_budget() {
    let budget_millis =
        backpressure_budget("KAMN_RUNTIME_QUEUE_BACKPRESSURE_ENFORCEMENT_BUDGET_MS", 500);
    let controller = backpressure_controller();
    let started = Instant::now();
    for offset in 0..1_024 {
        let state = if offset % 8 == 0 {
            PeerLifecycleState::Disconnected
        } else {
            PeerLifecycleState::Active
        };
        let input = backpressure_input(&format!("kamn:did:agent:peer-{offset}"), 8, 10, state);
        let _ = controller.evaluate(input).expect("evaluation should pass");
    }
    assert!(
        started.elapsed().as_millis() <= budget_millis,
        "runtime queue backpressure enforcement exceeded CI budget"
    );
}

#[test]
fn regression_runtime_backpressure_rejects_capacity_overflow_sample() {
    assert_eq!(
        RuntimeBackpressureInput::new(
            "kamn:did:agent:peer-overflow",
            11,
            10,
            PeerLifecycleState::Active,
        ),
        Err(RuntimeBackpressureError::QueueDepthExceedsCapacity {
            depth: 11,
            capacity: 10
        })
    );
}

#[test]
fn performance_runtime_backpressure_evaluation_stays_within_ci_budget() {
    let budget_millis = backpressure_budget("KAMN_RUNTIME_BACKPRESSURE_EVALUATION_BUDGET_MS", 250);
    let controller = backpressure_controller();
    let started = Instant::now();
    for offset in 0..1_024 {
        let input = backpressure_input(
            &format!("kamn:did:agent:peer-eval-{offset}"),
            offset % 10,
            10,
            PeerLifecycleState::Active,
        );
        let _ = controller.evaluate(input).expect("evaluation should pass");
    }
    assert!(
        started.elapsed().as_millis() <= budget_millis,
        "runtime backpressure evaluation exceeded CI budget"
    );
}
