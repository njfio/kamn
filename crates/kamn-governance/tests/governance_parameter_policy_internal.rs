use kamn_governance::{
    GovernanceParameterChangeDraft, GovernanceProposalDraft, GovernanceWorkflow,
    GovernanceWorkflowError,
};

fn draft(proposal_id: &str) -> GovernanceProposalDraft {
    GovernanceProposalDraft {
        proposal_id: proposal_id.to_owned(),
        title: format!("proposal {proposal_id}"),
        description: format!("description {proposal_id}"),
        proposer_did: "kamn:did:agent:validator-1".to_owned(),
        created_at_unix: 100,
        voting_deadline_unix: 200,
        quorum_threshold: 2,
        parameter_change: None,
    }
}

#[test]
fn parameter_change_rejects_unknown_key() {
    let mut workflow = GovernanceWorkflow::new();
    assert_eq!(
        workflow.submit_proposal(GovernanceProposalDraft {
            parameter_change: Some(GovernanceParameterChangeDraft {
                key: "unknown.parameter".to_owned(),
                proposed_value: 3,
                min_value: 1,
                max_value: 5,
                target_version: "1.0.0".to_owned(),
            }),
            ..draft("gov-parameter-policy")
        }),
        Err(GovernanceWorkflowError::UnknownParameterKey(
            "unknown.parameter".to_owned(),
        ))
    );
}
