//! Bootstrap planning for validated config, schema migrations, and runtime wiring.

use crate::config::{ConfigError, NodeConfig};
use crate::content_storage::{ContentStorageError, FileContentAdapter};
use crate::did_registry::{DidRegistryError, FileDidRegistrationChainAdapter};
use crate::durable_guard_store::{
    DurableGuardBundleSnapshotStore, DurableGuardSnapshotStoreError, FileDurableGuardSnapshotStore,
};
use crate::migrations::{MigrationPlan, MigrationRegistry};
use crate::namespaces::StateNamespaces;
use crate::runtime::{build_runtime_wiring, RuntimeWiring};
use crate::state::{AppStateSchema, StateVersion, APP_STATE_VERSION};
use crate::task_operations::{
    FileTaskOperationSnapshotStore, TaskOperationError, TaskOperationSnapshotStore,
    TaskOperationSnapshotStoreError,
};
use crate::token::{default_token_config, TokenConfig};
use std::fs;
use std::path::{Path, PathBuf};

const CONTENT_STORE_FILE_NAME: &str = "content-store.snapshot";
const DID_REGISTRY_STORE_FILE_NAME: &str = "did-chain-adapter.snapshot";
const TASK_OPERATION_STORE_FILE_NAME: &str = "task-operation.snapshot";
const DURABLE_GUARD_STORE_FILE_NAME: &str = "durable-guard.snapshot";

const CONTENT_STORE_COMPONENT: &str = "content-storage:file-default";
const DID_REGISTRY_STORE_COMPONENT: &str = "did-registry:file-default";
const TASK_OPERATION_STORE_COMPONENT: &str = "task-operation-snapshot-store:file-default";
const DURABLE_GUARD_STORE_COMPONENT: &str = "durable-guard-snapshot-store:file-default";
const DID_REGISTRY_BOOTSTRAP_PROVIDER: &str = "bootstrap-runtime-compatibility";

#[derive(Debug, Clone, PartialEq, Eq)]
struct RuntimePersistenceLayout {
    storage_root: PathBuf,
    content_store_path: PathBuf,
    did_registry_store_path: PathBuf,
    task_operation_store_path: PathBuf,
    durable_guard_store_path: PathBuf,
}

/// Deterministic bootstrap artifact bundling validated config, schema, and wiring.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BootstrapPlan {
    /// Validated node configuration used to build the plan.
    pub config: NodeConfig,
    /// Canonical state namespaces available to runtime services.
    pub namespaces: StateNamespaces,
    /// State schema version and namespace metadata.
    pub state_schema: AppStateSchema,
    /// Token configuration validated during bootstrap.
    pub token_config: TokenConfig,
    /// Ordered migration plan from persisted to target state version.
    pub migration_plan: MigrationPlan,
    /// Runtime wiring map for service/component initialization.
    pub wiring: RuntimeWiring,
}

/// Builds a bootstrap plan for the current application state version.
pub fn bootstrap(config: NodeConfig) -> Result<BootstrapPlan, ConfigError> {
    bootstrap_from_state_version(config, APP_STATE_VERSION)
}

/// Builds a bootstrap plan from an explicit persisted state version.
pub fn bootstrap_from_state_version(
    config: NodeConfig,
    persisted_state_version: StateVersion,
) -> Result<BootstrapPlan, ConfigError> {
    config.validate()?;

    let state_schema = AppStateSchema::default();
    let target_state_version = state_schema.version;

    let registry = MigrationRegistry::new();
    let migration_plan = registry
        .build_plan(persisted_state_version, target_state_version)
        .map_err(|error| ConfigError::MigrationPlan(error.to_string()))?;

    let namespaces = state_schema.namespaces.clone();
    let token_config = default_token_config();
    token_config
        .validate()
        .map_err(|error| ConfigError::TokenModel(error.to_string()))?;
    let persistence_layout = resolve_runtime_persistence_layout(config.storage_dir.as_str());
    validate_runtime_persistence_layout(&persistence_layout)?;

    let mut wiring = build_runtime_wiring(&config);
    for component in prioritized_runtime_store_components() {
        if !wiring.common_components.contains(&component) {
            wiring.common_components.push(component);
        }
    }

    Ok(BootstrapPlan {
        config,
        namespaces,
        state_schema,
        token_config,
        migration_plan,
        wiring,
    })
}

