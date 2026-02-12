//! Node configuration contracts for role and sync-mode policy surfaces.
//!
//! This module defines strongly typed role and synchronization profiles used by
//! runtime startup, recovery, and CLI/config validation paths.

use std::fmt;
use std::str::FromStr;

/// Node execution role determining primary runtime responsibilities.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NodeRole {
    /// Processes and executes protocol work units.
    Processor,
    /// Primarily listens and relays events without approval authority.
    Listener,
    /// Reviews and approves gated transitions or proposals.
    Approver,
}

impl NodeRole {
    /// Returns the canonical lowercase string form used in config parsing.
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

/// Chain synchronization mode used for startup and recovery planning.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SyncMode {
    /// Prioritize fast startup with minimal replay.
    Fast,
    /// Replay more history with stricter version matching.
    Slow,
    /// Maintain complete historical state and replay coverage.
    Archive,
}

impl SyncMode {
    /// Returns the canonical lowercase string form used in config parsing.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Fast => "fast",
            Self::Slow => "slow",
            Self::Archive => "archive",
        }
    }

    /// Resolves the operational startup/recovery profile for this sync mode.
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

/// Startup strategy chosen by sync-mode policy.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SyncStartupStrategy {
    /// Sync to latest state before serving traffic.
    StateSyncToLatest,
    /// Replay blocks from genesis to construct local state.
    BlockReplayFromGenesis,
    /// Perform archive-grade sync from genesis state.
    ArchiveStateSyncFromGenesis,
}

/// Recovery strategy chosen by sync-mode policy.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SyncRecoveryStrategy {
    /// Resume from recent persisted state with minimal replay.
    ResumeRecentState,
    /// Replay only missing block range after outage.
    ReplayMissingBlocks,
    /// Replay archived history to preserve full lineage.
    ReplayArchivedHistory,
}

/// Derived operational profile for a selected synchronization mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SyncOperationalProfile {
    /// Source sync mode that generated this profile.
    pub mode: SyncMode,
    /// Startup strategy contract for initialization.
    pub startup_strategy: SyncStartupStrategy,
    /// Recovery strategy contract after interruption.
    pub recovery_strategy: SyncRecoveryStrategy,
    /// Whether chain-version parity is required before joining.
    pub requires_chain_version_match: bool,
    /// Whether full historical state retention is required.
    pub maintain_full_history: bool,
}

/// Node runtime configuration envelope validated before startup.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NodeConfig {
    /// Logical chain identifier the node should join.
    pub chain_id: String,
    /// Expected chain runtime/schema version.
    pub chain_version: String,
    /// Node runtime role.
    pub role: NodeRole,
    /// Filesystem path used for durable node storage.
    pub storage_dir: String,
    /// Whether gossip transport is enabled.
    pub enable_gossip: bool,
    /// Synchronization mode used for startup/recovery.
    pub sync_mode: SyncMode,
}

impl NodeConfig {
    /// Validates required configuration fields for non-empty values.
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

    /// Returns the derived operational sync profile for this configuration.
    pub fn operational_profile(&self) -> SyncOperationalProfile {
        self.sync_mode.profile()
    }
}

/// Error surface for config parsing and runtime option validation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConfigError {
    /// `chain_id` was empty.
    EmptyChainId,
    /// `chain_version` was empty.
    EmptyChainVersion,
    /// `storage_dir` was empty.
    EmptyStorageDir,
    /// Migration planning argument validation failed.
    MigrationPlan(String),
    /// Token model argument validation failed.
    TokenModel(String),
    /// Unknown or invalid node role string.
    InvalidRole(String),
    /// Unknown or invalid sync mode string.
    InvalidSyncMode(String),
    /// Unknown or invalid output mode string.
    InvalidOutputMode(String),
    /// Unknown or invalid node profile string.
    InvalidNodeProfile(String),
    /// Unknown or invalid diagnostics mode string.
    InvalidDiagnosticsMode(String),
    /// Unknown or invalid runtime mode string.
    InvalidRuntimeMode(String),
    /// Invalid expected state version argument.
    InvalidExpectedStateVersion(String),
    /// Invalid daemon control argument.
    InvalidDaemonControlArgument(String),
    /// Invalid daemon lifecycle event argument.
    InvalidDaemonLifecycleEvent(String),
    /// Invalid governance proposal argument.
    InvalidProposalArgument(String),
    /// Invalid rejoin-attempt argument.
    InvalidRejoinAttemptArgument(String),
    /// Runtime planner argument validation failure.
    RuntimePlanner(String),
    /// Runtime recovery argument validation failure.
    RuntimeRecovery(String),
    /// Runtime daemon lifecycle argument validation failure.
    RuntimeDaemonLifecycle(String),
    /// Unknown command-line/config argument.
    UnknownArgument(String),
    /// Argument flag required a value but none was provided.
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
            Self::InvalidExpectedStateVersion(value) => {
                write!(f, "invalid expected state version: {value}")
            }
            Self::InvalidDaemonControlArgument(value) => {
                write!(f, "invalid daemon control argument: {value}")
            }
            Self::InvalidDaemonLifecycleEvent(value) => {
                write!(f, "invalid daemon lifecycle event: {value}")
            }
            Self::InvalidProposalArgument(value) => {
                write!(f, "invalid proposal argument: {value}")
            }
            Self::InvalidRejoinAttemptArgument(value) => {
                write!(f, "invalid rejoin attempt argument: {value}")
            }
            Self::RuntimePlanner(message) => {
                write!(f, "runtime planner validation failed: {message}")
            }
            Self::RuntimeRecovery(message) => {
                write!(f, "runtime recovery validation failed: {message}")
            }
            Self::RuntimeDaemonLifecycle(message) => {
                write!(f, "runtime daemon lifecycle validation failed: {message}")
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
