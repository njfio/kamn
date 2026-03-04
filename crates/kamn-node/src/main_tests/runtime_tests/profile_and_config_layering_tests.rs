#[test]
fn parses_runtime_mode_kolme_live_with_required_flags() {
    let args = vec![
        "kamn-node".to_owned(),
        "--role".to_owned(),
        "processor".to_owned(),
        "--runtime-mode".to_owned(),
        "kolme-live".to_owned(),
        "--kolme-live-base-url".to_owned(),
        "http://127.0.0.1:3000".to_owned(),
        "--kolme-live-provider-hint".to_owned(),
        "kolme-fork-local".to_owned(),
        "--kolme-live-signing-profile".to_owned(),
        "kolme-fork-secp256k1-v1".to_owned(),
        "--kolme-live-signer-key-source".to_owned(),
        "env-local".to_owned(),
    ];

    let parsed = parse_args(args).expect("kolme-live args should parse");
    assert_eq!(parsed.runtime_mode.as_str(), "kolme-live");
    assert_eq!(
        parsed.kolme_live_base_url,
        Some("http://127.0.0.1:3000".to_owned())
    );
    assert_eq!(
        parsed.kolme_live_provider_hint,
        Some("kolme-fork-local".to_owned())
    );
    assert_eq!(
        parsed.kolme_live_signing_profile,
        Some("kolme-fork-secp256k1-v1".to_owned())
    );
    assert!(!parsed.kolme_live_strict_signer_contracts);
    assert_eq!(parsed.kolme_live_signer_profile, None);
    assert_eq!(
        parsed.kolme_live_signer_key_source,
        Some("env-local".to_owned())
    );
}

#[test]
fn parses_local_listener_profile_defaults() {
    let args = vec![
        "kamn-node".to_owned(),
        "--profile".to_owned(),
        "local-listener".to_owned(),
    ];

    let parsed = parse_args(args).expect("profile args should parse");
    assert_eq!(parsed.profile, Some(LocalProfile::Listener));
    assert_eq!(parsed.role, NodeRole::Listener);
    assert_eq!(parsed.chain_id, "kamn-localnet");
    assert_eq!(parsed.storage_dir, "./data/listener");
    assert_eq!(parsed.sync_mode, SyncMode::Fast);
    assert!(parsed.enable_gossip);
}

#[test]
fn profile_defaults_can_be_overridden_by_explicit_flags() {
    let args = vec![
        "kamn-node".to_owned(),
        "--profile".to_owned(),
        "local-listener".to_owned(),
        "--chain-id".to_owned(),
        "kamn-custom".to_owned(),
        "--storage-dir".to_owned(),
        "./tmp/custom-listener".to_owned(),
        "--disable-gossip".to_owned(),
        "--sync-mode".to_owned(),
        "archive".to_owned(),
    ];

    let parsed = parse_args(args).expect("profile args with overrides should parse");
    assert_eq!(parsed.profile, Some(LocalProfile::Listener));
    assert_eq!(parsed.role, NodeRole::Listener);
    assert_eq!(parsed.chain_id, "kamn-custom");
    assert_eq!(parsed.storage_dir, "./tmp/custom-listener");
    assert_eq!(parsed.sync_mode, SyncMode::Archive);
    assert!(!parsed.enable_gossip);
}

#[test]
fn parses_config_file_layer_for_core_node_fields() {
    let _env_lock = signer_env_lock()
        .lock()
        .unwrap_or_else(|error| error.into_inner());
    let _chain_id_guard = EnvVarGuard::set("KAMN_NODE_CHAIN_ID", None);
    let _sync_mode_guard = EnvVarGuard::set("KAMN_NODE_SYNC_MODE", None);
    let _log_level_guard = EnvVarGuard::set("KAMN_NODE_LOG_LEVEL", None);
    let _log_format_guard = EnvVarGuard::set("KAMN_NODE_LOG_FORMAT", None);
    let config_path = write_temp_node_config(
        "role=listener\nchain_id=kamn-config-file\nchain_version=v0.2.0\nstorage_dir=./tmp/config-listener\nenable_gossip=false\nsync_mode=archive\noutput=json\ndiagnostics=snapshot\n",
    );
    let args = vec![
        "kamn-node".to_owned(),
        "--config-file".to_owned(),
        config_path.to_string_lossy().to_string(),
    ];

    let parsed_result = parse_args(args);
    std::fs::remove_file(config_path).expect("temp config file should clean up");
    let parsed = parsed_result.expect("config file args should parse");

    assert_eq!(parsed.role, NodeRole::Listener);
    assert_eq!(parsed.chain_id, "kamn-config-file");
    assert_eq!(parsed.chain_version, "v0.2.0");
    assert_eq!(parsed.storage_dir, "./tmp/config-listener");
    assert_eq!(parsed.sync_mode, SyncMode::Archive);
    assert!(!parsed.enable_gossip);
    assert_eq!(parsed.output_mode, OutputMode::json());
    assert_eq!(parsed.diagnostics_mode, DiagnosticsMode::snapshot());
}

