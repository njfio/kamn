use kamn_core::{
    GovernanceParameterChangeDraft, GovernanceProposalDraft, GovernanceProposalStatus,
    GovernanceVoteChoice, GovernanceWorkflow, UpgradeAuditEventKind, UpgradeOrchestrationError,
    UpgradeProposalState, VersionUpgradeOrchestrator,
};

#[test]
fn upgrade_orchestration_rejects_non_advancing_version_target() {
    let mut orchestrator =
        VersionUpgradeOrchestrator::new("v0.2.0").expect("orchestrator should initialize");
    assert_eq!(
        orchestrator.propose_upgrade(
            "gov-upgrade-1",
            "v0.1.9",
            "kamn:did:agent:validator-1",
            2,
            1_716_500_000,
        ),
        Err(UpgradeOrchestrationError::InvalidTargetVersionTransition {
            current_version: "v0.2.0".to_owned(),
            target_version: "v0.1.9".to_owned(),
        })
    );
}

#[test]
fn upgrade_orchestration_functional_propose_approve_activate_flow() {
    let mut orchestrator =
        VersionUpgradeOrchestrator::new("v0.1.0").expect("orchestrator should initialize");
    orchestrator
        .propose_upgrade(
            "gov-upgrade-2",
            "v0.2.0",
            "kamn:did:agent:validator-1",
            2,
            1_716_500_100,
        )
        .expect("proposal should register");
    orchestrator
        .approve_upgrade("gov-upgrade-2", "kamn:did:agent:validator-1", 1_716_500_110)
        .expect("approval should pass");
    orchestrator
        .approve_upgrade("gov-upgrade-2", "kamn:did:agent:validator-2", 1_716_500_120)
        .expect("approval should pass");
    orchestrator
        .mark_governance_status(
            "gov-upgrade-2",
            GovernanceProposalStatus::Approved,
            1_716_500_130,
        )
        .expect("governance status update should pass");
    orchestrator
        .activate_upgrade("gov-upgrade-2", "kamn:did:agent:validator-1", 1_716_500_140)
        .expect("activation should pass");

    let audit = orchestrator.audit_view();
    assert_eq!(audit.current_version, "v0.2.0");
    assert_eq!(
        audit.events.last().map(|event| event.kind),
        Some(UpgradeAuditEventKind::Activated)
    );
}

#[test]
fn upgrade_orchestration_integration_uses_governance_vote_outcome() {
    let mut governance = GovernanceWorkflow::new();
    governance
        .submit_proposal(GovernanceProposalDraft {
            proposal_id: "gov-upgrade-3".to_owned(),
            title: "Upgrade to v0.3.0".to_owned(),
            description: "protocol upgrade".to_owned(),
            proposer_did: "kamn:did:agent:validator-1".to_owned(),
            created_at_unix: 1_716_501_000,
            voting_deadline_unix: 1_716_501_600,
            quorum_threshold: 2,
            parameter_change: Some(GovernanceParameterChangeDraft {
                key: "listener.quorum".to_owned(),
                proposed_value: 3,
                min_value: 2,
                max_value: 7,
                target_version: "1.1.0".to_owned(),
            }),
        })
        .expect("proposal should submit");
    governance
        .cast_vote(
            "gov-upgrade-3",
            "kamn:did:agent:validator-2",
            GovernanceVoteChoice::Yes,
            1_716_501_100,
        )
        .expect("vote should pass");
    governance
        .cast_vote(
            "gov-upgrade-3",
            "kamn:did:agent:validator-3",
            GovernanceVoteChoice::Yes,
            1_716_501_200,
        )
        .expect("vote should pass");
    let governance_status = governance
        .evaluate("gov-upgrade-3", 1_716_501_201)
        .expect("evaluation should pass");
    assert_eq!(governance_status, GovernanceProposalStatus::Approved);

    let mut orchestrator =
        VersionUpgradeOrchestrator::new("v0.2.0").expect("orchestrator should initialize");
    orchestrator
        .propose_upgrade(
            "gov-upgrade-3",
            "v0.3.0",
            "kamn:did:agent:validator-1",
            2,
            1_716_501_300,
        )
        .expect("upgrade proposal should register");
    orchestrator
        .approve_upgrade("gov-upgrade-3", "kamn:did:agent:validator-1", 1_716_501_310)
        .expect("approval should pass");
    orchestrator
        .approve_upgrade("gov-upgrade-3", "kamn:did:agent:validator-2", 1_716_501_320)
        .expect("approval should pass");
    orchestrator
        .mark_governance_status("gov-upgrade-3", governance_status, 1_716_501_330)
        .expect("governance status should sync");

    orchestrator
        .activate_upgrade("gov-upgrade-3", "kamn:did:agent:validator-1", 1_716_501_340)
        .expect("activation should pass");
    assert_eq!(orchestrator.audit_view().current_version, "v0.3.0");
}

