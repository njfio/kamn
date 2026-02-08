use kamn_core::{
    AgentDrivenUpgradeWorkflow, AgentUpgradeAuditEventKind, AgentUpgradeProposalDraft,
    AgentUpgradeProposalState, AgentUpgradeWorkflowConfig, AgentUpgradeWorkflowError,
    GovernanceProposalStatus, GovernanceVoteChoice,
};

#[test]
fn agent_upgrade_workflow_rejects_unallowlisted_agent_proposer() {
    let mut workflow = AgentDrivenUpgradeWorkflow::new(AgentUpgradeWorkflowConfig {
        current_version: "v0.5.0".to_owned(),
        allowed_agent_proposers: vec!["kamn:did:agent:upgrade-bot".to_owned()],
        required_human_reviews: 1,
        required_validator_quorum: 2,
        min_activation_delay_secs: 60,
    })
    .expect("workflow should initialize");

    assert_eq!(
        workflow.submit_agent_proposal(AgentUpgradeProposalDraft {
            proposal_id: "pilot-upgrade-1".to_owned(),
            target_version: "v0.6.0".to_owned(),
            agent_did: "kamn:did:agent:rogue-bot".to_owned(),
            rationale: "unsafe source".to_owned(),
            created_at_unix: 1_716_610_000,
            voting_deadline_unix: 1_716_610_600,
        }),
        Err(AgentUpgradeWorkflowError::UnauthorizedAgentProposer(
            "kamn:did:agent:rogue-bot".to_owned()
        ))
    );
}

#[test]
fn agent_upgrade_workflow_functional_human_review_governance_activation_flow() {
    let mut workflow = AgentDrivenUpgradeWorkflow::new(AgentUpgradeWorkflowConfig {
        current_version: "v0.5.0".to_owned(),
        allowed_agent_proposers: vec!["kamn:did:agent:upgrade-bot".to_owned()],
        required_human_reviews: 2,
        required_validator_quorum: 2,
        min_activation_delay_secs: 60,
    })
    .expect("workflow should initialize");
    workflow
        .submit_agent_proposal(AgentUpgradeProposalDraft {
            proposal_id: "pilot-upgrade-2".to_owned(),
            target_version: "v0.6.0".to_owned(),
            agent_did: "kamn:did:agent:upgrade-bot".to_owned(),
            rationale: "batch safety checks passed".to_owned(),
            created_at_unix: 1_716_611_000,
            voting_deadline_unix: 1_716_611_600,
        })
        .expect("agent proposal should register");
    workflow
        .approve_human_review(
            "pilot-upgrade-2",
            "kamn:did:agent:validator-1",
            1_716_611_050,
        )
        .expect("first review should pass");
    workflow
        .approve_human_review(
            "pilot-upgrade-2",
            "kamn:did:agent:validator-2",
            1_716_611_060,
        )
        .expect("second review should pass");

    workflow
        .submit_to_governance("pilot-upgrade-2", 1_716_611_100)
        .expect("governance submission should pass");
    workflow
        .cast_validator_vote(
            "pilot-upgrade-2",
            "kamn:did:agent:validator-1",
            GovernanceVoteChoice::Yes,
            1_716_611_120,
        )
        .expect("yes vote should pass");
    workflow
        .cast_validator_vote(
            "pilot-upgrade-2",
            "kamn:did:agent:validator-2",
            GovernanceVoteChoice::Yes,
            1_716_611_130,
        )
        .expect("yes vote should pass");
    workflow
        .finalize_upgrade(
            "pilot-upgrade-2",
            "kamn:did:agent:validator-1",
            1_716_611_200,
            "op-hash-pilot-upgrade-2",
        )
        .expect("approved governance proposal should activate");

    let record = workflow
        .proposal("pilot-upgrade-2")
        .expect("proposal record should exist");
    assert_eq!(record.state, AgentUpgradeProposalState::Activated);
    assert_eq!(
        workflow.upgrade_audit_view().current_version,
        "v0.6.0".to_owned()
    );
}

