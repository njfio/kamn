const SHARED_DOC_HARNESS_SOURCE: &str = include_str!("./node_runtime_cli_docs.rs");

#[test]
fn regression_kolme_runtime_commit_docs_migrated_into_shared_harness_matrix() {
    assert!(
        SHARED_DOC_HARNESS_SOURCE
            .contains("migration_kolme_runtime_commit_signer_provenance_failure_taxonomy_markers"),
        "shared harness must retain signer provenance taxonomy marker coverage"
    );
    assert!(
        SHARED_DOC_HARNESS_SOURCE
            .contains("migration_kolme_runtime_commit_transient_classifier_and_retry_markers"),
        "shared harness must retain transient classifier marker coverage"
    );
    assert!(
        SHARED_DOC_HARNESS_SOURCE
            .contains("migration_kolme_runtime_commit_notifications_reconnect_pacing_markers"),
        "shared harness must retain notifications reconnect marker coverage"
    );
}
