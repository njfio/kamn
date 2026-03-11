use kamn_governance::{
    GovernanceParameterChangeDraft, GovernanceProposalDraft, GovernanceWorkflow,
    GovernanceWorkflowError,
};

const VALIDATOR_DID: &str = "kamn:did:agent:validator-1";

fn draft(proposal_id: &str) -> GovernanceProposalDraft {
    GovernanceProposalDraft {
        proposal_id: proposal_id.to_owned(),
        title: format!("proposal {proposal_id}"),
        description: format!("description {proposal_id}"),
        proposer_did: VALIDATOR_DID.to_owned(),
        created_at_unix: 100,
        voting_deadline_unix: 200,
        quorum_threshold: 2,
        parameter_change: None,
    }
}

fn parameter_change(
    key: &str,
    proposed_value: u64,
    min_value: u64,
    max_value: u64,
    target_version: &str,
) -> GovernanceParameterChangeDraft {
    GovernanceParameterChangeDraft {
        key: key.to_owned(),
        proposed_value,
        min_value,
        max_value,
        target_version: target_version.to_owned(),
    }
}

#[test]
fn parameter_change_rejects_unsupported_target_version_fail_closed() {
    let mut workflow = GovernanceWorkflow::new();
    assert_eq!(
        workflow.submit_proposal(GovernanceProposalDraft {
            parameter_change: Some(parameter_change(
                "watchdog.delivery_ratio_bps",
                9500,
                9000,
                9999,
                "1.0.0",
            )),
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
            parameter_change: Some(parameter_change("listener.quorum", 8, 1, 7, "1.0.0")),
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
