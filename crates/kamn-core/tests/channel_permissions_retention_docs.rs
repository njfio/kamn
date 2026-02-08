const DOC: &str = include_str!("../../../docs/foundation/channel-permissions-retention.md");

#[test]
fn doc_contains_channel_permissions_scope_and_models() {
    assert!(DOC.contains("# Channel Permissions and Retention Policies"));
    assert!(DOC.contains("ChannelPermissionEngine"));
    assert!(DOC.contains("PermissionRule"));
}

#[test]
fn regression_requires_allowlist_validation_rule() {
    // Regression: #458
    assert!(DOC.contains("Allowlist permission rules must not be empty"));
    assert!(DOC.contains("allowlist entries must be valid `kamn:did:agent:*` identifiers"));
    assert!(DOC.contains("malformed allowlist configuration is rejected (`Regression: #458`)"));
}
