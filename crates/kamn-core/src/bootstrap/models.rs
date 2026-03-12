use crate::config::NodeConfig;
use crate::migrations::MigrationPlan;
use crate::namespaces::StateNamespaces;
use crate::runtime::{RuntimeTransportProfile, RuntimeWiring};
use crate::state::AppStateSchema;
use crate::token::TokenConfig;
use std::path::PathBuf;

pub const CONTENT_STORE_FILE_NAME: &str = "content-store.snapshot";
pub const DID_REGISTRY_STORE_FILE_NAME: &str = "did-chain-adapter.snapshot";
pub const TASK_OPERATION_STORE_FILE_NAME: &str = "task-operation.snapshot";
pub const DURABLE_GUARD_STORE_FILE_NAME: &str = "durable-guard.snapshot";
pub const CHANNEL_SNAPSHOT_STORE_FILE_NAME: &str = "channel.snapshot";
pub const MESSAGE_LIFECYCLE_SNAPSHOT_STORE_FILE_NAME: &str = "message-lifecycle.snapshot";
pub const RUNTIME_SNAPSHOT_STORE_FILE_NAME: &str = "runtime.snapshot";
pub const SQLITE_STORAGE_SELECTOR_PREFIX: &str = "sqlite://";
pub const DID_REGISTRY_BOOTSTRAP_PROVIDER: &str = "bootstrap-runtime-compatibility";

pub const CONTENT_STORE_COMPONENT: &str = "content-storage:file-default";
pub const DID_REGISTRY_STORE_COMPONENT: &str = "did-registry:file-default";
pub const TASK_OPERATION_STORE_COMPONENT_FILE: &str = "task-operation-snapshot-store:file-default";
pub const DURABLE_GUARD_STORE_COMPONENT_FILE: &str = "durable-guard-snapshot-store:file-default";
pub const CHANNEL_SNAPSHOT_STORE_COMPONENT_FILE: &str = "channel-snapshot-store:file-default";
pub const MESSAGE_LIFECYCLE_SNAPSHOT_STORE_COMPONENT_FILE: &str =
    "message-lifecycle-snapshot-store:file-default";
pub const RUNTIME_SNAPSHOT_STORE_COMPONENT_FILE: &str = "runtime-snapshot-store:file-default";
pub const TASK_OPERATION_STORE_COMPONENT_SQLITE: &str =
    "task-operation-snapshot-store:sqlite-default";
pub const DURABLE_GUARD_STORE_COMPONENT_SQLITE: &str = "durable-guard-snapshot-store:sqlite-default";
pub const CHANNEL_SNAPSHOT_STORE_COMPONENT_SQLITE: &str = "channel-snapshot-store:sqlite-default";
pub const MESSAGE_LIFECYCLE_SNAPSHOT_STORE_COMPONENT_SQLITE: &str =
    "message-lifecycle-snapshot-store:sqlite-default";
pub const RUNTIME_SNAPSHOT_STORE_COMPONENT_SQLITE: &str = "runtime-snapshot-store:sqlite-default";

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RuntimeStoreAdapter {
    File,
    Sqlite { database_path: PathBuf },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimePersistenceLayout {
    pub storage_root: PathBuf,
    pub content_store_path: PathBuf,
    pub did_registry_store_path: PathBuf,
    pub task_operation_store_path: PathBuf,
    pub durable_guard_store_path: PathBuf,
    pub channel_snapshot_store_path: PathBuf,
    pub message_lifecycle_snapshot_store_path: PathBuf,
    pub runtime_snapshot_store_path: PathBuf,
    pub runtime_store_adapter: RuntimeStoreAdapter,
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

pub fn maybe_profile(
    profile: Option<RuntimeTransportProfile>,
) -> Option<RuntimeTransportProfile> {
    profile
}
