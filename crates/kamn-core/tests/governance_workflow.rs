use kamn_core::{
    ChannelStore, GovernanceParameterChangeDraft, GovernanceProposalDraft,
    GovernanceProposalStatus, GovernanceVoteChoice, GovernanceWorkflow, GovernanceWorkflowError,
};

#[test]
fn governance_workflow_rejects_zero_quorum_threshold() {
    let mut workflow = GovernanceWorkflow::new();
    assert_eq!(
        workflow.submit_proposal(GovernanceProposalDraft {
            proposal_id: "gov-proposal-1".to_owned(),
            title: "Rotate signer backend".to_owned(),
            description: "Switch secure signer endpoint".to_owned(),
            proposer_did: "kamn:did:agent:validator-1".to_owned(),
            created_at_unix: 1_716_300_000,
            voting_deadline_unix: 1_716_300_500,
            quorum_threshold: 0,
            parameter_change: None,
        }),
        Err(GovernanceWorkflowError::InvalidQuorum(0))
    );
}

#[test]
fn governance_workflow_functional_submit_vote_execute_flow() {
    let mut workflow = GovernanceWorkflow::new();
    workflow
        .submit_proposal(GovernanceProposalDraft {
            proposal_id: "gov-proposal-2".to_owned(),
            title: "Enable upgrade lane".to_owned(),
            description: "Allow protocol v0.2 rollout".to_owned(),
            proposer_did: "kamn:did:agent:validator-1".to_owned(),
            created_at_unix: 1_716_301_000,
            voting_deadline_unix: 1_716_302_000,
            quorum_threshold: 2,
            parameter_change: None,
        })
        .expect("proposal should submit");

    workflow
        .cast_vote(
            "gov-proposal-2",
            "kamn:did:agent:validator-2",
            GovernanceVoteChoice::Yes,
            1_716_301_100,
        )
        .expect("first vote should be accepted");
    assert_eq!(
        workflow
            .evaluate("gov-proposal-2", 1_716_301_101)
            .expect("evaluation should succeed"),
        GovernanceProposalStatus::Voting
    );
    workflow
        .cast_vote(
            "gov-proposal-2",
            "kamn:did:agent:validator-3",
            GovernanceVoteChoice::Yes,
            1_716_301_200,
        )
        .expect("second vote should satisfy quorum");
    assert_eq!(
        workflow
            .evaluate("gov-proposal-2", 1_716_301_201)
            .expect("evaluation should succeed"),
        GovernanceProposalStatus::Approved
    );

    let execution = workflow
        .execute(
            "gov-proposal-2",
            "kamn:did:agent:validator-1",
            1_716_301_300,
            "op-hash-1",
        )
        .expect("approved proposal should execute");
    assert_eq!(execution.proposal_id, "gov-proposal-2");
    assert_eq!(execution.executed_by, "kamn:did:agent:validator-1");

    let proposal = workflow
        .proposal("gov-proposal-2")
        .expect("proposal should exist");
    assert_eq!(proposal.status, GovernanceProposalStatus::Executed);
    assert_eq!(proposal.yes_votes, 2);
}

#[test]
fn governance_workflow_integration_with_governance_channel_members() {
    let mut channels = ChannelStore::default();
    channels
        .create_governance_channel(
            "governance-core",
            "kamn:did:agent:validator-1",
            "protocol-upgrades",
            vec![
                "kamn:did:agent:validator-1".to_owned(),
                "kamn:did:agent:validator-2".to_owned(),
                "kamn:did:agent:validator-3".to_owned(),
            ],
            vec!["kamn:did:agent:validator-1".to_owned()],
        )
        .expect("governance channel should be created");

    let mut workflow = GovernanceWorkflow::new();
    workflow
        .submit_proposal(GovernanceProposalDraft {
            proposal_id: "gov-proposal-3".to_owned(),
            title: "Update quorum".to_owned(),
            description: "Adjust listener quorum threshold".to_owned(),
            proposer_did: "kamn:did:agent:validator-1".to_owned(),
            created_at_unix: 1_716_302_000,
            voting_deadline_unix: 1_716_303_000,
            quorum_threshold: 2,
            parameter_change: None,
        })
        .expect("proposal should submit");
    workflow
        .cast_vote(
            "gov-proposal-3",
            "kamn:did:agent:validator-2",
            GovernanceVoteChoice::Yes,
            1_716_302_100,
        )
        .expect("vote should succeed");
    workflow
        .cast_vote(
            "gov-proposal-3",
            "kamn:did:agent:validator-3",
            GovernanceVoteChoice::Yes,
            1_716_302_200,
        )
        .expect("vote should succeed");

    assert_eq!(
        workflow
            .evaluate("gov-proposal-3", 1_716_302_201)
            .expect("proposal should evaluate"),
        GovernanceProposalStatus::Approved
    );
}

