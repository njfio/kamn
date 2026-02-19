const AUDIT_DOC: &str = include_str!("../../../docs/review/r42-api-shell-surface-audit.md");

#[test]
fn unit_r42_audit_declares_surface_inventory_versions() {
    assert!(AUDIT_DOC.contains("public_api_surface_audit_version="));
    assert!(AUDIT_DOC.contains("shell_rust_test_surface_audit_version="));
}

#[test]
fn functional_r42_audit_declares_ratchet_recommendation_markers() {
    assert!(AUDIT_DOC.contains("api_surface_ratchet_recommendation_status=proposed"));
    assert!(AUDIT_DOC.contains("test_surface_ratchet_recommendation_status=proposed"));
    assert!(AUDIT_DOC.contains("kamn_core_public_item_count="));
    assert!(AUDIT_DOC.contains("shell_to_rust_test_file_ratio="));
}

#[test]
fn integration_r42_audit_links_follow_up_implementation_tasks() {
    assert!(AUDIT_DOC.contains("follow_up_issue_api_surface_ratchet=#5188"));
    assert!(AUDIT_DOC.contains("follow_up_issue_test_surface_migration=#5189"));
    assert!(AUDIT_DOC.contains("audit_follow_up_issue_count=2"));
}
