const DOC: &str = include_str!("../../../docs/foundation/operator-permissioned-actions.md");

#[test]
fn doc_contains_scope_and_service_contracts() {
    assert!(DOC.contains("## Scope Delivered"));
    assert!(DOC.contains("PermissionedOperatorActionService"));
    assert!(DOC.contains("OperatorActionAuditRecord"));
    assert!(DOC.contains("OperatorActionServiceError"));
}

#[test]
fn doc_contains_binding_authorization_rules() {
    assert!(DOC.contains("## Authorization Rules"));
    assert!(DOC.contains("OperatorBindingAction::Configure"));
    assert!(DOC.contains("OperatorBindingAction::ReadHistory"));
    assert!(DOC.contains("Unauthorized requests return explicit binding errors"));
}

#[test]
fn doc_contains_fast_and_cost_effective_validation_lane() {
    assert!(DOC.contains("## Fast and Cost-Effective Validation"));
    assert!(DOC.contains("cargo test -p kamn-core --test operator_permissioned_actions"));
    assert!(DOC.contains("cargo clippy -p kamn-core -- -D warnings"));
}

#[test]
fn regression_requires_post_revoke_blocking_rule() {
    // Regression: #199
    assert!(DOC.contains("Revoked bindings cannot be reused"));
}
