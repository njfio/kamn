use kamn_core::{
    GovernanceProposalDraft, GovernanceProposalStatus, GovernanceVoteChoice, GovernanceWorkflow,
    ValidatorLifecycleError, ValidatorLifecycleManager, ValidatorTransitionProof,
};

fn proof(proposal_id: &str, approvers: &[&str]) -> ValidatorTransitionProof {
    ValidatorTransitionProof {
        proposal_id: proposal_id.to_owned(),
        approver_dids: approvers.iter().map(|value| (*value).to_owned()).collect(),
        proof_hash: format!("proof-hash-{proposal_id}"),
    }
}

#[test]
fn validator_lifecycle_rejects_invalid_initial_quorum_threshold() {
    assert_eq!(
        ValidatorLifecycleManager::new(
            vec![
                "kamn:did:agent:validator-1".to_owned(),
                "kamn:did:agent:validator-2".to_owned(),
            ],
            3,
        ),
        Err(ValidatorLifecycleError::InvalidQuorumThreshold {
            quorum_threshold: 3,
            validator_count: 2
        })
    );
}

#[test]
fn validator_lifecycle_functional_onboard_reconfigure_offboard_flow() {
    let mut manager = ValidatorLifecycleManager::new(
        vec![
            "kamn:did:agent:validator-1".to_owned(),
            "kamn:did:agent:validator-2".to_owned(),
        ],
        2,
    )
    .expect("manager should initialize");

    manager
        .onboard_validator(
            "kamn:did:agent:validator-3",
            &proof(
                "gov-proposal-onboard",
                &["kamn:did:agent:validator-1", "kamn:did:agent:validator-2"],
            ),
            1_716_400_100,
        )
        .expect("onboarding should succeed");
    let snapshot = manager.snapshot();
    assert_eq!(snapshot.validator_dids.len(), 3);

    manager
        .reconfigure_quorum(
            3,
            &proof(
                "gov-proposal-quorum",
                &[
                    "kamn:did:agent:validator-1",
                    "kamn:did:agent:validator-2",
                    "kamn:did:agent:validator-3",
                ],
            ),
            1_716_400_200,
        )
        .expect("quorum reconfiguration should succeed");
    assert_eq!(manager.snapshot().quorum_threshold, 3);
    manager
        .reconfigure_quorum(
            2,
            &proof(
                "gov-proposal-quorum-lower",
                &[
                    "kamn:did:agent:validator-1",
                    "kamn:did:agent:validator-2",
                    "kamn:did:agent:validator-3",
                ],
            ),
            1_716_400_250,
        )
        .expect("quorum lowering should succeed before offboarding");
    assert_eq!(manager.snapshot().quorum_threshold, 2);

    manager
        .offboard_validator(
            "kamn:did:agent:validator-3",
            &proof(
                "gov-proposal-offboard",
                &[
                    "kamn:did:agent:validator-1",
                    "kamn:did:agent:validator-2",
                    "kamn:did:agent:validator-3",
                ],
            ),
            1_716_400_300,
        )
        .expect("offboarding should succeed");
    let after = manager.snapshot();
    assert_eq!(after.validator_dids.len(), 2);
    assert_eq!(after.quorum_threshold, 2);
}

#[test]
fn validator_lifecycle_integration_requires_approved_governance_proposal_reference() {
    let mut governance = GovernanceWorkflow::new();
    governance
        .submit_proposal(GovernanceProposalDraft {
            proposal_id: "gov-proposal-validated".to_owned(),
            title: "Onboard validator-3".to_owned(),
            description: "Add one validator".to_owned(),
            proposer_did: "kamn:did:agent:validator-1".to_owned(),
            created_at_unix: 1_716_401_000,
            voting_deadline_unix: 1_716_401_600,
            quorum_threshold: 2,
        })
        .expect("proposal should submit");
    governance
        .cast_vote(
            "gov-proposal-validated",
            "kamn:did:agent:validator-2",
            GovernanceVoteChoice::Yes,
            1_716_401_100,
        )
        .expect("vote should be accepted");
    governance
        .cast_vote(
            "gov-proposal-validated",
            "kamn:did:agent:validator-3",
            GovernanceVoteChoice::Yes,
            1_716_401_200,
        )
        .expect("vote should be accepted");
    assert_eq!(
        governance
            .evaluate("gov-proposal-validated", 1_716_401_201)
            .expect("evaluation should succeed"),
        GovernanceProposalStatus::Approved
    );

    let mut manager = ValidatorLifecycleManager::new(
        vec![
            "kamn:did:agent:validator-1".to_owned(),
            "kamn:did:agent:validator-2".to_owned(),
            "kamn:did:agent:validator-3".to_owned(),
        ],
        2,
    )
    .expect("manager should initialize");
    manager
        .onboard_validator(
            "kamn:did:agent:validator-4",
            &proof(
                "gov-proposal-validated",
                &["kamn:did:agent:validator-1", "kamn:did:agent:validator-2"],
            ),
            1_716_401_300,
        )
        .expect("approved-governance-backed onboarding should pass");
}

#[test]
fn validator_lifecycle_regression_blocks_offboarding_that_breaks_quorum() {
    // Regression: #195
    let mut manager = ValidatorLifecycleManager::new(
        vec![
            "kamn:did:agent:validator-1".to_owned(),
            "kamn:did:agent:validator-2".to_owned(),
            "kamn:did:agent:validator-3".to_owned(),
        ],
        2,
    )
    .expect("manager should initialize");
    manager
        .offboard_validator(
            "kamn:did:agent:validator-3",
            &proof(
                "gov-proposal-offboard-1",
                &["kamn:did:agent:validator-1", "kamn:did:agent:validator-2"],
            ),
            1_716_402_100,
        )
        .expect("first offboarding should pass");

    assert_eq!(
        manager.offboard_validator(
            "kamn:did:agent:validator-2",
            &proof(
                "gov-proposal-offboard-2",
                &["kamn:did:agent:validator-1", "kamn:did:agent:validator-2",],
            ),
            1_716_402_200,
        ),
        Err(ValidatorLifecycleError::QuorumWouldExceedValidatorCount {
            quorum_threshold: 2,
            validator_count: 1
        })
    );
}
