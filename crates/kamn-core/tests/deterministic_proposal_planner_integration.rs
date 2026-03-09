use kamn_core::{
    DeterministicProposalPlanner, ProposalCandidate, ProposalPlannerError,
};

fn candidate(
    id: &str,
    sender_did: &str,
    nonce: u64,
    state_hash: &str,
) -> ProposalCandidate {
    ProposalCandidate::new(id, sender_did, nonce, state_hash)
        .expect("candidate should construct")
}

fn assert_invalid_candidate(
    id: &str,
    sender_did: &str,
    nonce: u64,
    state_hash: &str,
    expected: ProposalPlannerError,
) {
    assert_eq!(
        ProposalCandidate::new(id, sender_did, nonce, state_hash),
        Err(expected)
    );
}

#[test]
fn integration_deterministic_proposal_planner_valid_plan_returns_expected_order() {
    let planner = DeterministicProposalPlanner::new("state-hash-a");
    let plan = planner
        .plan(vec![
            candidate("cand-c", "kamn:did:agent:peer-b", 2, "state-hash-a"),
            candidate("cand-a", "kamn:did:agent:peer-b", 1, "state-hash-a"),
            candidate("cand-b", "kamn:did:agent:peer-a", 1, "state-hash-a"),
        ])
        .expect("planning should succeed");

    assert_eq!(
        plan.ordered_candidate_ids(),
        vec![
            "cand-b".to_owned(),
            "cand-a".to_owned(),
            "cand-c".to_owned(),
        ]
    );
    assert_eq!(plan.ordered_candidates()[0].sender_did(), "kamn:did:agent:peer-a");
    assert_eq!(plan.ordered_candidates()[0].nonce(), 1);
    assert_eq!(plan.ordered_candidates()[1].sender_did(), "kamn:did:agent:peer-b");
    assert_eq!(plan.ordered_candidates()[1].nonce(), 1);
    assert_eq!(plan.ordered_candidates()[2].nonce(), 2);
}

#[test]
fn integration_deterministic_proposal_planner_invalid_candidates_fail_closed() {
    assert_invalid_candidate(
        "",
        "kamn:did:agent:peer-a",
        1,
        "state-hash-a",
        ProposalPlannerError::InvalidCandidateId,
    );
    assert_invalid_candidate(
        "cand-a",
        "",
        1,
        "state-hash-a",
        ProposalPlannerError::InvalidSenderDid,
    );
    assert_invalid_candidate(
        "cand-a",
        "kamn:did:agent:peer-a",
        0,
        "state-hash-a",
        ProposalPlannerError::InvalidNonce,
    );
    assert_invalid_candidate(
        "cand-a",
        "kamn:did:agent:peer-a",
        1,
        "",
        ProposalPlannerError::InvalidStateHash,
    );
}

#[test]
fn integration_deterministic_proposal_planner_duplicate_ids_and_stale_state_fail_closed() {
    let planner = DeterministicProposalPlanner::new("state-hash-a");

    assert_eq!(
        planner.plan(vec![
            candidate("cand-a", "kamn:did:agent:peer-a", 1, "state-hash-a"),
            candidate("cand-a", "kamn:did:agent:peer-b", 2, "state-hash-a"),
        ]),
        Err(ProposalPlannerError::DuplicateCandidateId(
            "cand-a".to_owned(),
        ))
    );

    assert_eq!(
        planner.plan(vec![candidate(
            "cand-b",
            "kamn:did:agent:peer-a",
            1,
            "state-hash-b",
        )]),
        Err(ProposalPlannerError::StaleStateHash {
            expected: "state-hash-a".to_owned(),
            found: "state-hash-b".to_owned(),
        })
    );
}
