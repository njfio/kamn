const SHARED_DOC_HARNESS_SOURCE: &str = include_str!("./node_runtime_cli_docs.rs");

#[test]
fn regression_r42_api_shell_surface_audit_docs_migrated_into_shared_harness_matrix() {
    assert!(
        SHARED_DOC_HARNESS_SOURCE
            .contains("migration_r42_api_shell_surface_audit_inventory_versions"),
        "shared harness must retain R42 audit inventory marker coverage"
    );
    assert!(
        SHARED_DOC_HARNESS_SOURCE
            .contains("migration_r42_api_shell_surface_audit_ratchet_recommendations"),
        "shared harness must retain R42 audit ratchet marker coverage"
    );
    assert!(
        SHARED_DOC_HARNESS_SOURCE.contains("migration_r42_api_shell_surface_audit_follow_up_tasks"),
        "shared harness must retain R42 audit follow-up marker coverage"
    );
}
