use crate::config::{ConfigError, NodeConfig};
use crate::namespaces::StateNamespaces;
use crate::runtime::{build_runtime_wiring, RuntimeWiring};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BootstrapPlan {
    pub config: NodeConfig,
    pub namespaces: StateNamespaces,
    pub wiring: RuntimeWiring,
}

pub fn bootstrap(config: NodeConfig) -> Result<BootstrapPlan, ConfigError> {
    config.validate()?;

    let namespaces = StateNamespaces::default();
    let wiring = build_runtime_wiring(&config);

    Ok(BootstrapPlan {
        config,
        namespaces,
        wiring,
    })
}

#[cfg(test)]
mod tests {
    use super::bootstrap;
    use crate::config::{ConfigError, NodeConfig, NodeRole};

    #[test]
    fn bootstrap_plan_builds_for_valid_config() {
        let config = NodeConfig {
            chain_id: "kamn-devnet".to_owned(),
            chain_version: "v0.1.0".to_owned(),
            role: NodeRole::Processor,
            storage_dir: "/tmp/kamn".to_owned(),
            enable_gossip: true,
        };

        let plan = bootstrap(config).expect("bootstrap should succeed");
        assert!(plan.namespaces.all_unique());
        assert!(plan.wiring.all_components().contains(&"block-producer"));
    }

    #[test]
    fn bootstrap_fails_for_invalid_config() {
        let config = NodeConfig {
            chain_id: "".to_owned(),
            chain_version: "v0.1.0".to_owned(),
            role: NodeRole::Processor,
            storage_dir: "/tmp/kamn".to_owned(),
            enable_gossip: true,
        };

        assert_eq!(bootstrap(config), Err(ConfigError::EmptyChainId));
    }
}