#[test]
fn governance_workflow_regression_rejects_late_votes_after_deadline() {
    // Regression: #197
    let mut workflow = GovernanceWorkflow::new();
    workflow
        .submit_proposal(GovernanceProposalDraft {
            proposal_id: "gov-proposal-4".to_owned(),
            title: "Rotate validator set".to_owned(),
            description: "Adjust validator roster for maintenance".to_owned(),
            proposer_did: "kamn:did:agent:validator-1".to_owned(),
            created_at_unix: 100,
            voting_deadline_unix: 120,
            quorum_threshold: 2,
            parameter_change: None,
        })
        .expect("proposal should submit");

    assert_eq!(
        workflow.cast_vote(
            "gov-proposal-4",
            "kamn:did:agent:validator-2",
            GovernanceVoteChoice::Yes,
            121,
        ),
        Err(GovernanceWorkflowError::ProposalClosed {
            proposal_id: "gov-proposal-4".to_owned(),
            status: GovernanceProposalStatus::Expired
        })
    );
}

#[test]
fn governance_workflow_regression_rejects_replayed_voter_approval_artifact() {
    // Regression: #911
    let mut workflow = GovernanceWorkflow::new();
    workflow
        .submit_proposal(GovernanceProposalDraft {
            proposal_id: "gov-proposal-4b".to_owned(),
            title: "Replay guard validation".to_owned(),
            description: "Ensure duplicate approval artifacts are rejected".to_owned(),
            proposer_did: "kamn:did:agent:validator-1".to_owned(),
            created_at_unix: 1_716_302_500,
            voting_deadline_unix: 1_716_303_500,
            quorum_threshold: 2,
            parameter_change: None,
        })
        .expect("proposal should submit");

    workflow
        .cast_vote(
            "gov-proposal-4b",
            "kamn:did:agent:validator-2",
            GovernanceVoteChoice::Yes,
            1_716_302_700,
        )
        .expect("first approval should succeed");

    assert_eq!(
        workflow.cast_vote(
            "gov-proposal-4b",
            "kamn:did:agent:validator-2",
            GovernanceVoteChoice::Yes,
            1_716_302_800,
        ),
        Err(GovernanceWorkflowError::DuplicateVote {
            proposal_id: "gov-proposal-4b".to_owned(),
            voter_did: "kamn:did:agent:validator-2".to_owned(),
        })
    );
}

#[test]
fn governance_workflow_rejects_parameter_change_with_invalid_target_version() {
    let mut workflow = GovernanceWorkflow::new();
    assert_eq!(
        workflow.submit_proposal(GovernanceProposalDraft {
            proposal_id: "gov-proposal-5".to_owned(),
            title: "Parameter update".to_owned(),
            description: "Update listener quorum".to_owned(),
            proposer_did: "kamn:did:agent:validator-1".to_owned(),
            created_at_unix: 1_716_303_000,
            voting_deadline_unix: 1_716_304_000,
            quorum_threshold: 2,
            parameter_change: Some(GovernanceParameterChangeDraft {
                key: "listener.quorum".to_owned(),
                proposed_value: 2,
                min_value: 1,
                max_value: 5,
                target_version: "vNext".to_owned(),
            }),
        }),
        Err(GovernanceWorkflowError::InvalidParameterTargetVersion(
            "vNext".to_owned()
        ))
    );
}

#[test]
fn governance_workflow_regression_rejects_parameter_change_out_of_bounds() {
    // Regression: #476
    let mut workflow = GovernanceWorkflow::new();
    assert_eq!(
        workflow.submit_proposal(GovernanceProposalDraft {
            proposal_id: "gov-proposal-6".to_owned(),
            title: "Parameter update".to_owned(),
            description: "Update listener quorum".to_owned(),
            proposer_did: "kamn:did:agent:validator-1".to_owned(),
            created_at_unix: 1_716_304_000,
            voting_deadline_unix: 1_716_305_000,
            quorum_threshold: 2,
            parameter_change: Some(GovernanceParameterChangeDraft {
                key: "listener.quorum".to_owned(),
                proposed_value: 0,
                min_value: 1,
                max_value: 5,
                target_version: "1.2.0".to_owned(),
            }),
        }),
        Err(GovernanceWorkflowError::ParameterOutOfBounds {
            key: "listener.quorum".to_owned(),
            proposed_value: 0,
            min_value: 1,
            max_value: 5,
        })
    );
}

