mod helpers;

use self::helpers::{
    corrupt_payload, durable_guard_restore_error, durable_guard_schema_error, io_error,
    map_channel_snapshot_error, map_message_lifecycle_snapshot_error,
    map_task_operation_snapshot_error, runtime_regression_error, runtime_stale_hash_error,
};
use crate::channel_models::ChannelSnapshotStoreError;
use crate::config::ConfigError;
use crate::durable_guard_store::DurableGuardSnapshotStoreError;
use crate::message_lifecycle::MessageLifecycleSnapshotStoreError;
use crate::runtime::SnapshotStoreError;
use crate::task_operations::TaskOperationSnapshotStoreError;

pub fn map_task_operation_store_validation_error(error: TaskOperationSnapshotStoreError) -> ConfigError {
    match error {
        TaskOperationSnapshotStoreError::InvalidPayload(detail) => corrupt_payload(
            "task-operation-snapshot-store",
            "task_operation_snapshot_corrupt_payload_rejected",
            detail,
        ),
        TaskOperationSnapshotStoreError::Io(detail) => io_error(
            "task-operation-snapshot-store",
            "task_operation_snapshot_io_error",
            detail,
        ),
        TaskOperationSnapshotStoreError::Snapshot(other) => map_task_operation_snapshot_error(other),
    }
}

pub fn map_durable_guard_store_validation_error(error: DurableGuardSnapshotStoreError) -> ConfigError {
    match error {
        DurableGuardSnapshotStoreError::BundleSchemaVersionMismatch { expected, found } => {
            durable_guard_schema_error(expected, found)
        }
        DurableGuardSnapshotStoreError::InvalidPayload(detail) => corrupt_payload(
            "durable-guard-snapshot-store",
            "durable_guard_snapshot_corrupt_payload_rejected",
            detail,
        ),
        DurableGuardSnapshotStoreError::Io(detail) => io_error(
            "durable-guard-snapshot-store",
            "durable_guard_snapshot_io_error",
            detail,
        ),
        DurableGuardSnapshotStoreError::DeliverySnapshot(other) => durable_guard_restore_error(other),
        DurableGuardSnapshotStoreError::ChannelPolicySnapshot(other) => durable_guard_restore_error(other),
    }
}

pub fn map_channel_store_validation_error(error: ChannelSnapshotStoreError) -> ConfigError {
    match error {
        ChannelSnapshotStoreError::InvalidPayload(detail) => corrupt_payload(
            "channel-snapshot-store",
            "channel_snapshot_corrupt_payload_rejected",
            detail,
        ),
        ChannelSnapshotStoreError::Io(detail) => io_error(
            "channel-snapshot-store",
            "channel_snapshot_io_error",
            detail,
        ),
        ChannelSnapshotStoreError::Snapshot(other) => map_channel_snapshot_error(other),
    }
}

pub fn map_message_lifecycle_store_validation_error(
    error: MessageLifecycleSnapshotStoreError,
) -> ConfigError {
    match error {
        MessageLifecycleSnapshotStoreError::InvalidPayload(detail) => corrupt_payload(
            "message-lifecycle-snapshot-store",
            "message_lifecycle_snapshot_corrupt_payload_rejected",
            detail,
        ),
        MessageLifecycleSnapshotStoreError::Io(detail) => io_error(
            "message-lifecycle-snapshot-store",
            "message_lifecycle_snapshot_io_error",
            detail,
        ),
        MessageLifecycleSnapshotStoreError::Snapshot(other) => map_message_lifecycle_snapshot_error(other),
    }
}

pub fn map_runtime_snapshot_store_error(error: SnapshotStoreError) -> ConfigError {
    match error {
        SnapshotStoreError::InvalidPayload(detail) => corrupt_payload(
            "runtime-snapshot-store",
            "runtime_snapshot_corrupt_payload_rejected",
            detail,
        ),
        SnapshotStoreError::Io(detail) => io_error(
            "runtime-snapshot-store",
            "runtime_snapshot_io_error",
            detail,
        ),
        SnapshotStoreError::StateVersionRegression { previous, found } => {
            runtime_regression_error("runtime_snapshot_state_version_regression_rejected", previous, found)
        }
        SnapshotStoreError::CursorRegression { previous, found } => {
            runtime_regression_error("runtime_snapshot_cursor_regression_rejected", previous, found)
        }
        SnapshotStoreError::StaleStateHash { state_hash, previous_version, found_version } => {
            runtime_stale_hash_error(state_hash, previous_version, found_version)
        }
    }
}

pub(crate) fn schema_error(
    store: &'static str,
    reason_code: &'static str,
    expected: String,
    found: String,
) -> ConfigError {
    ConfigError::RuntimeStoreSchemaIncompatible { store, reason_code, expected, found }
}
