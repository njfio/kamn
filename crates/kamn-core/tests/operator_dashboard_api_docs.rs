const DOC: &str = include_str!("../../../docs/foundation/operator-dashboard-backend-apis.md");

#[test]
fn doc_contains_dashboard_backend_scope_and_contracts() {
    assert!(DOC.contains("## Scope Delivered"));
    assert!(DOC.contains("OperatorDashboardApi"));
    assert!(DOC.contains("DashboardPageRequest"));
    assert!(DOC.contains("snapshot(...)"));
    assert!(DOC.contains("packages/kamn-dashboard"));
}

#[test]
fn doc_contains_pagination_and_filter_rules() {
    assert!(DOC.contains("## Pagination and Filter Rules"));
    assert!(DOC.contains("Cursor tokens must match an existing key"));
    assert!(DOC.contains("Optional prefix filter applies before pagination."));
}

#[test]
fn doc_contains_fast_and_cost_effective_validation_lane() {
    assert!(DOC.contains("## Fast and Cost-Effective Validation"));
    assert!(DOC.contains("cargo test -p kamn-core --test operator_dashboard_api"));
    assert!(DOC.contains("npm --prefix packages/kamn-dashboard test"));
    assert!(DOC.contains("cargo clippy -p kamn-core -- -D warnings"));
}

#[test]
fn regression_requires_tampered_cursor_rejection_rule() {
    // Regression: #203
    assert!(DOC.contains("Cursor tokens must match an existing key"));
}

#[test]
fn regression_requires_frontend_state_mapping_contract_rules() {
    // Regression: #591
    assert!(DOC.contains("dashboard-loading"));
    assert!(DOC.contains("dashboard-error"));
    assert!(DOC.contains("dashboard-empty"));
}
