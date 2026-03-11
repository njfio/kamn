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
fn parameter_change_rejects_unsupported_target_version_fail_closed() {
    let mut workflow = GovernanceWorkflow::new();
    assert_eq!(
        workflow.submit_proposal(GovernanceProposalDraft {
            parameter_change: Some(GovernanceParameterChangeDraft {
                key: "watchdog.delivery_ratio_bps".to_owned(),
                proposed_value: 9500,
                min_value: 9000,
                max_value: 9999,
                target_version: "1.0.0".to_owned(),
            }),
            ..draft("gov-unsupported-version")
        }),
        Err(GovernanceWorkflowError::ParameterUnsupportedForVersion {
            key: "watchdog.delivery_ratio_bps".to_owned(),
            target_version: "1.0.0".to_owned(),
            min_supported_version: "1.1.0".to_owned(),
        })
    );
}

#[test]
fn parameter_change_rejects_value_outside_declared_bounds_fail_closed() {
    let mut workflow = GovernanceWorkflow::new();
    assert_eq!(
        workflow.submit_proposal(GovernanceProposalDraft {
            parameter_change: Some(GovernanceParameterChangeDraft {
                key: "listener.quorum".to_owned(),
                proposed_value: 8,
                min_value: 1,
                max_value: 7,
                target_version: "1.0.0".to_owned(),
            }),
            ..draft("gov-out-of-bounds")
        }),
        Err(GovernanceWorkflowError::ParameterOutOfBounds {
            key: "listener.quorum".to_owned(),
            proposed_value: 8,
            min_value: 1,
            max_value: 7,
        })
    );
}
