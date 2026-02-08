const DOC: &str = include_str!("../../../docs/foundation/content-retention-tombstones.md");

#[test]
fn doc_contains_retention_class_and_lifecycle_scope() {
    assert!(DOC.contains("## Scope Delivered"));
    assert!(DOC.contains("ContentRetentionClass"));
    assert!(DOC.contains("ContentLifecycleManager"));
    assert!(DOC.contains("ContentCleanupActionKind"));
}

#[test]
fn doc_contains_cleanup_and_deleted_reference_rules() {
    assert!(DOC.contains("## Lifecycle and Cleanup Rules"));
    assert!(DOC.contains("Active` -> `Expired` -> `Tombstoned` -> `Purged"));
    assert!(DOC.contains("## Deleted Reference Semantics"));
    assert!(DOC.contains("assert_uri_accessible(...)"));
}

#[test]
fn doc_contains_fast_and_cost_effective_validation_lane() {
    assert!(DOC.contains("## Fast and Cost-Effective Validation"));
    assert!(DOC.contains("cargo test -p kamn-core --test content_retention_tombstones"));
    assert!(DOC.contains("cargo clippy -p kamn-core -- -D warnings"));
}

#[test]
fn regression_requires_deleted_reference_replay_block_rule() {
    // Regression: #163
    assert!(DOC.contains("Deleted/tombstoned references remain blocked under replay attempts."));
}
