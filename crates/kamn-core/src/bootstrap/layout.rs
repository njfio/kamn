use super::models::{
    RuntimePersistenceLayout, RuntimeStoreAdapter, CHANNEL_SNAPSHOT_STORE_COMPONENT_FILE,
    CHANNEL_SNAPSHOT_STORE_COMPONENT_SQLITE, CHANNEL_SNAPSHOT_STORE_FILE_NAME,
    CONTENT_STORE_COMPONENT, CONTENT_STORE_FILE_NAME, DID_REGISTRY_STORE_COMPONENT,
    DID_REGISTRY_STORE_FILE_NAME, DURABLE_GUARD_STORE_COMPONENT_FILE,
    DURABLE_GUARD_STORE_COMPONENT_SQLITE, DURABLE_GUARD_STORE_FILE_NAME,
    MESSAGE_LIFECYCLE_SNAPSHOT_STORE_COMPONENT_FILE,
    MESSAGE_LIFECYCLE_SNAPSHOT_STORE_COMPONENT_SQLITE, MESSAGE_LIFECYCLE_SNAPSHOT_STORE_FILE_NAME,
    RUNTIME_SNAPSHOT_STORE_COMPONENT_FILE, RUNTIME_SNAPSHOT_STORE_COMPONENT_SQLITE,
    RUNTIME_SNAPSHOT_STORE_FILE_NAME, SQLITE_STORAGE_SELECTOR_PREFIX,
    TASK_OPERATION_STORE_COMPONENT_FILE, TASK_OPERATION_STORE_COMPONENT_SQLITE,
    TASK_OPERATION_STORE_FILE_NAME,
};
use crate::config::ConfigError;
use std::path::{Path, PathBuf};

pub fn resolve_runtime_persistence_layout(
    storage_dir: &str,
) -> Result<RuntimePersistenceLayout, ConfigError> {
    if let Some(database_path_raw) = storage_dir.strip_prefix(SQLITE_STORAGE_SELECTOR_PREFIX) {
        return sqlite_layout(database_path_raw);
    }
    Ok(file_layout(PathBuf::from(storage_dir)))
}

pub fn prioritized_runtime_store_components(
    store_adapter: &RuntimeStoreAdapter,
) -> [&'static str; 7] {
    match store_adapter {
        RuntimeStoreAdapter::File => [
            CONTENT_STORE_COMPONENT,
            DID_REGISTRY_STORE_COMPONENT,
            TASK_OPERATION_STORE_COMPONENT_FILE,
            DURABLE_GUARD_STORE_COMPONENT_FILE,
            CHANNEL_SNAPSHOT_STORE_COMPONENT_FILE,
            MESSAGE_LIFECYCLE_SNAPSHOT_STORE_COMPONENT_FILE,
            RUNTIME_SNAPSHOT_STORE_COMPONENT_FILE,
        ],
        RuntimeStoreAdapter::Sqlite { .. } => [
            CONTENT_STORE_COMPONENT,
            DID_REGISTRY_STORE_COMPONENT,
            TASK_OPERATION_STORE_COMPONENT_SQLITE,
            DURABLE_GUARD_STORE_COMPONENT_SQLITE,
            CHANNEL_SNAPSHOT_STORE_COMPONENT_SQLITE,
            MESSAGE_LIFECYCLE_SNAPSHOT_STORE_COMPONENT_SQLITE,
            RUNTIME_SNAPSHOT_STORE_COMPONENT_SQLITE,
        ],
    }
}

fn sqlite_layout(database_path_raw: &str) -> Result<RuntimePersistenceLayout, ConfigError> {
    let trimmed = database_path_raw.trim();
    if trimmed.is_empty() {
        return Err(invalid_sqlite_selector_error());
    }
    let database_path = PathBuf::from(trimmed);
    let storage_root = sqlite_storage_root(&database_path);
    Ok(RuntimePersistenceLayout {
        content_store_path: storage_root.join(CONTENT_STORE_FILE_NAME),
        did_registry_store_path: storage_root.join(DID_REGISTRY_STORE_FILE_NAME),
        task_operation_store_path: database_path.clone(),
        durable_guard_store_path: database_path.clone(),
        channel_snapshot_store_path: database_path.clone(),
        message_lifecycle_snapshot_store_path: database_path.clone(),
        runtime_snapshot_store_path: database_path.clone(),
        storage_root,
        runtime_store_adapter: RuntimeStoreAdapter::Sqlite { database_path },
    })
}

fn invalid_sqlite_selector_error() -> ConfigError {
    ConfigError::RuntimeStoreCompatibility {
        store: "runtime-storage-root",
        reason_code: "runtime_storage_root_invalid_sqlite_selector",
        detail: "sqlite storage_dir selector must include a database path".to_owned(),
    }
}

fn sqlite_storage_root(database_path: &Path) -> PathBuf {
    database_path
        .parent()
        .map(Path::to_path_buf)
        .unwrap_or_else(|| PathBuf::from("."))
}

fn file_layout(storage_root: PathBuf) -> RuntimePersistenceLayout {
    RuntimePersistenceLayout {
        content_store_path: storage_root.join(CONTENT_STORE_FILE_NAME),
        did_registry_store_path: storage_root.join(DID_REGISTRY_STORE_FILE_NAME),
        task_operation_store_path: storage_root.join(TASK_OPERATION_STORE_FILE_NAME),
        durable_guard_store_path: storage_root.join(DURABLE_GUARD_STORE_FILE_NAME),
        channel_snapshot_store_path: storage_root.join(CHANNEL_SNAPSHOT_STORE_FILE_NAME),
        message_lifecycle_snapshot_store_path: storage_root
            .join(MESSAGE_LIFECYCLE_SNAPSHOT_STORE_FILE_NAME),
        runtime_snapshot_store_path: storage_root.join(RUNTIME_SNAPSHOT_STORE_FILE_NAME),
        storage_root,
        runtime_store_adapter: RuntimeStoreAdapter::File,
    }
}
