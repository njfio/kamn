const DOC: &str =
    include_str!("../../../docs/foundation/agent-driven-upgrade-proposal-workflow.md");

#[test]
fn doc_contains_agent_upgrade_scope_and_models() {
    assert!(DOC.contains("## Scope Delivered"));
    assert!(DOC.contains("AgentDrivenUpgradeWorkflow"));
    assert!(DOC.contains("AgentUpgradeWorkflowConfig"));
    assert!(DOC.contains("AgentUpgradeWorkflowError"));
}

#[test]
fn doc_contains_workflow_safeguards_and_audit_rules() {
    assert!(DOC.contains("## Workflow Safeguards"));
    assert!(DOC.contains("## Governance and Audit Rules"));
    assert!(DOC.contains("Governance submission requires:"));
}

#[test]
fn doc_contains_fast_and_cost_effective_validation_lane() {
    assert!(DOC.contains("## Fast and Cost-Effective Validation"));
    assert!(DOC.contains("cargo test -p kamn-core --test agent_upgrade_workflow"));
    assert!(DOC.contains("cargo clippy -p kamn-core -- -D warnings"));
}

#[test]
fn regression_requires_human_review_quorum_gating_rule() {
    // Regression: #235
    assert!(DOC.contains("sufficient unique human reviewer approvals"));
}
