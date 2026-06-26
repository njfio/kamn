use super::layout::{prioritized_runtime_store_components, resolve_runtime_persistence_layout};
use super::models::{maybe_profile, BootstrapPlan};
use super::validation::validate_runtime_persistence_layout;
use crate::config::{ConfigError, NodeConfig};
use crate::migrations::MigrationRegistry;
use crate::runtime::{
    build_runtime_wiring, build_runtime_wiring_with_transport_profile, RuntimeTransportProfile,
};
use crate::state::{AppStateSchema, StateVersion, APP_STATE_VERSION};
use crate::token::default_token_config;

/// Builds a bootstrap plan for the current application state version.
pub fn bootstrap(config: NodeConfig) -> Result<BootstrapPlan, ConfigError> {
    bootstrap_from_state_version(config, APP_STATE_VERSION)
}

/// Builds a bootstrap plan with explicit runtime transport profile selection.
pub fn bootstrap_with_transport_profile(
    config: NodeConfig,
    transport_profile: RuntimeTransportProfile,
) -> Result<BootstrapPlan, ConfigError> {
    bootstrap_from_state_version_with_transport_profile(
        config,
        APP_STATE_VERSION,
        Some(transport_profile),
    )
}

/// Builds a bootstrap plan from an explicit persisted state version.
pub fn bootstrap_from_state_version(
    config: NodeConfig,
    persisted_state_version: StateVersion,
) -> Result<BootstrapPlan, ConfigError> {
    bootstrap_from_state_version_with_transport_profile(config, persisted_state_version, None)
}

fn bootstrap_from_state_version_with_transport_profile(
    config: NodeConfig,
    persisted_state_version: StateVersion,
    transport_profile: Option<RuntimeTransportProfile>,
) -> Result<BootstrapPlan, ConfigError> {
    config.validate()?;
    let state_schema = AppStateSchema::default();
    let migration_plan = build_migration_plan(persisted_state_version, state_schema.version)?;
    let token_config = validated_token_config()?;
    let persistence_layout = resolve_runtime_persistence_layout(config.storage_dir.as_str())?;
    validate_runtime_persistence_layout(&persistence_layout)?;
    let wiring = build_wiring(&config, transport_profile, &persistence_layout);
    Ok(BootstrapPlan {
        config,
        namespaces: state_schema.namespaces.clone(),
        state_schema,
        token_config,
        migration_plan,
        wiring,
    })
}

fn build_migration_plan(
    persisted_state_version: StateVersion,
    target_state_version: StateVersion,
) -> Result<crate::migrations::MigrationPlan, ConfigError> {
    MigrationRegistry::new()
        .build_plan(persisted_state_version, target_state_version)
        .map_err(|error| ConfigError::MigrationPlan(error.to_string()))
}

fn validated_token_config() -> Result<crate::token::TokenConfig, ConfigError> {
    let token_config = default_token_config();
    token_config
        .validate()
        .map_err(|error| ConfigError::TokenModel(error.to_string()))?;
    Ok(token_config)
}

fn build_wiring(
    config: &NodeConfig,
    transport_profile: Option<RuntimeTransportProfile>,
    persistence_layout: &super::models::RuntimePersistenceLayout,
) -> crate::runtime::RuntimeWiring {
    let mut wiring = match maybe_profile(transport_profile) {
        Some(profile) => build_runtime_wiring_with_transport_profile(config, profile),
        None => build_runtime_wiring(config),
    };
    append_runtime_store_components(&mut wiring, persistence_layout);
    wiring
}

fn append_runtime_store_components(
    wiring: &mut crate::runtime::RuntimeWiring,
    persistence_layout: &super::models::RuntimePersistenceLayout,
) {
    for component in prioritized_runtime_store_components(&persistence_layout.runtime_store_adapter)
    {
        if !wiring.common_components.contains(&component) {
            wiring.common_components.push(component);
        }
    }
}
