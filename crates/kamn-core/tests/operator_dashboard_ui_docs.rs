const DOC: &str = include_str!("../../../docs/foundation/operator-dashboard-ui-mvp.md");

#[test]
fn doc_contains_ui_scope_and_composer_contract() {
    assert!(DOC.contains("## Scope Delivered"));
    assert!(DOC.contains("OperatorDashboardUi"));
    assert!(DOC.contains("DashboardSummary"));
    assert!(DOC.contains("OperatorDashboardUiError"));
    assert!(DOC.contains("packages/kamn-dashboard"));
    assert!(DOC.contains("src/live_api.ts"));
    assert!(DOC.contains("buildDashboardShellFromBackend(...)"));
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
fn doc_contains_frontend_shell_matrix_contract_lane() {
    assert!(DOC.contains("## Frontend Shell Determinism Matrix Contract"));
    assert!(DOC.contains("run_dashboard_shell_determinism_matrix_lane.sh"));
    assert!(DOC.contains("check_dashboard_shell_determinism_matrix_policy.sh"));
    assert!(DOC.contains("run_dashboard_shell_determinism_matrix_contract_lane.sh"));
    assert!(DOC.contains("kamn.frontend.shell-matrix-report.v1"));
    assert!(DOC.contains("KAMN_FRONTEND_SHELL_MATRIX_MAX_SECONDS"));
    assert!(DOC.contains("KAMN_FRONTEND_SHELL_MATRIX_CONTRACT_MAX_SECONDS"));
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

#[test]
fn regression_requires_live_backend_error_shell_rule() {
    // Regression: #639
    assert!(DOC.contains("fetchDashboardSnapshotFromBackend(...)"));
    assert!(DOC.contains("dashboard-error"));
    assert!(DOC.contains("Regression: #639"));
}

#[test]
fn regression_requires_live_session_gate_rule() {
    // Regression: #640
    assert!(DOC.contains("missing/expired/unauthorized operator sessions"));
    assert!(DOC.contains("Regression: #640"));
}

#[test]
fn regression_requires_frontend_shell_matrix_fail_closed_rules() {
    // Regression: #943
    assert!(DOC.contains(
        "healthy/stale-critical/error shell drift, docs parity drift, or runtime budget overflow force `NO-GO` (`Regression: #943`)."
    ));
}
