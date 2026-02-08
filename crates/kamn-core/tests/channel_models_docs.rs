const DOC: &str = include_str!("../../../docs/foundation/channel-models.md");

#[test]
fn doc_contains_core_channel_models_and_operations() {
    assert!(DOC.contains("## Core Models"));
    assert!(DOC.contains("ChannelType"));
    assert!(DOC.contains("ChannelMetadata"));
    assert!(DOC.contains("ChannelStore"));
    assert!(DOC.contains("## Supported Operations"));
    assert!(DOC.contains("create_group(channel_id, creator, members, admins)"));
}

#[test]
fn doc_contains_snapshot_persistence_and_restore_contract_rules() {
    assert!(DOC.contains("## Snapshot Persistence and Restore Contract Rules"));
    assert!(DOC.contains("export_snapshot()"));
    assert!(DOC.contains("restore_snapshot(snapshot)"));
    assert!(DOC.contains("ChannelSnapshotStore"));
    assert!(DOC.contains("recover_latest_and_repair()"));
    assert!(DOC.contains("CHANNEL_SNAPSHOT_SCHEMA_VERSION"));
}

#[test]
fn doc_contains_fast_and_cost_effective_validation_lane_commands() {
    assert!(DOC.contains("## Fast and Cost-Effective Validation"));
    assert!(DOC.contains("cargo test -p kamn-core --lib channel_models::tests::"));
    assert!(DOC.contains("cargo test -p kamn-core --test channel_models"));
    assert!(DOC.contains("cargo test -p kamn-core --test channel_models_docs"));
    assert!(DOC.contains("bash scripts/channel/run_channel_lifecycle_contract_lane.sh"));
    assert!(DOC.contains(
        "cargo test -p kamn-core --lib channel_models::tests::performance_channel_snapshot_deep_lane_stress -- --ignored"
    ));
    assert!(DOC.contains("bash scripts/channel/run_channel_lifecycle_deep_lane.sh"));
    assert!(DOC.contains("cargo clippy -p kamn-core -- -D warnings"));
}

#[test]
fn regression_requires_channel_snapshot_restore_guard_rules() {
    // Regression: #617
    assert!(DOC.contains("duplicate channel IDs on restore are rejected (`Regression: #617`)"));
    assert!(DOC.contains("admin/member mismatch on restore is rejected (`Regression: #617`)"));
}
