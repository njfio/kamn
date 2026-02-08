const DOC: &str = include_str!("../../../docs/foundation/runtime-processor-ha.md");

#[test]
fn doc_contains_processor_ha_scope_and_models() {
    assert!(DOC.contains("## Scope Delivered"));
    assert!(DOC.contains("## Snapshot Restore Rules"));
    assert!(DOC.contains("## Construct Lock Rules"));
    assert!(DOC.contains("## Fast and Cost-Effective Validation"));
}

#[test]
fn doc_contains_fast_lane_command_references() {
    assert!(DOC.contains("cargo test -p kamn-node --test runtime_processor_ha_docs"));
    assert!(DOC.contains("cargo test -p kamn-node --test node_runtime_cli_docs"));
}

#[test]
fn regression_requires_snapshot_restore_guard_rules() {
    // Regression: #361
    assert!(DOC.contains("snapshot version/hash mismatch restores are rejected"));
}

#[test]
fn regression_requires_construct_lock_guard_rules() {
    // Regression: #362
    assert!(DOC.contains("split-brain lock acquisition attempts are rejected"));
    assert!(DOC.contains("stale lease renewal attempts are rejected"));
}
