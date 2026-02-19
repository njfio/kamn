const SHARED_DOC_HARNESS_SOURCE: &str = include_str!("./node_runtime_cli_docs.rs");

#[test]
fn regression_runtime_processor_ha_docs_migrated_into_shared_harness_matrix() {
    assert!(
        SHARED_DOC_HARNESS_SOURCE
            .contains("migration_runtime_processor_ha_doc_contains_scope_and_models"),
        "shared harness must retain runtime processor HA scope marker coverage"
    );
    assert!(
        SHARED_DOC_HARNESS_SOURCE
            .contains("migration_runtime_processor_ha_doc_contains_fast_lane_command_references"),
        "shared harness must retain runtime processor HA command marker coverage"
    );
    assert!(
        SHARED_DOC_HARNESS_SOURCE.contains(
            "migration_runtime_processor_ha_regression_construct_lock_release_transfer_tick_rules"
        ),
        "shared harness must retain runtime processor HA regression marker coverage"
    );
}
