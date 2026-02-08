const DOC: &str = include_str!("../../../docs/foundation/operator-dashboard-ui-mvp.md");

#[test]
fn doc_contains_ui_scope_and_composer_contract() {
    assert!(DOC.contains("## Scope Delivered"));
    assert!(DOC.contains("OperatorDashboardUi"));
    assert!(DOC.contains("DashboardSummary"));
    assert!(DOC.contains("OperatorDashboardUiError"));
    assert!(DOC.contains("packages/kamn-dashboard"));
}

#[test]
fn doc_contains_section_and_audit_trace_rules() {
    assert!(DOC.contains("## UI Composition Rules"));
    assert!(DOC.contains("## Audit Trace Rules"));
    assert!(DOC.contains("Denied operator actions are marked critical"));
    assert!(DOC.contains("dashboard-loading"));
    assert!(DOC.contains("dashboard-error"));
    assert!(DOC.contains("dashboard-empty"));
}

#[test]
fn doc_contains_fast_and_cost_effective_validation_lane() {
    assert!(DOC.contains("## Fast and Cost-Effective Validation"));
    assert!(DOC.contains("cargo test -p kamn-core --test operator_dashboard_ui"));
    assert!(DOC.contains("npm --prefix packages/kamn-dashboard test"));
    assert!(DOC.contains("bash scripts/frontend/test_dashboard_package.sh"));
    assert!(DOC.contains("cargo clippy -p kamn-core -- -D warnings"));
}

#[test]
fn regression_requires_newest_first_audit_trace_ordering_rule() {
    // Regression: #201
    assert!(DOC.contains("newest `requested_at_unix` first"));
}

#[test]
fn regression_requires_critical_badge_with_stale_banner_rule() {
    // Regression: #591
    assert!(DOC.contains("stale-data-banner"));
    assert!(DOC.contains("severity-critical"));
    assert!(DOC.contains("Regression: #591"));
}
