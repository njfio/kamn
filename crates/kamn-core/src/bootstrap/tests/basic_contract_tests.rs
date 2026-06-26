use super::support::temp_storage_dir;
use crate::bootstrap::{bootstrap, bootstrap_from_state_version};
use crate::config::{ConfigError, NodeConfig, NodeRole, SyncMode};
use crate::state::{StateVersion, APP_STATE_VERSION};
use crate::token::DEFAULT_TOKEN_SYMBOL;

#[test]
fn bootstrap_plan_builds_for_valid_config() {
    let plan = bootstrap(valid_config("/tmp/kamn")).expect("bootstrap should succeed");
    assert!(plan.namespaces.all_unique());
    assert!(plan.wiring.all_components().contains(&"block-producer"));
    assert_eq!(plan.state_schema.version, APP_STATE_VERSION);
    assert_eq!(plan.token_config.symbol, DEFAULT_TOKEN_SYMBOL);
    assert!(plan.token_config.validate().is_ok());
    assert!(plan.migration_plan.steps.is_empty());
}

#[test]
fn bootstrap_fails_for_invalid_config() {
    let mut config = valid_config("/tmp/kamn");
    config.chain_id.clear();
    assert_eq!(bootstrap(config), Err(ConfigError::EmptyChainId));
}

#[test]
fn bootstrap_rejects_state_downgrade() {
    let result = bootstrap_from_state_version(
        valid_config("/tmp/kamn"),
        StateVersion(APP_STATE_VERSION.0 + 1),
    );
    assert!(matches!(result, Err(ConfigError::MigrationPlan(_))));
}

#[test]
fn bootstrap_wiring_includes_durable_store_components() {
    let storage_dir = temp_storage_dir("durable-components");
    let plan =
        bootstrap(valid_config(&storage_dir.to_string_lossy())).expect("bootstrap should succeed");
    let components = plan.wiring.all_components();
    assert!(components.contains(&"content-storage:file-default"));
    assert!(components.contains(&"did-registry:file-default"));
    assert!(components.contains(&"task-operation-snapshot-store:file-default"));
    assert!(components.contains(&"durable-guard-snapshot-store:file-default"));
    assert!(components.contains(&"channel-snapshot-store:file-default"));
    assert!(components.contains(&"message-lifecycle-snapshot-store:file-default"));
    assert!(components.contains(&"runtime-snapshot-store:file-default"));
}

fn valid_config(storage_dir: &str) -> NodeConfig {
    NodeConfig {
        chain_id: "kamn-devnet".to_owned(),
        chain_version: "v0.1.0".to_owned(),
        role: NodeRole::Processor,
        storage_dir: storage_dir.to_owned(),
        enable_gossip: true,
        sync_mode: SyncMode::Fast,
    }
}
