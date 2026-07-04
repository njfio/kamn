use crate::support::{
    concurrency_replay_fixture, regression_accept_contenders, run_task_accept_race,
};

#[test]
fn task_accept_concurrency_has_single_winner_and_consistent_state() {
    let contenders = ["kamn:did:agent:worker-1", "kamn:did:agent:worker-2"];
    assert_accept_outcome("task-concurrency-accept", &contenders, contenders.len() - 1);
}

#[test]
fn unit_concurrency_replay_fixture_entries_are_valid() {
    for contenders in concurrency_replay_fixture() {
        assert!(contenders.len() >= 3);
        assert!(contenders
            .iter()
            .all(|contender| contender.starts_with("kamn:did:agent:")));
    }
}

#[test]
fn functional_task_accept_concurrency_replay_fixture_preserves_invariants() {
    for (index, contenders) in concurrency_replay_fixture().iter().enumerate() {
        let task_id = format!("task-concurrency-fixture-{index}");
        assert_accept_outcome(&task_id, contenders, contenders.len() - 1);
    }
}

#[test]
fn regression_concurrency_accept_race_never_allows_multiple_winners() {
    let contenders = regression_accept_contenders();
    for round in 0..24 {
        let task_id = format!("task-concurrency-regression-{round}");
        assert_accept_outcome(&task_id, &contenders, contenders.len() - 1);
    }
}

fn assert_accept_outcome(task_id: &str, contenders: &[&str], expected_rejections: usize) {
    let (success_count, unauthorized_count, winner) = run_task_accept_race(task_id, contenders);
    assert_eq!(success_count, 1);
    assert_eq!(unauthorized_count, expected_rejections);
    let winner = winner.expect("race should return a winning actor");
    assert!(contenders.iter().any(|contender| contender == &winner));
}
