use super::support::{
    temp_storage_dir, write_fixture, CHANNEL_STORE_FIXTURE, CONTENT_STORE_FIXTURE,
    MESSAGE_LIFECYCLE_STORE_FIXTURE, RUNTIME_SNAPSHOT_STORE_FIXTURE, TASK_OPERATION_STORE_FIXTURE,
};
use crate::bootstrap::bootstrap;
use crate::config::{ConfigError, NodeConfig, NodeRole, SyncMode};

#[test]
fn regression_bootstrap_fails_closed_when_content_store_payload_is_corrupt() {
    assert_corrupt_payload(CONTENT_STORE_FIXTURE, "schema|kamn.content.file-store.v1\nobject|broken\n", "content-storage", "content_storage_corrupt_payload_rejected");
}

#[test]
fn regression_bootstrap_fails_closed_when_task_snapshot_schema_is_incompatible() {
    assert_schema_error(TASK_OPERATION_STORE_FIXTURE, "schema|99\n", "task-operation-snapshot-store", "task_operation_snapshot_schema_mismatch_rejected", "1", "99");
}

#[test]
fn regression_bootstrap_fails_closed_when_channel_snapshot_payload_is_corrupt() {
    assert_corrupt_payload(CHANNEL_STORE_FIXTURE, "schema|1\nbroken\n", "channel-snapshot-store", "channel_snapshot_corrupt_payload_rejected");
}

#[test]
fn regression_bootstrap_fails_closed_when_channel_snapshot_schema_is_incompatible() {
    assert_schema_error(CHANNEL_STORE_FIXTURE, "schema|99\n", "channel-snapshot-store", "channel_snapshot_schema_mismatch_rejected", "1", "99");
}

#[test]
fn regression_bootstrap_fails_closed_when_message_snapshot_payload_is_corrupt() {
    assert_corrupt_payload(MESSAGE_LIFECYCLE_STORE_FIXTURE, "schema|1\nbroken\n", "message-lifecycle-snapshot-store", "message_lifecycle_snapshot_corrupt_payload_rejected");
}

#[test]
fn regression_bootstrap_fails_closed_when_message_snapshot_schema_is_incompatible() {
    assert_schema_error(MESSAGE_LIFECYCLE_STORE_FIXTURE, "schema|99\n", "message-lifecycle-snapshot-store", "message_lifecycle_snapshot_schema_mismatch_rejected", "1", "99");
}

#[test]
fn regression_bootstrap_fails_closed_when_runtime_snapshot_payload_is_corrupt() {
    assert_corrupt_payload(RUNTIME_SNAPSHOT_STORE_FIXTURE, "not-a-valid-snapshot-line\n", "runtime-snapshot-store", "runtime_snapshot_corrupt_payload_rejected");
}

#[test]
fn regression_bootstrap_fails_closed_when_runtime_snapshot_state_version_regresses() {
    assert_schema_error(
        RUNTIME_SNAPSHOT_STORE_FIXTURE,
        "10|statehash_a|10\n9|statehash_b|11\n",
        "runtime-snapshot-store",
        "runtime_snapshot_state_version_regression_rejected",
        ">10",
        "9",
    );
}

fn assert_corrupt_payload(fixture: &str, contents: &str, store: &'static str, reason_code: &'static str) {
    let storage_dir = temp_storage_dir(fixture);
    write_fixture(storage_dir.join(fixture), contents);
    let result = bootstrap(config_for(&storage_dir));
    assert!(matches!(result, Err(ConfigError::RuntimeStoreCorruptPayload { store: actual_store, reason_code: actual_reason, .. }) if actual_store == store && actual_reason == reason_code));
}

fn assert_schema_error(
    fixture: &str,
    contents: &str,
    store: &'static str,
    reason_code: &'static str,
    expected: &'static str,
    found: &'static str,
) {
    let storage_dir = temp_storage_dir(fixture);
    write_fixture(storage_dir.join(fixture), contents);
    let result = bootstrap(config_for(&storage_dir));
    assert!(matches!(result, Err(ConfigError::RuntimeStoreSchemaIncompatible { store: actual_store, reason_code: actual_reason, expected: actual_expected, found: actual_found }) if actual_store == store && actual_reason == reason_code && actual_expected == expected && actual_found == found));
}

fn config_for(storage_dir: &std::path::Path) -> NodeConfig {
    NodeConfig {
        chain_id: "kamn-devnet".to_owned(),
        chain_version: "v0.1.0".to_owned(),
        role: NodeRole::Processor,
        storage_dir: storage_dir.to_string_lossy().into_owned(),
        enable_gossip: true,
        sync_mode: SyncMode::Fast,
    }
}
