//! Bootstrap planning for validated config, schema migrations, and runtime wiring.

use crate::config::{ConfigError, NodeConfig};
use crate::migrations::{MigrationPlan, MigrationRegistry};
use crate::namespaces::StateNamespaces;
use crate::runtime::{build_runtime_wiring, RuntimeWiring};
use crate::state::{AppStateSchema, StateVersion, APP_STATE_VERSION};
use crate::token::{default_token_config, TokenConfig};

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
    let wiring = build_runtime_wiring(&config);

    Ok(BootstrapPlan {
        config,
        namespaces,
        state_schema,
        token_config,
        migration_plan,
        wiring,
    })
}

#[cfg(test)]
mod tests {
    use super::{bootstrap, bootstrap_from_state_version};
    use crate::config::{ConfigError, NodeConfig, NodeRole, SyncMode};
    use crate::state::{StateVersion, APP_STATE_VERSION};
    use crate::token::DEFAULT_TOKEN_SYMBOL;

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
}
