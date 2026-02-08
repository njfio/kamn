const DOC: &str = include_str!("../../../docs/foundation/version-upgrade-orchestration-audit.md");

#[test]
fn doc_contains_upgrade_orchestrator_scope_and_models() {
    assert!(DOC.contains("## Scope Delivered"));
    assert!(DOC.contains("VersionUpgradeOrchestrator"));
    assert!(DOC.contains("UpgradeAuditEventKind"));
    assert!(DOC.contains("UpgradeOrchestrationError"));
}

#[test]
fn doc_contains_upgrade_gating_and_audit_rules() {
    assert!(DOC.contains("## Upgrade Gating Rules"));
    assert!(DOC.contains("## Governance Audit View Rules"));
    assert!(DOC.contains("Activation requires:"));
}

#[test]
fn doc_contains_fast_and_cost_effective_validation_lane() {
    assert!(DOC.contains("## Fast and Cost-Effective Validation"));
    assert!(DOC.contains("cargo test -p kamn-core --test upgrade_orchestration"));
    assert!(DOC.contains("cargo clippy -p kamn-core -- -D warnings"));
}

#[test]
fn regression_requires_activation_quorum_gating_rule() {
    // Regression: #193
    assert!(DOC.contains("sufficient unique validator approvals"));
}