#[test]
fn agent_upgrade_workflow_integration_records_governance_and_upgrade_audit_traces() {
    let mut workflow = AgentDrivenUpgradeWorkflow::new(AgentUpgradeWorkflowConfig {
        current_version: "v0.6.0".to_owned(),
        allowed_agent_proposers: vec!["kamn:did:agent:upgrade-bot".to_owned()],
        required_human_reviews: 1,
        required_validator_quorum: 2,
        min_activation_delay_secs: 60,
    })
    .expect("workflow should initialize");
    workflow
        .submit_agent_proposal(AgentUpgradeProposalDraft {
            proposal_id: "pilot-upgrade-3".to_owned(),
            target_version: "v0.7.0".to_owned(),
            agent_did: "kamn:did:agent:upgrade-bot".to_owned(),
            rationale: "performance baseline green".to_owned(),
            created_at_unix: 1_716_612_000,
            voting_deadline_unix: 1_716_612_500,
        })
        .expect("proposal should register");
    workflow
        .approve_human_review(
            "pilot-upgrade-3",
            "kamn:did:agent:validator-1",
            1_716_612_050,
        )
        .expect("review should pass");
    workflow
        .submit_to_governance("pilot-upgrade-3", 1_716_612_100)
        .expect("governance submission should pass");
    workflow
        .cast_validator_vote(
            "pilot-upgrade-3",
            "kamn:did:agent:validator-1",
            GovernanceVoteChoice::Yes,
            1_716_612_120,
        )
        .expect("yes vote should pass");
    workflow
        .cast_validator_vote(
            "pilot-upgrade-3",
            "kamn:did:agent:validator-2",
            GovernanceVoteChoice::Yes,
            1_716_612_130,
        )
        .expect("yes vote should pass");
    workflow
        .finalize_upgrade(
            "pilot-upgrade-3",
            "kamn:did:agent:validator-1",
            1_716_612_200,
            "op-hash-pilot-upgrade-3",
        )
        .expect("finalization should pass");

    let governance_record = workflow
        .governance_record("pilot-upgrade-3")
        .expect("governance record should exist");
    assert_eq!(governance_record.status, GovernanceProposalStatus::Executed);

    let events = workflow.agent_audit_log();
    assert!(
        events
            .iter()
            .any(|event| event.kind == AgentUpgradeAuditEventKind::GovernanceSubmitted),
        "governance submission event should be emitted"
    );
    assert!(
        events
            .iter()
            .any(|event| event.kind == AgentUpgradeAuditEventKind::UpgradeActivated),
        "upgrade activation event should be emitted"
    );
}

#[test]
fn agent_upgrade_workflow_regression_blocks_governance_submission_without_human_quorum() {
    // Regression: #235
    let mut workflow = AgentDrivenUpgradeWorkflow::new(AgentUpgradeWorkflowConfig {
        current_version: "v0.7.0".to_owned(),
        allowed_agent_proposers: vec!["kamn:did:agent:upgrade-bot".to_owned()],
        required_human_reviews: 2,
        required_validator_quorum: 2,
        min_activation_delay_secs: 60,
    })
    .expect("workflow should initialize");
    workflow
        .submit_agent_proposal(AgentUpgradeProposalDraft {
            proposal_id: "pilot-upgrade-4".to_owned(),
            target_version: "v0.8.0".to_owned(),
            agent_did: "kamn:did:agent:upgrade-bot".to_owned(),
            rationale: "insufficient review gate".to_owned(),
            created_at_unix: 1_716_613_000,
            voting_deadline_unix: 1_716_613_700,
        })
        .expect("proposal should register");
    workflow
        .approve_human_review(
            "pilot-upgrade-4",
            "kamn:did:agent:validator-1",
            1_716_613_010,
        )
        .expect("single review should pass");

    assert_eq!(
        workflow.submit_to_governance("pilot-upgrade-4", 1_716_613_020),
        Err(AgentUpgradeWorkflowError::InsufficientHumanReviews {
            required: 2,
            provided: 1,
        })
    );
}

#[test]
fn agent_upgrade_workflow_regression_rejects_early_activation_before_delay() {
    // Regression: #528
    let mut workflow = AgentDrivenUpgradeWorkflow::new(AgentUpgradeWorkflowConfig {
        current_version: "v0.8.0".to_owned(),
        allowed_agent_proposers: vec!["kamn:did:agent:upgrade-bot".to_owned()],
        required_human_reviews: 1,
        required_validator_quorum: 2,
        min_activation_delay_secs: 120,
    })
    .expect("workflow should initialize");
    workflow
        .submit_agent_proposal(AgentUpgradeProposalDraft {
            proposal_id: "pilot-upgrade-5".to_owned(),
            target_version: "v0.9.0".to_owned(),
            agent_did: "kamn:did:agent:upgrade-bot".to_owned(),
            rationale: "timelock regression".to_owned(),
            created_at_unix: 1_716_614_000,
            voting_deadline_unix: 1_716_614_700,
        })
        .expect("proposal should register");
    workflow
        .approve_human_review(
            "pilot-upgrade-5",
            "kamn:did:agent:validator-1",
            1_716_614_050,
        )
        .expect("review should pass");
    workflow
        .submit_to_governance("pilot-upgrade-5", 1_716_614_100)
        .expect("governance submission should pass");
    workflow
        .cast_validator_vote(
            "pilot-upgrade-5",
            "kamn:did:agent:validator-1",
            GovernanceVoteChoice::Yes,
            1_716_614_120,
        )
        .expect("yes vote should pass");
    workflow
        .cast_validator_vote(
            "pilot-upgrade-5",
            "kamn:did:agent:validator-2",
            GovernanceVoteChoice::Yes,
            1_716_614_130,
        )
        .expect("yes vote should pass");

    assert_eq!(
        workflow.finalize_upgrade(
            "pilot-upgrade-5",
            "kamn:did:agent:validator-1",
            1_716_614_200,
            "op-hash-pilot-upgrade-5-early",
        ),
        Err(AgentUpgradeWorkflowError::ActivationDelayNotElapsed {
            proposal_id: "pilot-upgrade-5".to_owned(),
            earliest_activation_unix: 1_716_614_250,
            attempted_activation_unix: 1_716_614_200,
        })
    );
}
