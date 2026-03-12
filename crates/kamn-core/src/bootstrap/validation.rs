use super::error_mapping::{
    map_channel_store_validation_error, map_content_store_validation_error,
    map_did_registry_store_validation_error, map_durable_guard_store_validation_error,
    map_message_lifecycle_store_validation_error, map_runtime_snapshot_store_error,
    map_task_operation_store_validation_error,
};
use super::models::{RuntimePersistenceLayout, RuntimeStoreAdapter, DID_REGISTRY_BOOTSTRAP_PROVIDER};
use crate::channel_models::{ChannelSnapshotStore, FileChannelSnapshotStore, SqliteChannelSnapshotStore};
use crate::config::ConfigError;
use crate::content_storage::FileContentAdapter;
use crate::did_registry::FileDidRegistrationChainAdapter;
use crate::durable_guard_store::{
    DurableGuardBundleSnapshotStore, FileDurableGuardSnapshotStore, SqliteDurableGuardSnapshotStore,
};
use crate::message_lifecycle::{
    FileMessageLifecycleSnapshotStore, MessageLifecycleSnapshotStore, SqliteMessageLifecycleSnapshotStore,
};
use crate::runtime::{FileRuntimeSnapshotStore, RuntimeSnapshotStore, SqliteRuntimeSnapshotStore};
use crate::task_operations::{
    FileTaskOperationSnapshotStore, SqliteTaskOperationSnapshotStore, TaskOperationSnapshotStore,
};
use std::fs;
use std::path::Path;

pub fn validate_runtime_persistence_layout(layout: &RuntimePersistenceLayout) -> Result<(), ConfigError> {
    fs::create_dir_all(&layout.storage_root).map_err(|error| ConfigError::RuntimeStoreCompatibility {
        store: "runtime-storage-root",
        reason_code: "runtime_storage_root_unavailable",
        detail: error.to_string(),
    })?;
    validate_content_store_path(&layout.content_store_path)?;
    validate_did_registry_store_path(&layout.did_registry_store_path)?;
    match &layout.runtime_store_adapter {
        RuntimeStoreAdapter::File => validate_file_store_paths(layout),
        RuntimeStoreAdapter::Sqlite { database_path } => validate_sqlite_store_paths(database_path),
    }
}

fn validate_file_store_paths(layout: &RuntimePersistenceLayout) -> Result<(), ConfigError> {
    validate_task_operation_store_path(&layout.task_operation_store_path)?;
    validate_durable_guard_store_path(&layout.durable_guard_store_path)?;
    validate_channel_snapshot_store_path(&layout.channel_snapshot_store_path)?;
    validate_message_lifecycle_snapshot_store_path(&layout.message_lifecycle_snapshot_store_path)?;
    validate_runtime_snapshot_store_path(&layout.runtime_snapshot_store_path)
}

fn validate_sqlite_store_paths(path: &Path) -> Result<(), ConfigError> {
    validate_task_operation_store_sqlite_path(path)?;
    validate_durable_guard_store_sqlite_path(path)?;
    validate_channel_snapshot_store_sqlite_path(path)?;
    validate_message_lifecycle_snapshot_store_sqlite_path(path)?;
    validate_runtime_snapshot_store_sqlite_path(path)
}

fn validate_content_store_path(path: &Path) -> Result<(), ConfigError> {
    FileContentAdapter::new(path.to_path_buf()).map(|_| ()).map_err(map_content_store_validation_error)
}

fn validate_did_registry_store_path(path: &Path) -> Result<(), ConfigError> {
    FileDidRegistrationChainAdapter::new(path.to_path_buf(), DID_REGISTRY_BOOTSTRAP_PROVIDER)
        .map(|_| ())
        .map_err(map_did_registry_store_validation_error)
}

fn validate_task_operation_store_path(path: &Path) -> Result<(), ConfigError> {
    FileTaskOperationSnapshotStore::new(path.to_path_buf())
        .map_err(map_task_operation_store_validation_error)?
        .read_latest()
        .map(|_| ())
        .map_err(map_task_operation_store_validation_error)
}

fn validate_durable_guard_store_path(path: &Path) -> Result<(), ConfigError> {
    FileDurableGuardSnapshotStore::new(path.to_path_buf())
        .map_err(map_durable_guard_store_validation_error)?
        .load_bundle()
        .map(|_| ())
        .map_err(map_durable_guard_store_validation_error)
}

fn validate_channel_snapshot_store_path(path: &Path) -> Result<(), ConfigError> {
    FileChannelSnapshotStore::new(path.to_path_buf())
        .map_err(map_channel_store_validation_error)?
        .read_latest()
        .map(|_| ())
        .map_err(map_channel_store_validation_error)
}

fn validate_message_lifecycle_snapshot_store_path(path: &Path) -> Result<(), ConfigError> {
    FileMessageLifecycleSnapshotStore::new(path.to_path_buf())
        .map_err(map_message_lifecycle_store_validation_error)?
        .read_latest()
        .map(|_| ())
        .map_err(map_message_lifecycle_store_validation_error)
}

fn validate_runtime_snapshot_store_path(path: &Path) -> Result<(), ConfigError> {
    FileRuntimeSnapshotStore::new(path.to_path_buf())
        .map_err(map_runtime_snapshot_store_error)?
        .read_latest()
        .map(|_| ())
        .map_err(map_runtime_snapshot_store_error)
}

fn validate_task_operation_store_sqlite_path(path: &Path) -> Result<(), ConfigError> {
    SqliteTaskOperationSnapshotStore::new(path.to_path_buf())
        .map_err(map_task_operation_store_validation_error)?
        .read_latest()
        .map(|_| ())
        .map_err(map_task_operation_store_validation_error)
}

fn validate_durable_guard_store_sqlite_path(path: &Path) -> Result<(), ConfigError> {
    SqliteDurableGuardSnapshotStore::new(path.to_path_buf())
        .map_err(map_durable_guard_store_validation_error)?
        .load_bundle()
        .map(|_| ())
        .map_err(map_durable_guard_store_validation_error)
}

fn validate_channel_snapshot_store_sqlite_path(path: &Path) -> Result<(), ConfigError> {
    SqliteChannelSnapshotStore::new(path.to_path_buf())
        .map_err(map_channel_store_validation_error)?
        .read_latest()
        .map(|_| ())
        .map_err(map_channel_store_validation_error)
}

fn validate_message_lifecycle_snapshot_store_sqlite_path(path: &Path) -> Result<(), ConfigError> {
    SqliteMessageLifecycleSnapshotStore::new(path.to_path_buf())
        .map_err(map_message_lifecycle_store_validation_error)?
        .read_latest()
        .map(|_| ())
        .map_err(map_message_lifecycle_store_validation_error)
}

fn validate_runtime_snapshot_store_sqlite_path(path: &Path) -> Result<(), ConfigError> {
    SqliteRuntimeSnapshotStore::new(path.to_path_buf())
        .map_err(map_runtime_snapshot_store_error)?
        .read_latest()
        .map(|_| ())
        .map_err(map_runtime_snapshot_store_error)
}
