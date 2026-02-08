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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SyncMode {
    Fast,
    Slow,
    Archive,
}

impl SyncMode {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Fast => "fast",
            Self::Slow => "slow",
            Self::Archive => "archive",
        }
    }

    pub fn profile(self) -> SyncOperationalProfile {
        match self {
            Self::Fast => SyncOperationalProfile {
                mode: self,
                startup_strategy: SyncStartupStrategy::StateSyncToLatest,
                recovery_strategy: SyncRecoveryStrategy::ResumeRecentState,
                requires_chain_version_match: false,
                maintain_full_history: false,
            },
            Self::Slow => SyncOperationalProfile {
                mode: self,
                startup_strategy: SyncStartupStrategy::BlockReplayFromGenesis,
                recovery_strategy: SyncRecoveryStrategy::ReplayMissingBlocks,
                requires_chain_version_match: true,
                maintain_full_history: false,
            },
            Self::Archive => SyncOperationalProfile {
                mode: self,
                startup_strategy: SyncStartupStrategy::ArchiveStateSyncFromGenesis,
                recovery_strategy: SyncRecoveryStrategy::ReplayArchivedHistory,
                requires_chain_version_match: false,
                maintain_full_history: true,
            },
        }
    }
}

impl FromStr for SyncMode {
    type Err = ConfigError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "fast" => Ok(Self::Fast),
            "slow" => Ok(Self::Slow),
            "archive" => Ok(Self::Archive),
            other => Err(ConfigError::InvalidSyncMode(other.to_owned())),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SyncStartupStrategy {
    StateSyncToLatest,
    BlockReplayFromGenesis,
    ArchiveStateSyncFromGenesis,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SyncRecoveryStrategy {
    ResumeRecentState,
    ReplayMissingBlocks,
    ReplayArchivedHistory,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SyncOperationalProfile {
    pub mode: SyncMode,
    pub startup_strategy: SyncStartupStrategy,
    pub recovery_strategy: SyncRecoveryStrategy,
    pub requires_chain_version_match: bool,
    pub maintain_full_history: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NodeConfig {
    pub chain_id: String,
    pub chain_version: String,
    pub role: NodeRole,
    pub storage_dir: String,
    pub enable_gossip: bool,
    pub sync_mode: SyncMode,
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

    pub fn operational_profile(&self) -> SyncOperationalProfile {
        self.sync_mode.profile()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConfigError {
    EmptyChainId,
    EmptyChainVersion,
    EmptyStorageDir,
    MigrationPlan(String),
    TokenModel(String),
    InvalidRole(String),
    InvalidSyncMode(String),
    InvalidOutputMode(String),
    InvalidNodeProfile(String),
    InvalidDiagnosticsMode(String),
    InvalidRuntimeMode(String),
    InvalidProposalArgument(String),
    RuntimePlanner(String),
    UnknownArgument(String),
    MissingArgumentValue(&'static str),
}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyChainId => write!(f, "chain_id must not be empty"),
            Self::EmptyChainVersion => write!(f, "chain_version must not be empty"),
            Self::EmptyStorageDir => write!(f, "storage_dir must not be empty"),
            Self::MigrationPlan(message) => write!(f, "migration planning failed: {message}"),
            Self::TokenModel(message) => write!(f, "token model validation failed: {message}"),
            Self::InvalidRole(value) => write!(f, "invalid role: {value}"),
            Self::InvalidSyncMode(value) => write!(f, "invalid sync mode: {value}"),
            Self::InvalidOutputMode(value) => write!(f, "invalid output mode: {value}"),
            Self::InvalidNodeProfile(value) => write!(f, "invalid node profile: {value}"),
            Self::InvalidDiagnosticsMode(value) => {
                write!(f, "invalid diagnostics mode: {value}")
            }
            Self::InvalidRuntimeMode(value) => write!(f, "invalid runtime mode: {value}"),
            Self::InvalidProposalArgument(value) => {
                write!(f, "invalid proposal argument: {value}")
            }
            Self::RuntimePlanner(message) => {
                write!(f, "runtime planner validation failed: {message}")
            }
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
    use super::{
        ConfigError, NodeConfig, NodeRole, SyncMode, SyncRecoveryStrategy, SyncStartupStrategy,
    };

    #[test]
    fn parses_node_roles() {
        assert_eq!("processor".parse::<NodeRole>(), Ok(NodeRole::Processor));
        assert_eq!("listener".parse::<NodeRole>(), Ok(NodeRole::Listener));
        assert_eq!("approver".parse::<NodeRole>(), Ok(NodeRole::Approver));
    }

    #[test]
    fn parses_sync_modes() {
        assert_eq!("fast".parse::<SyncMode>(), Ok(SyncMode::Fast));
        assert_eq!("slow".parse::<SyncMode>(), Ok(SyncMode::Slow));
        assert_eq!("archive".parse::<SyncMode>(), Ok(SyncMode::Archive));
    }

    #[test]
    fn rejects_invalid_role() {
        assert_eq!(
            "invalid".parse::<NodeRole>(),
            Err(ConfigError::InvalidRole("invalid".to_owned()))
        );
    }

    #[test]
    fn rejects_invalid_sync_mode() {
        assert_eq!(
            "turbo".parse::<SyncMode>(),
            Err(ConfigError::InvalidSyncMode("turbo".to_owned()))
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
            sync_mode: SyncMode::Fast,
        };

        assert!(config.validate().is_ok());
    }

    #[test]
    fn operational_profile_maps_archive_recovery() {
        let config = NodeConfig {
            chain_id: "kamn-devnet".to_owned(),
            chain_version: "v0.1.0".to_owned(),
            role: NodeRole::Processor,
            storage_dir: "/tmp/kamn".to_owned(),
            enable_gossip: true,
            sync_mode: SyncMode::Archive,
        };

        let profile = config.operational_profile();
        assert_eq!(profile.mode, SyncMode::Archive);
        assert_eq!(
            profile.startup_strategy,
            SyncStartupStrategy::ArchiveStateSyncFromGenesis
        );
        assert_eq!(
            profile.recovery_strategy,
            SyncRecoveryStrategy::ReplayArchivedHistory
        );
        assert!(profile.maintain_full_history);
    }

    #[test]
    fn rejects_empty_chain_id() {
        let config = NodeConfig {
            chain_id: "".to_owned(),
            chain_version: "v0.1.0".to_owned(),
            role: NodeRole::Processor,
            storage_dir: "/tmp/kamn".to_owned(),
            enable_gossip: true,
            sync_mode: SyncMode::Fast,
        };

        assert_eq!(config.validate(), Err(ConfigError::EmptyChainId));
    }
}
