const DOC: &str = include_str!("../../../docs/foundation/governance-proposal-vote-execution.md");

#[test]
fn doc_contains_governance_workflow_scope_and_models() {
    assert!(DOC.contains("## Scope Delivered"));
    assert!(DOC.contains("GovernanceWorkflow"));
    assert!(DOC.contains("GovernanceProposalDraft"));
    assert!(DOC.contains("GovernanceWorkflowError"));
}

#[test]
fn doc_contains_lifecycle_and_quorum_rules() {
    assert!(DOC.contains("## Proposal Lifecycle Rules"));
    assert!(DOC.contains("## Vote and Quorum Rules"));
    assert!(DOC.contains("yes votes reaching quorum => `Approved`."));
}

#[test]
fn doc_contains_fast_and_cost_effective_validation_lane() {
    assert!(DOC.contains("## Fast and Cost-Effective Validation"));
    assert!(DOC.contains("cargo test -p kamn-core --test governance_workflow"));
    assert!(DOC.contains("cargo clippy -p kamn-core -- -D warnings"));
}

#[test]
fn regression_requires_late_vote_rejection_rule() {
    // Regression: #197
    assert!(DOC.contains("Late votes after deadline are rejected"));
}

#[test]
fn regression_requires_parameter_payload_validation_rule() {
    // Regression: #476
    assert!(DOC.contains("## Parameter Proposal Validation Rules"));
    assert!(DOC.contains("semver-style target version"));
    assert!(DOC.contains("Regression: #476"));
}

#[test]
fn doc_contains_parameter_catalog_and_compatibility_policy() {
    assert!(DOC.contains("listener.quorum"));
    assert!(DOC.contains("watchdog.delivery_ratio_bps"));
    assert!(DOC.contains("supported from `1.1.0`"));
}

#[test]
fn doc_contains_simulation_and_human_veto_evidence_contract() {
    assert!(DOC.contains("## Proposal Simulation and Human-Veto Evidence Contract"));
    assert!(DOC.contains("generate_governance_simulation_evidence_bundle.sh"));
    assert!(DOC.contains("check_governance_simulation_policy.sh"));
    assert!(DOC.contains("run_governance_simulation_contract_lane.sh"));
    assert!(DOC.contains("run_governance_simulation_deep_lane.sh"));
    assert!(DOC.contains("run_governance_simulation_matrix.py"));
    assert!(DOC.contains("fixtures/governance_simulation/veto_timelock_cases.json"));
}

#[test]
fn regression_requires_simulation_and_veto_bypass_guard_marker() {
    // Regression: #733
    assert!(DOC.contains(
        "simulation/veto bypass attempts and tampered evidence bundles force `NO-GO` (`Regression: #733`)."
    ));
}

#[test]
fn doc_contains_stake_slash_risk_threshold_evidence_contract() {
    assert!(DOC.contains("## Stake/Slash Risk Threshold Evidence Contract"));
    assert!(DOC.contains("generate_stake_slash_risk_evidence_bundle.sh"));
    assert!(DOC.contains("check_stake_slash_risk_policy.sh"));
    assert!(DOC.contains("run_stake_slash_risk_contract_lane.sh"));
    assert!(DOC.contains("run_stake_slash_risk_deep_lane.sh"));
    assert!(DOC.contains("run_stake_slash_risk_matrix.py"));
    assert!(DOC.contains("fixtures/governance_stake_slash/risk_threshold_cases.json"));
}

#[test]
fn regression_requires_stake_slash_threshold_bypass_guard_marker() {
    // Regression: #733
    assert!(DOC.contains(
        "unsafe threshold bypass attempts and tampered risk evidence force `NO-GO` (`Regression: #733`)."
    ));
}
