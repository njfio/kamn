use super::super::*;

#[test]
fn functional_planner_orders_candidates_deterministically() {
    let candidates = vec![
        ProposalCandidate::new("tx-3", "kamn:did:agent:bbb", 2, "state-1").expect("valid"),
        ProposalCandidate::new("tx-1", "kamn:did:agent:aaa", 1, "state-1").expect("valid"),
        ProposalCandidate::new("tx-2", "kamn:did:agent:bbb", 1, "state-1").expect("valid"),
    ];
    let planner = DeterministicProposalPlanner::new("state-1");
    let plan = planner.plan(candidates).expect("plan should build");
    assert_eq!(
        plan.ordered_candidate_ids(),
        vec!["tx-1".to_owned(), "tx-2".to_owned(), "tx-3".to_owned()]
    );
}

#[test]
fn integration_queue_drains_into_planner_without_order_loss() {
    let mut queue = BoundedRuntimeQueue::new(3).expect("queue should build");
    assert!(queue
        .enqueue(ProposalCandidate::new("tx-3", "kamn:did:agent:bbb", 2, "state-1").expect("valid"))
        .is_ok());
    assert!(queue
        .enqueue(ProposalCandidate::new("tx-1", "kamn:did:agent:aaa", 1, "state-1").expect("valid"))
        .is_ok());
    assert!(queue
        .enqueue(ProposalCandidate::new("tx-2", "kamn:did:agent:bbb", 1, "state-1").expect("valid"))
        .is_ok());
    let mut drained = Vec::new();
    while let Some(candidate) = queue.dequeue() {
        drained.push(candidate);
    }
    let planner = DeterministicProposalPlanner::new("state-1");
    let plan = planner.plan(drained).expect("plan should build");
    assert_eq!(
        plan.ordered_candidate_ids(),
        vec!["tx-1".to_owned(), "tx-2".to_owned(), "tx-3".to_owned()]
    );
}

#[test]
fn unit_rejects_empty_candidate_id() {
    let candidate = ProposalCandidate::new("", "kamn:did:agent:aaa", 1, "state-1");
    assert_eq!(candidate, Err(ProposalPlannerError::InvalidCandidateId));
}

#[test]
fn regression_duplicate_candidate_id_is_rejected() {
    let candidates = vec![
        ProposalCandidate::new("tx-1", "kamn:did:agent:aaa", 1, "state-1").expect("valid"),
        ProposalCandidate::new("tx-1", "kamn:did:agent:bbb", 2, "state-1").expect("valid"),
    ];
    let planner = DeterministicProposalPlanner::new("state-1");
    let error = planner
        .plan(candidates)
        .expect_err("duplicate candidate id must fail");
    assert_eq!(
        error,
        ProposalPlannerError::DuplicateCandidateId("tx-1".to_owned())
    );
}

#[test]
fn regression_stale_state_hash_is_rejected() {
    let candidates =
        vec![ProposalCandidate::new("tx-1", "kamn:did:agent:aaa", 1, "state-2").expect("valid")];
    let planner = DeterministicProposalPlanner::new("state-1");
    let error = planner
        .plan(candidates)
        .expect_err("candidate state mismatch must fail");
    assert_eq!(
        error,
        ProposalPlannerError::StaleStateHash {
            expected: "state-1".to_owned(),
            found: "state-2".to_owned()
        }
    );
}