#[test]
fn governance_workflow_rejects_unknown_parameter_key() {
    let mut workflow = GovernanceWorkflow::new();
    assert_eq!(
        workflow.submit_proposal(GovernanceProposalDraft {
            proposal_id: "gov-proposal-7".to_owned(),
            title: "Parameter update".to_owned(),
            description: "Attempt unknown parameter".to_owned(),
            proposer_did: "kamn:did:agent:validator-1".to_owned(),
            created_at_unix: 1_716_305_000,
            voting_deadline_unix: 1_716_306_000,
            quorum_threshold: 2,
            parameter_change: Some(GovernanceParameterChangeDraft {
                key: "runtime.unknown-toggle".to_owned(),
                proposed_value: 1,
                min_value: 0,
                max_value: 1,
                target_version: "1.2.0".to_owned(),
            }),
        }),
        Err(GovernanceWorkflowError::UnknownParameterKey(
            "runtime.unknown-toggle".to_owned()
        ))
    );
}

#[test]
fn governance_workflow_regression_rejects_parameter_change_for_unsupported_version() {
    // Regression: #476
    let mut workflow = GovernanceWorkflow::new();
    assert_eq!(
        workflow.submit_proposal(GovernanceProposalDraft {
            proposal_id: "gov-proposal-8".to_owned(),
            title: "Parameter update".to_owned(),
            description: "Update watchdog threshold before supported version".to_owned(),
            proposer_did: "kamn:did:agent:validator-1".to_owned(),
            created_at_unix: 1_716_306_000,
            voting_deadline_unix: 1_716_307_000,
            quorum_threshold: 2,
            parameter_change: Some(GovernanceParameterChangeDraft {
                key: "watchdog.delivery_ratio_bps".to_owned(),
                proposed_value: 9700,
                min_value: 9000,
                max_value: 9999,
                target_version: "1.0.0".to_owned(),
            }),
        }),
        Err(GovernanceWorkflowError::ParameterUnsupportedForVersion {
            key: "watchdog.delivery_ratio_bps".to_owned(),
            target_version: "1.0.0".to_owned(),
            min_supported_version: "1.1.0".to_owned(),
        })
    );
}

#[test]
fn governance_workflow_functional_executes_valid_parameter_change_proposal() {
    let mut workflow = GovernanceWorkflow::new();
    workflow
        .submit_proposal(GovernanceProposalDraft {
            proposal_id: "gov-proposal-9".to_owned(),
            title: "Parameter update".to_owned(),
            description: "Adjust listener quorum for runtime quorum safety".to_owned(),
            proposer_did: "kamn:did:agent:validator-1".to_owned(),
            created_at_unix: 1_716_307_000,
            voting_deadline_unix: 1_716_308_000,
            quorum_threshold: 2,
            parameter_change: Some(GovernanceParameterChangeDraft {
                key: "listener.quorum".to_owned(),
                proposed_value: 4,
                min_value: 2,
                max_value: 7,
                target_version: "1.2.0".to_owned(),
            }),
        })
        .expect("proposal should submit");
    workflow
        .cast_vote(
            "gov-proposal-9",
            "kamn:did:agent:validator-2",
            GovernanceVoteChoice::Yes,
            1_716_307_100,
        )
        .expect("vote should succeed");
    workflow
        .cast_vote(
            "gov-proposal-9",
            "kamn:did:agent:validator-3",
            GovernanceVoteChoice::Yes,
            1_716_307_200,
        )
        .expect("vote should succeed");
    workflow
        .execute(
            "gov-proposal-9",
            "kamn:did:agent:validator-1",
            1_716_307_300,
            "op-hash-9",
        )
        .expect("approved parameter proposal should execute");

    let proposal = workflow
        .proposal("gov-proposal-9")
        .expect("proposal should exist");
    assert_eq!(proposal.status, GovernanceProposalStatus::Executed);
    assert_eq!(
        proposal.parameter_change,
        Some(GovernanceParameterChangeDraft {
            key: "listener.quorum".to_owned(),
            proposed_value: 4,
            min_value: 2,
            max_value: 7,
            target_version: "1.2.0".to_owned(),
        })
    );
}
