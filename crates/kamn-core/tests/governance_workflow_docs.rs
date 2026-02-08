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
