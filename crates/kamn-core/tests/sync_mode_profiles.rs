use kamn_core::{
    ConfigError, NodeConfig, NodeRole, SyncMode, SyncOperationalProfile, SyncRecoveryStrategy,
    SyncStartupStrategy,
};

#[test]
fn fast_mode_uses_state_sync_then_block_follow() {
    let profile = SyncMode::Fast.profile();
    assert_eq!(
        profile,
        SyncOperationalProfile {
            mode: SyncMode::Fast,
            startup_strategy: SyncStartupStrategy::StateSyncToLatest,
            recovery_strategy: SyncRecoveryStrategy::ResumeRecentState,
            requires_chain_version_match: false,
            maintain_full_history: false,
        }
    );
}

#[test]
fn slow_mode_prefers_block_replay_with_version_guard() {
    let profile = SyncMode::Slow.profile();
    assert_eq!(
        profile.startup_strategy,
        SyncStartupStrategy::BlockReplayFromGenesis
    );
    assert_eq!(
        profile.recovery_strategy,
        SyncRecoveryStrategy::ReplayMissingBlocks
    );
    assert!(profile.requires_chain_version_match);
    assert!(!profile.maintain_full_history);
}

#[test]
fn integration_node_config_exposes_archive_sync_profile() {
    let config = NodeConfig {
        chain_id: "kamn-devnet".to_owned(),
        chain_version: "v0.1.0".to_owned(),
        role: NodeRole::Processor,
        storage_dir: "./data".to_owned(),
        enable_gossip: true,
        sync_mode: SyncMode::Archive,
    };

    config.validate().expect("config should validate");

    let profile = config.operational_profile();
    assert_eq!(profile.mode, SyncMode::Archive);
    assert_eq!(
        profile.recovery_strategy,
        SyncRecoveryStrategy::ReplayArchivedHistory
    );
    assert!(profile.maintain_full_history);
}

#[test]
fn regression_invalid_sync_mode_string_is_rejected() {
    // Regression: #209
    assert_eq!(
        "turbo".parse::<SyncMode>(),
        Err(ConfigError::InvalidSyncMode("turbo".to_owned()))
    );
}