#[test]
fn upgrade_orchestration_regression_rejects_activation_without_quorum_approvals() {
    // Regression: #193
    let mut orchestrator =
        VersionUpgradeOrchestrator::new("v0.4.0").expect("orchestrator should initialize");
    orchestrator
        .propose_upgrade(
            "gov-upgrade-4",
            "v0.5.0",
            "kamn:did:agent:validator-1",
            2,
            1_716_502_100,
        )
        .expect("proposal should register");
    orchestrator
        .approve_upgrade("gov-upgrade-4", "kamn:did:agent:validator-1", 1_716_502_110)
        .expect("single approval should pass");
    orchestrator
        .mark_governance_status(
            "gov-upgrade-4",
            GovernanceProposalStatus::Approved,
            1_716_502_120,
        )
        .expect("governance approved status should set");

    assert_eq!(
        orchestrator.activate_upgrade("gov-upgrade-4", "kamn:did:agent:validator-1", 1_716_502_130),
        Err(UpgradeOrchestrationError::InsufficientApprovals {
            required: 2,
            provided: 1,
        })
    );
}

#[test]
fn upgrade_orchestration_functional_activate_then_rollback_restores_version_and_audits_event() {
    let mut orchestrator =
        VersionUpgradeOrchestrator::new("v0.6.0").expect("orchestrator should initialize");
    orchestrator
        .propose_upgrade(
            "gov-upgrade-5",
            "v0.7.0",
            "kamn:did:agent:validator-1",
            2,
            1_716_503_100,
        )
        .expect("proposal should register");
    orchestrator
        .approve_upgrade("gov-upgrade-5", "kamn:did:agent:validator-1", 1_716_503_110)
        .expect("approval should pass");
    orchestrator
        .approve_upgrade("gov-upgrade-5", "kamn:did:agent:validator-2", 1_716_503_120)
        .expect("approval should pass");
    orchestrator
        .mark_governance_status(
            "gov-upgrade-5",
            GovernanceProposalStatus::Approved,
            1_716_503_130,
        )
        .expect("governance approved status should set");
    orchestrator
        .activate_upgrade("gov-upgrade-5", "kamn:did:agent:validator-1", 1_716_503_140)
        .expect("activation should pass");
    orchestrator
        .rollback_upgrade(
            "gov-upgrade-5",
            "v0.6.0",
            "kamn:did:agent:validator-1",
            1_716_503_150,
            "post-upgrade verification failed",
        )
        .expect("rollback should succeed for activated proposal");

    assert_eq!(orchestrator.audit_view().current_version, "v0.6.0");
    let proposal = orchestrator
        .proposal("gov-upgrade-5")
        .expect("proposal should exist");
    assert_eq!(proposal.state, UpgradeProposalState::RolledBack);
    assert_eq!(
        orchestrator
            .audit_view()
            .events
            .last()
            .map(|event| event.kind),
        Some(UpgradeAuditEventKind::RolledBack)
    );
}

#[test]
fn upgrade_orchestration_regression_rejects_rollback_before_activation() {
    // Regression: #910
    let mut orchestrator =
        VersionUpgradeOrchestrator::new("v0.8.0").expect("orchestrator should initialize");
    orchestrator
        .propose_upgrade(
            "gov-upgrade-6",
            "v0.9.0",
            "kamn:did:agent:validator-1",
            2,
            1_716_504_100,
        )
        .expect("proposal should register");

    assert_eq!(
        orchestrator.rollback_upgrade(
            "gov-upgrade-6",
            "v0.8.0",
            "kamn:did:agent:validator-1",
            1_716_504_110,
            "activation gate was never satisfied",
        ),
        Err(UpgradeOrchestrationError::RollbackNotAllowed(
            "gov-upgrade-6".to_owned()
        ))
    );
}

#[test]
fn upgrade_orchestration_regression_rejects_empty_rollback_reason() {
    // Regression: #910
    let mut orchestrator =
        VersionUpgradeOrchestrator::new("v1.0.0").expect("orchestrator should initialize");
    orchestrator
        .propose_upgrade(
            "gov-upgrade-7",
            "v1.1.0",
            "kamn:did:agent:validator-1",
            1,
            1_716_505_100,
        )
        .expect("proposal should register");
    orchestrator
        .approve_upgrade("gov-upgrade-7", "kamn:did:agent:validator-1", 1_716_505_110)
        .expect("approval should pass");
    orchestrator
        .mark_governance_status(
            "gov-upgrade-7",
            GovernanceProposalStatus::Approved,
            1_716_505_120,
        )
        .expect("governance approved status should set");
    orchestrator
        .activate_upgrade("gov-upgrade-7", "kamn:did:agent:validator-1", 1_716_505_130)
        .expect("activation should pass");

    assert_eq!(
        orchestrator.rollback_upgrade(
            "gov-upgrade-7",
            "v1.0.0",
            "kamn:did:agent:validator-1",
            1_716_505_140,
            "",
        ),
        Err(UpgradeOrchestrationError::EmptyField("reason"))
    );
}
