use super::super::*;

#[test]
fn performance_runtime_queue_backpressure_enforcement_stays_within_ci_budget() {
    let budget_millis = std::env::var("KAMN_RUNTIME_QUEUE_BACKPRESSURE_ENFORCEMENT_BUDGET_MS")
        .ok()
        .and_then(|raw| raw.parse::<u128>().ok())
        .unwrap_or(500);
    let policy = RuntimeBackpressurePolicy::new(700, 900, true).expect("valid policy");
    let controller = DeterministicBackpressureController::new(policy);
    let started = Instant::now();
    for offset in 0..1_024 {
        let state = if offset % 8 == 0 {
            PeerLifecycleState::Disconnected
        } else {
            PeerLifecycleState::Active
        };
        let input = RuntimeBackpressureInput::new(
            &format!("kamn:did:agent:peer-{offset}"),
            8,
            10,
            state,
        )
        .expect("valid input");
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
    let budget_millis = std::env::var("KAMN_RUNTIME_BACKPRESSURE_EVALUATION_BUDGET_MS")
        .ok()
        .and_then(|raw| raw.parse::<u128>().ok())
        .unwrap_or(250);
    let policy = RuntimeBackpressurePolicy::new(700, 900, true).expect("valid policy");
    let controller = DeterministicBackpressureController::new(policy);
    let started = Instant::now();
    for offset in 0..1_024 {
        let input = RuntimeBackpressureInput::new(
            &format!("kamn:did:agent:peer-eval-{offset}"),
            offset % 10,
            10,
            PeerLifecycleState::Active,
        )
        .expect("valid input");
        let _ = controller.evaluate(input).expect("evaluation should pass");
    }
    assert!(
        started.elapsed().as_millis() <= budget_millis,
        "runtime backpressure evaluation exceeded CI budget"
    );
}
