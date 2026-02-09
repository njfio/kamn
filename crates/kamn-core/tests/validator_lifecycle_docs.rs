const DOC: &str =
    include_str!("../../../docs/foundation/validator-lifecycle-quorum-reconfiguration.md");

#[test]
fn doc_contains_validator_lifecycle_scope_and_contracts() {
    assert!(DOC.contains("## Scope Delivered"));
    assert!(DOC.contains("ValidatorLifecycleManager"));
    assert!(DOC.contains("ValidatorTransitionProof"));
    assert!(DOC.contains("ValidatorLifecycleError"));
}

#[test]
fn doc_contains_transition_proof_and_quorum_safety_rules() {
    assert!(DOC.contains("## Transition Proof and Validation Rules"));
    assert!(DOC.contains("## Quorum Safety and Rollback Rules"));
    assert!(DOC.contains("Offboarding is blocked when resulting validator count would fall below current quorum threshold."));
    assert!(DOC.contains("Transition proof fingerprint (`proposal_id` + `proof_hash`) is one-time-use and replay attempts are rejected."));
    assert!(DOC.contains("Onboarding proof approver sets cannot include the candidate validator DID (self-approval rejection)."));
}

#[test]
fn doc_contains_fast_and_cost_effective_validation_lane() {
    assert!(DOC.contains("## Fast and Cost-Effective Validation"));
    assert!(DOC.contains("cargo test -p kamn-core --test validator_lifecycle"));
    assert!(DOC.contains("cargo clippy -p kamn-core -- -D warnings"));
}

#[test]
fn regression_requires_quorum_breaking_offboard_block_rule() {
    // Regression: #195
    assert!(DOC.contains("Offboarding is blocked when resulting validator count would fall below current quorum threshold."));
}

#[test]
fn regression_requires_replay_and_self_approval_rejection_rules() {
    // Regression: #523
    assert!(DOC.contains("transition proof replay is rejected (`Regression: #523`)."));
    assert!(DOC.contains("onboarding self-approval is rejected (`Regression: #523`)."));
}

#[test]
fn doc_contains_governance_stake_slash_threshold_gate_integration() {
    assert!(DOC.contains("## Governance Stake/Slash Threshold Gate Integration"));
    assert!(DOC.contains("run_stake_slash_risk_contract_lane.sh"));
    assert!(DOC.contains("run_stake_slash_risk_deep_lane.sh"));
}

#[test]
fn regression_requires_stake_slash_tamper_and_threshold_fail_closed_rule() {
    // Regression: #733
    assert!(DOC.contains("tampered or incomplete risk evidence fails closed (`Regression: #733`)."));
}