fn resolve_runtime_persistence_layout(storage_dir: &str) -> RuntimePersistenceLayout {
    let storage_root = PathBuf::from(storage_dir);
    RuntimePersistenceLayout {
        content_store_path: storage_root.join(CONTENT_STORE_FILE_NAME),
        did_registry_store_path: storage_root.join(DID_REGISTRY_STORE_FILE_NAME),
        task_operation_store_path: storage_root.join(TASK_OPERATION_STORE_FILE_NAME),
        durable_guard_store_path: storage_root.join(DURABLE_GUARD_STORE_FILE_NAME),
        storage_root,
    }
}

fn prioritized_runtime_store_components() -> [&'static str; 4] {
    [
        CONTENT_STORE_COMPONENT,
        DID_REGISTRY_STORE_COMPONENT,
        TASK_OPERATION_STORE_COMPONENT,
        DURABLE_GUARD_STORE_COMPONENT,
    ]
}

fn validate_runtime_persistence_layout(
    layout: &RuntimePersistenceLayout,
) -> Result<(), ConfigError> {
    fs::create_dir_all(&layout.storage_root).map_err(|error| {
        ConfigError::RuntimeStoreCompatibility {
            store: "runtime-storage-root",
            reason_code: "runtime_storage_root_unavailable",
            detail: error.to_string(),
        }
    })?;
    validate_content_store_path(&layout.content_store_path)?;
    validate_did_registry_store_path(&layout.did_registry_store_path)?;
    validate_task_operation_store_path(&layout.task_operation_store_path)?;
    validate_durable_guard_store_path(&layout.durable_guard_store_path)?;
    Ok(())
}

fn validate_content_store_path(path: &Path) -> Result<(), ConfigError> {
    FileContentAdapter::new(path.to_path_buf())
        .map(|_| ())
        .map_err(map_content_store_validation_error)
}

fn validate_did_registry_store_path(path: &Path) -> Result<(), ConfigError> {
    FileDidRegistrationChainAdapter::new(path.to_path_buf(), DID_REGISTRY_BOOTSTRAP_PROVIDER)
        .map(|_| ())
        .map_err(map_did_registry_store_validation_error)
}

fn validate_task_operation_store_path(path: &Path) -> Result<(), ConfigError> {
    let store = FileTaskOperationSnapshotStore::new(path.to_path_buf())
        .map_err(map_task_operation_store_validation_error)?;
    store
        .read_latest()
        .map(|_| ())
        .map_err(map_task_operation_store_validation_error)
}

fn validate_durable_guard_store_path(path: &Path) -> Result<(), ConfigError> {
    let store = FileDurableGuardSnapshotStore::new(path.to_path_buf())
        .map_err(map_durable_guard_store_validation_error)?;
    store
        .load_bundle()
        .map(|_| ())
        .map_err(map_durable_guard_store_validation_error)
}

fn map_content_store_validation_error(error: ContentStorageError) -> ConfigError {
    match error {
        ContentStorageError::InvalidPayload(detail) => ConfigError::RuntimeStoreCorruptPayload {
            store: "content-storage",
            reason_code: "content_storage_corrupt_payload_rejected",
            detail,
        },
        ContentStorageError::Io(detail) => ConfigError::RuntimeStoreCompatibility {
            store: "content-storage",
            reason_code: "content_storage_io_error",
            detail,
        },
        other => ConfigError::RuntimeStoreCompatibility {
            store: "content-storage",
            reason_code: "content_storage_compatibility_failed",
            detail: other.to_string(),
        },
    }
}

fn map_did_registry_store_validation_error(error: DidRegistryError) -> ConfigError {
    match error {
        DidRegistryError::PersistenceInvalidPayload(detail) => {
            ConfigError::RuntimeStoreCorruptPayload {
                store: "did-registry",
                reason_code: "did_registry_corrupt_payload_rejected",
                detail,
            }
        }
        DidRegistryError::PersistenceIo(detail) => ConfigError::RuntimeStoreCompatibility {
            store: "did-registry",
            reason_code: "did_registry_io_error",
            detail,
        },
        other => ConfigError::RuntimeStoreCompatibility {
            store: "did-registry",
            reason_code: "did_registry_compatibility_failed",
            detail: other.to_string(),
        },
    }
}

