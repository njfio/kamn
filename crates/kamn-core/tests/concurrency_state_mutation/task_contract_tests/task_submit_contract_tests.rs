use crate::support::run_task_submit_race;

#[test]
fn task_submit_concurrency_rejects_duplicate_task_id_deterministically() {
    let requesters = ["kamn:did:agent:requester-1", "kamn:did:agent:requester-2"];
    let (success_count, duplicate_count, winner) =
        run_task_submit_race("task-concurrency-submit", &requesters);
    assert_eq!(success_count, 1);
    assert_eq!(duplicate_count, 1);
    assert!(requesters.iter().any(|requester| requester == &winner));
}