#[test]
fn env_overrides_config_file_chain_id_and_sync_mode() {
    let _env_lock = signer_env_lock()
        .lock()
        .unwrap_or_else(|error| error.into_inner());
    let _chain_id_guard = EnvVarGuard::set("KAMN_NODE_CHAIN_ID", Some("kamn-env"));
    let _sync_mode_guard = EnvVarGuard::set("KAMN_NODE_SYNC_MODE", Some("slow"));
    let config_path = write_temp_node_config(
        "role=listener\nchain_id=kamn-config-file\nsync_mode=archive\nstorage_dir=./tmp/config-listener\n",
    );
    let args = vec![
        "kamn-node".to_owned(),
        "--config-file".to_owned(),
        config_path.to_string_lossy().to_string(),
    ];

    let parsed_result = parse_args(args);
    std::fs::remove_file(config_path).expect("temp config file should clean up");
    let parsed = parsed_result.expect("config + env layered args should parse");

    assert_eq!(parsed.role, NodeRole::Listener);
    assert_eq!(parsed.chain_id, "kamn-env");
    assert_eq!(parsed.sync_mode, SyncMode::Slow);
}

#[test]
fn regression_6321_dual_cli_and_env_config_sources_fail_closed() {
    let _env_lock = signer_env_lock()
        .lock()
        .unwrap_or_else(|error| error.into_inner());
    let cli_config_path = write_temp_node_config("role=listener\nchain_id=kamn-cli-config\n");
    let env_config_path = write_temp_node_config("role=listener\nchain_id=kamn-env-config\n");
    let env_config_value = env_config_path.to_string_lossy().to_string();
    let _config_env_guard = EnvVarGuard::set("KAMN_NODE_CONFIG_FILE", Some(env_config_value.as_str()));
    let args = vec![
        "kamn-node".to_owned(),
        "--config-file".to_owned(),
        cli_config_path.to_string_lossy().to_string(),
    ];

    let parse_result = parse_args(args);
    std::fs::remove_file(cli_config_path).expect("cli temp config file should clean up");
    std::fs::remove_file(env_config_path).expect("env temp config file should clean up");

    assert_eq!(
        parse_result,
        Err(ConfigError::InvalidNodeConfig(
            "both --config-file and KAMN_NODE_CONFIG_FILE are set; declare one config source"
                .to_owned()
        ))
    );
}

#[test]
fn regression_6321_env_config_file_source_still_applies_when_cli_source_absent() {
    let _env_lock = signer_env_lock()
        .lock()
        .unwrap_or_else(|error| error.into_inner());
    let env_config_path = write_temp_node_config("role=listener\nchain_id=kamn-env-config\n");
    let env_config_value = env_config_path.to_string_lossy().to_string();
    let _config_env_guard = EnvVarGuard::set("KAMN_NODE_CONFIG_FILE", Some(env_config_value.as_str()));
    let args = vec!["kamn-node".to_owned()];

    let parsed_result = parse_args(args);
    std::fs::remove_file(env_config_path).expect("env temp config file should clean up");
    let parsed = parsed_result.expect("env config file source should parse");

    assert_eq!(parsed.role, NodeRole::Listener);
    assert_eq!(parsed.chain_id, "kamn-env-config");
}

#[test]
fn cli_values_override_env_and_config_layers() {
    let _env_lock = signer_env_lock()
        .lock()
        .unwrap_or_else(|error| error.into_inner());
    let _chain_id_guard = EnvVarGuard::set("KAMN_NODE_CHAIN_ID", Some("kamn-env"));
    let config_path = write_temp_node_config("role=listener\nchain_id=kamn-config-file\n");
    let args = vec![
        "kamn-node".to_owned(),
        "--config-file".to_owned(),
        config_path.to_string_lossy().to_string(),
        "--chain-id".to_owned(),
        "kamn-cli".to_owned(),
    ];

    let parsed_result = parse_args(args);
    std::fs::remove_file(config_path).expect("temp config file should clean up");
    let parsed = parsed_result.expect("config + env + cli layered args should parse");

    assert_eq!(parsed.role, NodeRole::Listener);
    assert_eq!(parsed.chain_id, "kamn-cli");
}

#[test]
fn regression_2967_invalid_env_override_fails_closed() {
    let _env_lock = signer_env_lock()
        .lock()
        .unwrap_or_else(|error| error.into_inner());
    let _sync_mode_guard = EnvVarGuard::set("KAMN_NODE_SYNC_MODE", Some("turbo"));
    let config_path = write_temp_node_config("role=processor\n");

    let args = vec![
        "kamn-node".to_owned(),
        "--config-file".to_owned(),
        config_path.to_string_lossy().to_string(),
    ];

    let parse_result = parse_args(args);
    std::fs::remove_file(config_path).expect("temp config file should clean up");

    assert!(
        matches!(
            parse_result,
            Err(ConfigError::InvalidSyncMode(value)) if value == "turbo"
        ),
        "invalid KAMN_NODE_SYNC_MODE override must fail closed with typed config error"
    );
}

#[test]
fn integration_config_layering_executes_bootstrap_report_with_expected_precedence() {
    let _env_lock = signer_env_lock()
        .lock()
        .unwrap_or_else(|error| error.into_inner());
    let _chain_id_guard = EnvVarGuard::set("KAMN_NODE_CHAIN_ID", Some("kamn-env"));
    let config_path = write_temp_node_config("role=listener\nchain_id=kamn-config-file\n");
    let args = vec![
        "kamn-node".to_owned(),
        "--config-file".to_owned(),
        config_path.to_string_lossy().to_string(),
        "--role".to_owned(),
        "processor".to_owned(),
        "--chain-id".to_owned(),
        "kamn-cli".to_owned(),
    ];

    let parsed_result = parse_args(args);
    std::fs::remove_file(config_path).expect("temp config file should clean up");
    let parsed = parsed_result.expect("layered args should parse");
    let report = execute(parsed).expect("bootstrap execution should succeed");

    assert_eq!(report.role, "processor");
    assert_eq!(report.chain_id, "kamn-cli");
}