fn map_task_operation_store_validation_error(
    error: TaskOperationSnapshotStoreError,
) -> ConfigError {
    match error {
        TaskOperationSnapshotStoreError::InvalidPayload(detail) => {
            ConfigError::RuntimeStoreCorruptPayload {
                store: "task-operation-snapshot-store",
                reason_code: "task_operation_snapshot_corrupt_payload_rejected",
                detail,
            }
        }
        TaskOperationSnapshotStoreError::Io(detail) => ConfigError::RuntimeStoreCompatibility {
            store: "task-operation-snapshot-store",
            reason_code: "task_operation_snapshot_io_error",
            detail,
        },
        TaskOperationSnapshotStoreError::Snapshot(
            TaskOperationError::SnapshotVersionMismatch { expected, found },
        ) => ConfigError::RuntimeStoreSchemaIncompatible {
            store: "task-operation-snapshot-store",
            reason_code: "task_operation_snapshot_schema_mismatch_rejected",
            expected: expected.to_string(),
            found: found.to_string(),
        },
        TaskOperationSnapshotStoreError::Snapshot(other) => {
            ConfigError::RuntimeStoreCompatibility {
                store: "task-operation-snapshot-store",
                reason_code: "task_operation_snapshot_restore_validation_failed",
                detail: other.to_string(),
            }
        }
    }
}

