use crate::support::{
    deep_lane_accept_contenders, performance_accept_contenders, run_escrow_dispute_refund_race,
    run_task_accept_race,
};
use std::time::Instant;

#[test]
fn performance_concurrency_state_mutation_contract_lane_stays_within_budget() {
    let started = Instant::now();
    let contenders = performance_accept_contenders();
    for round in 0..48 {
        let task_id = format!("task-concurrency-performance-{round}");
        let (success_count, unauthorized_count, _) = run_task_accept_race(&task_id, &contenders);
        assert_eq!(success_count, 1);
        assert_eq!(unauthorized_count, contenders.len() - 1);
    }
    assert!(started.elapsed().as_millis() < 800);
}

#[test]
fn performance_escrow_dispute_refund_concurrency_lane_stays_within_budget() {
    let started = Instant::now();
    for _ in 0..64 {
        let (success_count, invalid_count, released, refunded, remaining) =
            run_escrow_dispute_refund_race(55);
        assert!(success_count >= 1);
        assert!(success_count <= 2);
        assert_eq!(success_count + invalid_count, 2);
        assert_eq!((released, refunded, remaining), (0, 55, 0));
    }
    assert!(started.elapsed().as_millis() < 600);
}

#[test]
#[ignore = "scheduled concurrency stress deep lane"]
fn performance_concurrency_state_mutation_deep_lane_stress() {
    let contenders = deep_lane_accept_contenders();
    for round in 0..512 {
        let task_id = format!("task-concurrency-deep-{round}");
        let (success_count, unauthorized_count, _) = run_task_accept_race(&task_id, &contenders);
        assert_eq!(success_count, 1);
        assert_eq!(unauthorized_count, contenders.len() - 1);
    }
}
