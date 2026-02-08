use std::fmt;
use std::str::FromStr;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NodeRole {
    Processor,
    Listener,
    Approver,
}

impl NodeRole {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Processor => "processor",
            Self::Listener => "listener",
            Self::Approver => "approver",
        }
    }
}

impl FromStr for NodeRole {
    type Err = ConfigError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "processor" => Ok(Self::Processor),
            "listener" => Ok(Self::Listener),
            "approver" => Ok(Self::Approver),
            other => Err(ConfigError::InvalidRole(other.to_owned())),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NodeConfig {
    pub chain_id: String,
    pub chain_version: String,
    pub role: NodeRole,
    pub storage_dir: String,
    pub enable_gossip: bool,
}

impl NodeConfig {
    pub fn validate(&self) -> Result<(), ConfigError> {
        if self.chain_id.trim().is_empty() {
            return Err(ConfigError::EmptyChainId);
        }
        if self.chain_version.trim().is_empty() {
            return Err(ConfigError::EmptyChainVersion);
        }
        if self.storage_dir.trim().is_empty() {
            return Err(ConfigError::EmptyStorageDir);
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConfigError {
    EmptyChainId,
    EmptyChainVersion,
    EmptyStorageDir,
    InvalidRole(String),
    UnknownArgument(String),
    MissingArgumentValue(&'static str),
}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyChainId => write!(f, "chain_id must not be empty"),
            Self::EmptyChainVersion => write!(f, "chain_version must not be empty"),
            Self::EmptyStorageDir => write!(f, "storage_dir must not be empty"),
            Self::InvalidRole(value) => write!(f, "invalid role: {value}"),
            Self::UnknownArgument(value) => write!(f, "unknown argument: {value}"),
            Self::MissingArgumentValue(flag) => {
                write!(f, "missing value for argument: {flag}")
            }
        }
    }
}

impl std::error::Error for ConfigError {}

#[cfg(test)]
mod tests {
    use super::{ConfigError, NodeConfig, NodeRole};

    #[test]
    fn parses_node_roles() {
        assert_eq!("processor".parse::<NodeRole>(), Ok(NodeRole::Processor));
        assert_eq!("listener".parse::<NodeRole>(), Ok(NodeRole::Listener));
        assert_eq!("approver".parse::<NodeRole>(), Ok(NodeRole::Approver));
    }

    #[test]
    fn rejects_invalid_role() {
        assert_eq!(
            "invalid".parse::<NodeRole>(),
            Err(ConfigError::InvalidRole("invalid".to_owned()))
        );
    }

    #[test]
    fn validates_config_fields() {
        let config = NodeConfig {
            chain_id: "kamn-devnet".to_owned(),
            chain_version: "v0.1.0".to_owned(),
            role: NodeRole::Processor,
            storage_dir: "/tmp/kamn".to_owned(),
            enable_gossip: true,
        };

        assert!(config.validate().is_ok());
    }

    #[test]
    fn rejects_empty_chain_id() {
        let config = NodeConfig {
            chain_id: "".to_owned(),
            chain_version: "v0.1.0".to_owned(),
            role: NodeRole::Processor,
            storage_dir: "/tmp/kamn".to_owned(),
            enable_gossip: true,
        };

        assert_eq!(config.validate(), Err(ConfigError::EmptyChainId));
    }
}