fn map_durable_guard_store_validation_error(error: DurableGuardSnapshotStoreError) -> ConfigError {
    match error {
        DurableGuardSnapshotStoreError::BundleSchemaVersionMismatch { expected, found } => {
            ConfigError::RuntimeStoreSchemaIncompatible {
                store: "durable-guard-snapshot-store",
                reason_code: "durable_guard_snapshot_schema_mismatch_rejected",
                expected: expected.to_string(),
                found: found.to_string(),
            }
        }
        DurableGuardSnapshotStoreError::InvalidPayload(detail) => {
            ConfigError::RuntimeStoreCorruptPayload {
                store: "durable-guard-snapshot-store",
                reason_code: "durable_guard_snapshot_corrupt_payload_rejected",
                detail,
            }
        }
        DurableGuardSnapshotStoreError::Io(detail) => ConfigError::RuntimeStoreCompatibility {
            store: "durable-guard-snapshot-store",
            reason_code: "durable_guard_snapshot_io_error",
            detail,
        },
        DurableGuardSnapshotStoreError::DeliverySnapshot(other) => {
            ConfigError::RuntimeStoreCompatibility {
                store: "durable-guard-snapshot-store",
                reason_code: "durable_guard_snapshot_restore_validation_failed",
                detail: other.to_string(),
            }
        }
        DurableGuardSnapshotStoreError::ChannelPolicySnapshot(other) => {
            ConfigError::RuntimeStoreCompatibility {
                store: "durable-guard-snapshot-store",
                reason_code: "durable_guard_snapshot_restore_validation_failed",
                detail: other.to_string(),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{bootstrap, bootstrap_from_state_version};
    use crate::config::{ConfigError, NodeConfig, NodeRole, SyncMode};
    use crate::state::{StateVersion, APP_STATE_VERSION};
    use crate::task_operations::{
        FileTaskOperationSnapshotStore, TaskOperationSnapshot, TaskOperationSnapshotStore,
    };
    use crate::token::DEFAULT_TOKEN_SYMBOL;
    use std::fs;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    const CONTENT_STORE_FIXTURE: &str = "content-store.snapshot";
    const TASK_OPERATION_STORE_FIXTURE: &str = "task-operation.snapshot";

    #[test]
    fn bootstrap_plan_builds_for_valid_config() {
        let config = NodeConfig {
            chain_id: "kamn-devnet".to_owned(),
            chain_version: "v0.1.0".to_owned(),
            role: NodeRole::Processor,
            storage_dir: "/tmp/kamn".to_owned(),
            enable_gossip: true,
            sync_mode: SyncMode::Fast,
        };

        let plan = bootstrap(config).expect("bootstrap should succeed");
        assert!(plan.namespaces.all_unique());
        assert!(plan.wiring.all_components().contains(&"block-producer"));
        assert_eq!(plan.state_schema.version, APP_STATE_VERSION);
        assert_eq!(plan.token_config.symbol, DEFAULT_TOKEN_SYMBOL);
        assert!(plan.token_config.validate().is_ok());
        assert!(plan.migration_plan.steps.is_empty());
    }

    #[test]
    fn bootstrap_fails_for_invalid_config() {
        let config = NodeConfig {
            chain_id: "".to_owned(),
            chain_version: "v0.1.0".to_owned(),
            role: NodeRole::Processor,
            storage_dir: "/tmp/kamn".to_owned(),
            enable_gossip: true,
            sync_mode: SyncMode::Fast,
        };

        assert_eq!(bootstrap(config), Err(ConfigError::EmptyChainId));
    }

    #[test]
    fn bootstrap_rejects_state_downgrade() {
        let config = NodeConfig {
            chain_id: "kamn-devnet".to_owned(),
            chain_version: "v0.1.0".to_owned(),
            role: NodeRole::Processor,
            storage_dir: "/tmp/kamn".to_owned(),
            enable_gossip: true,
            sync_mode: SyncMode::Fast,
        };

        let result = bootstrap_from_state_version(config, StateVersion(APP_STATE_VERSION.0 + 1));
        assert!(matches!(result, Err(ConfigError::MigrationPlan(_))));
    }

    #[test]
    fn bootstrap_wiring_includes_durable_store_components() {
        let config = NodeConfig {
            chain_id: "kamn-devnet".to_owned(),
            chain_version: "v0.1.0".to_owned(),
            role: NodeRole::Processor,
            storage_dir: temp_storage_dir("durable-components")
                .to_string_lossy()
                .into_owned(),
            enable_gossip: true,
            sync_mode: SyncMode::Fast,
        };

        let plan = bootstrap(config).expect("bootstrap should succeed");
        let components = plan.wiring.all_components();
        assert!(components.contains(&"content-storage:file-default"));
        assert!(components.contains(&"did-registry:file-default"));
        assert!(components.contains(&"task-operation-snapshot-store:file-default"));
        assert!(components.contains(&"durable-guard-snapshot-store:file-default"));
    }

    #[test]
    fn regression_bootstrap_fails_closed_when_content_store_payload_is_corrupt() {
        let storage_dir = temp_storage_dir("corrupt-content-store");
        fs::create_dir_all(&storage_dir).expect("fixture directory should build");
        fs::write(
            storage_dir.join(CONTENT_STORE_FIXTURE),
            "schema|kamn.content.file-store.v1\nobject|broken\n",
        )
        .expect("fixture should write");

        let config = NodeConfig {
            chain_id: "kamn-devnet".to_owned(),
            chain_version: "v0.1.0".to_owned(),
            role: NodeRole::Processor,
            storage_dir: storage_dir.to_string_lossy().into_owned(),
            enable_gossip: true,
            sync_mode: SyncMode::Fast,
        };

        let result = bootstrap(config);
        assert!(
            matches!(
                result,
                Err(ConfigError::RuntimeStoreCorruptPayload {
                    store,
                    reason_code,
                    ..
                }) if store == "content-storage" && reason_code == "content_storage_corrupt_payload_rejected"
            ),
            "corrupt content-store payload must fail closed with deterministic reason code"
        );
    }

    #[test]
    fn regression_bootstrap_fails_closed_when_task_snapshot_schema_is_incompatible() {
        let storage_dir = temp_storage_dir("incompatible-task-snapshot");
        fs::create_dir_all(&storage_dir).expect("fixture directory should build");
        let path = storage_dir.join(TASK_OPERATION_STORE_FIXTURE);
        let mut store = FileTaskOperationSnapshotStore::new(path.clone()).expect("store");
        store
            .write(TaskOperationSnapshot {
                schema_version: 1,
                tasks: vec![],
            })
            .expect("fixture snapshot should persist");
        fs::write(path, "schema|99\n").expect("fixture mutation should write");

        let config = NodeConfig {
            chain_id: "kamn-devnet".to_owned(),
            chain_version: "v0.1.0".to_owned(),
            role: NodeRole::Processor,
            storage_dir: storage_dir.to_string_lossy().into_owned(),
            enable_gossip: true,
            sync_mode: SyncMode::Fast,
        };

        let result = bootstrap(config);
        assert!(
            matches!(
                result,
                Err(ConfigError::RuntimeStoreSchemaIncompatible {
                    store,
                    reason_code,
                    expected,
                    found,
                }) if store == "task-operation-snapshot-store"
                    && reason_code == "task_operation_snapshot_schema_mismatch_rejected"
                    && expected == "1"
                    && found == "99"
            ),
            "incompatible task snapshot schema must fail closed with deterministic reason code"
        );
    }

    fn temp_storage_dir(tag: &str) -> PathBuf {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time should be monotonic")
            .as_nanos();
        std::env::temp_dir().join(format!(
            "kamn-bootstrap-{tag}-{}-{nonce}",
            std::process::id()
        ))
    }
}
