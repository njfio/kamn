use kamn_governance::{
    GovernanceProposalDraft, GovernanceProposalStatus, GovernanceVoteChoice, GovernanceWorkflow,
    GovernanceWorkflowError,
};

#[test]
fn submit_rejects_invalid_deadline() {
    let mut workflow = GovernanceWorkflow::new();
    assert_eq!(
        workflow.submit_proposal(GovernanceProposalDraft {
            proposal_id: "gov-deadline".to_owned(),
            title: "Invalid deadline".to_owned(),
            description: "Should fail".to_owned(),
            proposer_did: "kamn:did:agent:validator-1".to_owned(),
            created_at_unix: 100,
            voting_deadline_unix: 99,
            quorum_threshold: 1,
            parameter_change: None,
        }),
        Err(GovernanceWorkflowError::InvalidDeadline {
            created_at_unix: 100,
            voting_deadline_unix: 99,
        })
    );
}

#[test]
fn no_quorum_transitions_to_rejected() {
    let mut workflow = GovernanceWorkflow::new();
    workflow
        .submit_proposal(GovernanceProposalDraft {
            proposal_id: "gov-reject".to_owned(),
            title: "Reject path".to_owned(),
            description: "No votes should reject".to_owned(),
            proposer_did: "kamn:did:agent:validator-1".to_owned(),
            created_at_unix: 100,
            voting_deadline_unix: 200,
            quorum_threshold: 2,
            parameter_change: None,
        })
        .expect("proposal should submit");
    workflow
        .cast_vote(
            "gov-reject",
            "kamn:did:agent:validator-2",
            GovernanceVoteChoice::No,
            110,
        )
        .expect("first no vote should pass");
    workflow
        .cast_vote(
            "gov-reject",
            "kamn:did:agent:validator-3",
            GovernanceVoteChoice::No,
            111,
        )
        .expect("second no vote should pass");

    assert_eq!(
        workflow
            .evaluate("gov-reject", 112)
            .expect("evaluation should succeed"),
        GovernanceProposalStatus::Rejected
    );
}
