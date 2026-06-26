use super::schema_error;
use crate::channel_models::ChannelSnapshotError;
use crate::config::ConfigError;
use crate::message_lifecycle::MessageLifecycleSnapshotError;
use crate::task_operations::TaskOperationError;

pub(crate) fn map_task_operation_snapshot_error(error: TaskOperationError) -> ConfigError {
    match error {
        TaskOperationError::SnapshotVersionMismatch { expected, found } => schema_error(
            "task-operation-snapshot-store",
            "task_operation_snapshot_schema_mismatch_rejected",
            expected.to_string(),
            found.to_string(),
        ),
        other => compatibility_error(
            "task-operation-snapshot-store",
            "task_operation_snapshot_restore_validation_failed",
            other,
        ),
    }
}

pub(crate) fn durable_guard_schema_error(expected: u16, found: u16) -> ConfigError {
    schema_error(
        "durable-guard-snapshot-store",
        "durable_guard_snapshot_schema_mismatch_rejected",
        expected.to_string(),
        found.to_string(),
    )
}

pub(crate) fn durable_guard_restore_error(other: impl ToString) -> ConfigError {
    compatibility_error(
        "durable-guard-snapshot-store",
        "durable_guard_snapshot_restore_validation_failed",
        other,
    )
}

pub(crate) fn map_channel_snapshot_error(error: ChannelSnapshotError) -> ConfigError {
    match error {
        ChannelSnapshotError::SnapshotVersionMismatch { expected, found } => schema_error(
            "channel-snapshot-store",
            "channel_snapshot_schema_mismatch_rejected",
            expected.to_string(),
            found.to_string(),
        ),
        other => compatibility_error(
            "channel-snapshot-store",
            "channel_snapshot_restore_validation_failed",
            other,
        ),
    }
}

pub(crate) fn map_message_lifecycle_snapshot_error(
    error: MessageLifecycleSnapshotError,
) -> ConfigError {
    match error {
        MessageLifecycleSnapshotError::SnapshotVersionMismatch { expected, found } => schema_error(
            "message-lifecycle-snapshot-store",
            "message_lifecycle_snapshot_schema_mismatch_rejected",
            expected.to_string(),
            found.to_string(),
        ),
        other => compatibility_error(
            "message-lifecycle-snapshot-store",
            "message_lifecycle_snapshot_restore_validation_failed",
            other,
        ),
    }
}

pub(crate) fn runtime_regression_error(
    reason_code: &'static str,
    previous: u64,
    found: u64,
) -> ConfigError {
    schema_error(
        "runtime-snapshot-store",
        reason_code,
        format!(">{previous}"),
        found.to_string(),
    )
}

pub(crate) fn runtime_stale_hash_error(
    state_hash: String,
    previous_version: u64,
    found_version: u64,
) -> ConfigError {
    schema_error(
        "runtime-snapshot-store",
        "runtime_snapshot_stale_hash_regression_rejected",
        format!("{state_hash}@>{previous_version}"),
        format!("{state_hash}@{found_version}"),
    )
}

pub(crate) fn compatibility_error(
    store: &'static str,
    reason_code: &'static str,
    other: impl ToString,
) -> ConfigError {
    ConfigError::RuntimeStoreCompatibility {
        store,
        reason_code,
        detail: other.to_string(),
    }
}

pub(crate) fn corrupt_payload(
    store: &'static str,
    reason_code: &'static str,
    detail: String,
) -> ConfigError {
    ConfigError::RuntimeStoreCorruptPayload {
        store,
        reason_code,
        detail,
    }
}

pub(crate) fn io_error(
    store: &'static str,
    reason_code: &'static str,
    detail: String,
) -> ConfigError {
    ConfigError::RuntimeStoreCompatibility {
        store,
        reason_code,
        detail,
    }
}
