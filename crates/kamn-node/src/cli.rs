use super::{
    normalize_kolme_live_signer_key_source, normalize_kolme_live_signer_profile_selector,
    ConfigError, DiagnosticsMode, LocalProfile, NodeCli, NodeRole, OutputMode, PeerLifecycleEvent,
    ProposalCandidate, RejoinAttempt, RuntimeMode, RuntimeModeKind, SyncMode,
    DEFAULT_OBSERVABILITY_ENDPOINT_HEALTH_PATH, DEFAULT_OBSERVABILITY_ENDPOINT_IDLE_TIMEOUT_MS,
    DEFAULT_OBSERVABILITY_ENDPOINT_MAX_REQUESTS, DEFAULT_OBSERVABILITY_ENDPOINT_METRICS_PATH,
    DEFAULT_SERVICE_API_IDLE_TIMEOUT_MS, DEFAULT_SERVICE_API_MAX_REQUESTS,
    KOLME_IN_MEMORY_PROVIDER_MARKER, KOLME_LIVE_SIGNING_PROFILE,
};
use std::{env, fs};

const CONFIG_FILE_FLAG: &str = "--config-file";
const NODE_CONFIG_FILE_ENV: &str = "KAMN_NODE_CONFIG_FILE";

fn read_env_var_trimmed(name: &str) -> Result<Option<String>, ConfigError> {
    match env::var(name) {
        Ok(value) => {
            let trimmed = value.trim();
            if trimmed.is_empty() {
                return Err(ConfigError::InvalidNodeConfig(format!(
                    "{name} must not be empty when set"
                )));
            }
            Ok(Some(trimmed.to_owned()))
        }
        Err(env::VarError::NotPresent) => Ok(None),
        Err(env::VarError::NotUnicode(_)) => Err(ConfigError::InvalidNodeConfig(format!(
            "{name} must be valid utf-8"
        ))),
    }
}

fn parse_bool_override(value: &str, source: &str) -> Result<bool, ConfigError> {
    match value {
        "true" => Ok(true),
        "false" => Ok(false),
        _ => Err(ConfigError::InvalidNodeConfig(format!(
            "{source} must be one of: true,false"
        ))),
    }
}

fn push_key_value_flag(args: &mut Vec<String>, value: &str, flag: &str) {
    args.push(flag.to_owned());
    args.push(value.to_owned());
}

fn map_config_entry_to_args(
    key: &str,
    value: &str,
    source: &str,
) -> Result<Vec<String>, ConfigError> {
    let mut mapped = Vec::new();
    match key {
        "profile" => push_key_value_flag(&mut mapped, value, "--profile"),
        "role" => push_key_value_flag(&mut mapped, value, "--role"),
        "chain_id" => push_key_value_flag(&mut mapped, value, "--chain-id"),
        "chain_version" => push_key_value_flag(&mut mapped, value, "--chain-version"),
        "storage_dir" => push_key_value_flag(&mut mapped, value, "--storage-dir"),
        "enable_gossip" => {
            if !parse_bool_override(value, source)? {
                mapped.push("--disable-gossip".to_owned());
            }
        }
        "sync_mode" => push_key_value_flag(&mut mapped, value, "--sync-mode"),
        "runtime_mode" => push_key_value_flag(&mut mapped, value, "--runtime-mode"),
        "expected_state_version" => {
            push_key_value_flag(&mut mapped, value, "--expected-state-version")
        }
        "expected_state_hash" => push_key_value_flag(&mut mapped, value, "--expected-state-hash"),
        "proposal" => push_key_value_flag(&mut mapped, value, "--proposal"),
        "rejoin_attempt" => push_key_value_flag(&mut mapped, value, "--rejoin-attempt"),
        "daemon_max_ticks" => push_key_value_flag(&mut mapped, value, "--daemon-max-ticks"),
        "daemon_tick_interval_ms" => {
            push_key_value_flag(&mut mapped, value, "--daemon-tick-interval-ms")
        }
        "daemon_shutdown_signal_tick" => {
            push_key_value_flag(&mut mapped, value, "--daemon-shutdown-signal-tick")
        }
        "daemon_shutdown_os_signals" => {
            if parse_bool_override(value, source)? {
                mapped.push("--daemon-shutdown-os-signals".to_owned());
            }
        }
        "daemon_shutdown_drain_ticks" => {
            push_key_value_flag(&mut mapped, value, "--daemon-shutdown-drain-ticks")
        }
        "daemon_shutdown_timeout_ticks" => {
            push_key_value_flag(&mut mapped, value, "--daemon-shutdown-timeout-ticks")
        }
        "daemon_peer_id" => push_key_value_flag(&mut mapped, value, "--daemon-peer-id"),
        "daemon_lifecycle_event" => {
            push_key_value_flag(&mut mapped, value, "--daemon-lifecycle-event")
        }
        "kolme_live_base_url" => push_key_value_flag(&mut mapped, value, "--kolme-live-base-url"),
        "kolme_live_provider_hint" => {
            push_key_value_flag(&mut mapped, value, "--kolme-live-provider-hint")
        }
        "kolme_live_signing_profile" => {
            push_key_value_flag(&mut mapped, value, "--kolme-live-signing-profile")
        }
        "kolme_live_strict_signer_contracts" => {
            if parse_bool_override(value, source)? {
                mapped.push("--kolme-live-strict-signer-contracts".to_owned());
            }
        }
        "kolme_live_signer_profile" => {
            push_key_value_flag(&mut mapped, value, "--kolme-live-signer-profile")
        }
        "kolme_live_signer_key_source" => {
            push_key_value_flag(&mut mapped, value, "--kolme-live-signer-key-source")
        }
        "api_bind" => push_key_value_flag(&mut mapped, value, "--api-bind"),
        "api_max_requests" => push_key_value_flag(&mut mapped, value, "--api-max-requests"),
        "api_idle_timeout_ms" => push_key_value_flag(&mut mapped, value, "--api-idle-timeout-ms"),
        "observability_endpoint_bind" => {
            push_key_value_flag(&mut mapped, value, "--observability-endpoint-bind")
        }
        "observability_endpoint_metrics_path" => {
            push_key_value_flag(&mut mapped, value, "--observability-endpoint-metrics-path")
        }
        "observability_endpoint_health_path" => {
            push_key_value_flag(&mut mapped, value, "--observability-endpoint-health-path")
        }
        "observability_endpoint_max_requests" => {
            push_key_value_flag(&mut mapped, value, "--observability-endpoint-max-requests")
        }
        "observability_endpoint_idle_timeout_ms" => push_key_value_flag(
            &mut mapped,
            value,
            "--observability-endpoint-idle-timeout-ms",
        ),
        "output" => push_key_value_flag(&mut mapped, value, "--output"),
        "diagnostics" => push_key_value_flag(&mut mapped, value, "--diagnostics"),
        _ => {
            return Err(ConfigError::InvalidNodeConfig(format!(
                "{source} contains unsupported key: {key}"
            )));
        }
    }
    Ok(mapped)
}

fn parse_config_file_args(path: &str) -> Result<Vec<String>, ConfigError> {
    let content = fs::read_to_string(path).map_err(|error| {
        ConfigError::InvalidNodeConfig(format!("failed to read config file {path}: {error}"))
    })?;
    let mut args = Vec::new();
    for (line_index, raw_line) in content.lines().enumerate() {
        let trimmed = raw_line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        let (key, value) = trimmed.split_once('=').ok_or_else(|| {
            ConfigError::InvalidNodeConfig(format!(
                "config file {path} line {} must match key=value",
                line_index + 1
            ))
        })?;
        let key = key.trim();
        let value = value.trim();
        let source = format!("config file {path} line {}", line_index + 1);
        args.extend(map_config_entry_to_args(key, value, source.as_str())?);
    }
    Ok(args)
}

fn append_env_override(
    args: &mut Vec<String>,
    env_name: &str,
    key: &str,
) -> Result<(), ConfigError> {
    if let Some(value) = read_env_var_trimmed(env_name)? {
        let source = format!("environment variable {env_name}");
        args.extend(map_config_entry_to_args(
            key,
            value.as_str(),
            source.as_str(),
        )?);
    }
    Ok(())
}

fn collect_env_override_args() -> Result<Vec<String>, ConfigError> {
    let mut args = Vec::new();
    append_env_override(&mut args, "KAMN_NODE_PROFILE", "profile")?;
    append_env_override(&mut args, "KAMN_NODE_ROLE", "role")?;
    append_env_override(&mut args, "KAMN_NODE_CHAIN_ID", "chain_id")?;
    append_env_override(&mut args, "KAMN_NODE_CHAIN_VERSION", "chain_version")?;
    append_env_override(&mut args, "KAMN_NODE_STORAGE_DIR", "storage_dir")?;
    append_env_override(&mut args, "KAMN_NODE_ENABLE_GOSSIP", "enable_gossip")?;
    append_env_override(&mut args, "KAMN_NODE_SYNC_MODE", "sync_mode")?;
    append_env_override(&mut args, "KAMN_NODE_RUNTIME_MODE", "runtime_mode")?;
    append_env_override(
        &mut args,
        "KAMN_NODE_EXPECTED_STATE_VERSION",
        "expected_state_version",
    )?;
    append_env_override(
        &mut args,
        "KAMN_NODE_EXPECTED_STATE_HASH",
        "expected_state_hash",
    )?;
    append_env_override(&mut args, "KAMN_NODE_API_BIND", "api_bind")?;
    append_env_override(&mut args, "KAMN_NODE_API_MAX_REQUESTS", "api_max_requests")?;
    append_env_override(
        &mut args,
        "KAMN_NODE_API_IDLE_TIMEOUT_MS",
        "api_idle_timeout_ms",
    )?;
    append_env_override(
        &mut args,
        "KAMN_NODE_OBSERVABILITY_ENDPOINT_BIND",
        "observability_endpoint_bind",
    )?;
    append_env_override(
        &mut args,
        "KAMN_NODE_OBSERVABILITY_ENDPOINT_METRICS_PATH",
        "observability_endpoint_metrics_path",
    )?;
    append_env_override(
        &mut args,
        "KAMN_NODE_OBSERVABILITY_ENDPOINT_HEALTH_PATH",
        "observability_endpoint_health_path",
    )?;
    append_env_override(
        &mut args,
        "KAMN_NODE_OBSERVABILITY_ENDPOINT_MAX_REQUESTS",
        "observability_endpoint_max_requests",
    )?;
    append_env_override(
        &mut args,
        "KAMN_NODE_OBSERVABILITY_ENDPOINT_IDLE_TIMEOUT_MS",
        "observability_endpoint_idle_timeout_ms",
    )?;
    append_env_override(
        &mut args,
        "KAMN_NODE_KOLME_LIVE_BASE_URL",
        "kolme_live_base_url",
    )?;
    append_env_override(
        &mut args,
        "KAMN_NODE_KOLME_LIVE_PROVIDER_HINT",
        "kolme_live_provider_hint",
    )?;
    append_env_override(
        &mut args,
        "KAMN_NODE_KOLME_LIVE_SIGNING_PROFILE",
        "kolme_live_signing_profile",
    )?;
    append_env_override(
        &mut args,
        "KAMN_NODE_KOLME_LIVE_STRICT_SIGNER_CONTRACTS",
        "kolme_live_strict_signer_contracts",
    )?;
    append_env_override(
        &mut args,
        "KAMN_NODE_KOLME_LIVE_SIGNER_PROFILE",
        "kolme_live_signer_profile",
    )?;
    append_env_override(
        &mut args,
        "KAMN_NODE_KOLME_LIVE_SIGNER_KEY_SOURCE",
        "kolme_live_signer_key_source",
    )?;
    append_env_override(&mut args, "KAMN_NODE_OUTPUT", "output")?;
    append_env_override(&mut args, "KAMN_NODE_DIAGNOSTICS", "diagnostics")?;
    Ok(args)
}

fn extract_config_file_path(
    raw_args: Vec<String>,
) -> Result<(Option<String>, Vec<String>), ConfigError> {
    let mut args_without_config = Vec::new();
    let mut config_file_path: Option<String> = None;
    let mut iter = raw_args.into_iter();
    if let Some(bin) = iter.next() {
        args_without_config.push(bin);
    }
    while let Some(arg) = iter.next() {
        if arg == CONFIG_FILE_FLAG {
            let value = iter
                .next()
                .ok_or(ConfigError::MissingArgumentValue(CONFIG_FILE_FLAG))?;
            if config_file_path.is_some() {
                return Err(ConfigError::InvalidNodeConfig(
                    "duplicate --config-file declarations are not allowed".to_owned(),
                ));
            }
            config_file_path = Some(value);
            continue;
        }
        args_without_config.push(arg);
    }
    Ok((config_file_path, args_without_config))
}

fn build_layered_cli_args(raw_args: Vec<String>) -> Result<Vec<String>, ConfigError> {
    let (config_file_from_cli, args_without_config) = extract_config_file_path(raw_args)?;
    let mut layered_args = Vec::new();
    let bin = args_without_config
        .first()
        .cloned()
        .unwrap_or_else(|| "kamn-node".to_owned());
    layered_args.push(bin);
    let config_file_from_env = read_env_var_trimmed(NODE_CONFIG_FILE_ENV)?;
    let config_file_path = config_file_from_cli.or(config_file_from_env);
    if let Some(path) = config_file_path.as_deref() {
        layered_args.extend(parse_config_file_args(path)?);
        layered_args.extend(collect_env_override_args()?);
    }
    layered_args.extend(args_without_config.into_iter().skip(1));
    Ok(layered_args)
}

pub(super) fn parse_args<I>(args: I) -> Result<NodeCli, ConfigError>
where
    I: IntoIterator<Item = String>,
{
    let layered_args = build_layered_cli_args(args.into_iter().collect())?;
    let mut role: Option<NodeRole> = None;
    let mut profile: Option<LocalProfile> = None;
    let mut chain_id = String::from("kamn-devnet");
    let mut chain_version = String::from("v0.1.0");
    let mut storage_dir = String::from("./data");
    let mut enable_gossip = true;
    let mut sync_mode = SyncMode::Fast;
    let mut runtime_mode = RuntimeMode::bootstrap();
    let mut expected_state_version: Option<u64> = None;
    let mut expected_state_hash: Option<String> = None;
    let mut proposals: Vec<ProposalCandidate> = Vec::new();
    let mut rejoin_attempts: Vec<RejoinAttempt> = Vec::new();
    let mut daemon_max_ticks: Option<u64> = None;
    let mut daemon_tick_interval_ms: Option<u64> = None;
    let mut daemon_shutdown_signal_ticks: Vec<u64> = Vec::new();
    let mut daemon_shutdown_os_signals = false;
    let mut daemon_shutdown_drain_ticks: Option<u64> = None;
    let mut daemon_shutdown_timeout_ticks: Option<u64> = None;
    let mut daemon_peer_id: Option<String> = None;
    let mut daemon_lifecycle_events: Vec<PeerLifecycleEvent> = Vec::new();
    let mut kolme_live_base_url: Option<String> = None;
    let mut kolme_live_provider_hint: Option<String> = None;
    let mut kolme_live_signing_profile: Option<String> = None;
    let mut kolme_live_strict_signer_contracts = false;
    let mut kolme_live_signer_profile: Option<String> = None;
    let mut kolme_live_signer_key_source: Option<String> = None;
    let mut api_bind_addr: Option<String> = None;
    let mut api_max_requests = DEFAULT_SERVICE_API_MAX_REQUESTS;
    let mut api_idle_timeout_ms = DEFAULT_SERVICE_API_IDLE_TIMEOUT_MS;
    let mut observability_endpoint_bind_addr: Option<String> = None;
    let mut observability_endpoint_metrics_path =
        DEFAULT_OBSERVABILITY_ENDPOINT_METRICS_PATH.to_owned();
    let mut observability_endpoint_health_path =
        DEFAULT_OBSERVABILITY_ENDPOINT_HEALTH_PATH.to_owned();
    let mut observability_endpoint_max_requests = DEFAULT_OBSERVABILITY_ENDPOINT_MAX_REQUESTS;
    let mut observability_endpoint_idle_timeout_ms = DEFAULT_OBSERVABILITY_ENDPOINT_IDLE_TIMEOUT_MS;
    let mut output_mode = OutputMode::text();
    let mut diagnostics_mode = DiagnosticsMode::basic();
    let mut observability_endpoint_metrics_path_overridden = false;
    let mut observability_endpoint_health_path_overridden = false;
    let mut observability_endpoint_max_requests_overridden = false;
    let mut observability_endpoint_idle_timeout_ms_overridden = false;
    let mut api_max_requests_overridden = false;
    let mut api_idle_timeout_ms_overridden = false;
    let mut role_overridden = false;
    let mut chain_id_overridden = false;
    let mut chain_version_overridden = false;
    let mut storage_dir_overridden = false;
    let mut gossip_overridden = false;
    let mut sync_mode_overridden = false;

    let mut iter = layered_args.into_iter();
    let _bin = iter.next();

    while let Some(arg) = iter.next() {
        match arg.as_str() {
            "--role" => {
                let value = iter
                    .next()
                    .ok_or(ConfigError::MissingArgumentValue("--role"))?;
                role = Some(value.parse::<NodeRole>()?);
                role_overridden = true;
            }
            "--profile" => {
                let value = iter
                    .next()
                    .ok_or(ConfigError::MissingArgumentValue("--profile"))?;
                profile = Some(LocalProfile::parse(&value)?);
            }
            "--chain-id" => {
                chain_id = iter
                    .next()
                    .ok_or(ConfigError::MissingArgumentValue("--chain-id"))?;
                chain_id_overridden = true;
            }
            "--chain-version" => {
                chain_version = iter
                    .next()
                    .ok_or(ConfigError::MissingArgumentValue("--chain-version"))?;
                chain_version_overridden = true;
            }
            "--storage-dir" => {
                storage_dir = iter
                    .next()
                    .ok_or(ConfigError::MissingArgumentValue("--storage-dir"))?;
                storage_dir_overridden = true;
            }
            "--disable-gossip" => {
                enable_gossip = false;
                gossip_overridden = true;
            }
            "--sync-mode" => {
                let value = iter
                    .next()
                    .ok_or(ConfigError::MissingArgumentValue("--sync-mode"))?;
                sync_mode = value.parse::<SyncMode>()?;
                sync_mode_overridden = true;
            }
            "--runtime-mode" => {
                let value = iter
                    .next()
                    .ok_or(ConfigError::MissingArgumentValue("--runtime-mode"))?;
                runtime_mode = RuntimeMode::parse(&value)?;
            }
            "--expected-state-version" => {
                let value = iter.next().ok_or(ConfigError::MissingArgumentValue(
                    "--expected-state-version",
                ))?;
                expected_state_version = Some(parse_state_version_arg(&value)?);
            }
            "--expected-state-hash" => {
                expected_state_hash = Some(
                    iter.next()
                        .ok_or(ConfigError::MissingArgumentValue("--expected-state-hash"))?,
                );
            }
            "--proposal" => {
                let value = iter
                    .next()
                    .ok_or(ConfigError::MissingArgumentValue("--proposal"))?;
                proposals.push(parse_proposal_candidate(&value)?);
            }
            "--rejoin-attempt" => {
                let value = iter
                    .next()
                    .ok_or(ConfigError::MissingArgumentValue("--rejoin-attempt"))?;
                rejoin_attempts.push(parse_rejoin_attempt(&value)?);
            }
            "--daemon-max-ticks" => {
                let value = iter
                    .next()
                    .ok_or(ConfigError::MissingArgumentValue("--daemon-max-ticks"))?;
                daemon_max_ticks = Some(parse_daemon_control_arg(&value)?);
            }
            "--daemon-tick-interval-ms" => {
                let value = iter.next().ok_or(ConfigError::MissingArgumentValue(
                    "--daemon-tick-interval-ms",
                ))?;
                daemon_tick_interval_ms = Some(parse_daemon_control_arg(&value)?);
            }
            "--daemon-shutdown-signal-tick" => {
                let value = iter.next().ok_or(ConfigError::MissingArgumentValue(
                    "--daemon-shutdown-signal-tick",
                ))?;
                daemon_shutdown_signal_ticks.push(parse_daemon_control_arg(&value)?);
            }
            "--daemon-shutdown-os-signals" => {
                daemon_shutdown_os_signals = true;
            }
            "--daemon-shutdown-drain-ticks" => {
                let value = iter.next().ok_or(ConfigError::MissingArgumentValue(
                    "--daemon-shutdown-drain-ticks",
                ))?;
                daemon_shutdown_drain_ticks = Some(parse_daemon_control_arg(&value)?);
            }
            "--daemon-shutdown-timeout-ticks" => {
                let value = iter.next().ok_or(ConfigError::MissingArgumentValue(
                    "--daemon-shutdown-timeout-ticks",
                ))?;
                daemon_shutdown_timeout_ticks = Some(parse_daemon_control_arg(&value)?);
            }
            "--daemon-peer-id" => {
                daemon_peer_id = Some(
                    iter.next()
                        .ok_or(ConfigError::MissingArgumentValue("--daemon-peer-id"))?,
                );
            }
            "--daemon-lifecycle-event" => {
                let value = iter.next().ok_or(ConfigError::MissingArgumentValue(
                    "--daemon-lifecycle-event",
                ))?;
                daemon_lifecycle_events.push(parse_daemon_lifecycle_event(&value)?);
            }
            "--kolme-live-base-url" => {
                kolme_live_base_url = Some(
                    iter.next()
                        .ok_or(ConfigError::MissingArgumentValue("--kolme-live-base-url"))?,
                );
            }
            "--kolme-live-provider-hint" => {
                kolme_live_provider_hint = Some(iter.next().ok_or(
                    ConfigError::MissingArgumentValue("--kolme-live-provider-hint"),
                )?);
            }
            "--kolme-live-signing-profile" => {
                kolme_live_signing_profile = Some(iter.next().ok_or(
                    ConfigError::MissingArgumentValue("--kolme-live-signing-profile"),
                )?);
            }
            "--kolme-live-strict-signer-contracts" => {
                kolme_live_strict_signer_contracts = true;
            }
            "--kolme-live-signer-profile" => {
                kolme_live_signer_profile = Some(iter.next().ok_or(
                    ConfigError::MissingArgumentValue("--kolme-live-signer-profile"),
                )?);
            }
            "--kolme-live-signer-key-source" => {
                kolme_live_signer_key_source = Some(iter.next().ok_or(
                    ConfigError::MissingArgumentValue("--kolme-live-signer-key-source"),
                )?);
            }
            "--api-bind" => {
                api_bind_addr = Some(
                    iter.next()
                        .ok_or(ConfigError::MissingArgumentValue("--api-bind"))?,
                );
            }
            "--api-max-requests" => {
                let value = iter
                    .next()
                    .ok_or(ConfigError::MissingArgumentValue("--api-max-requests"))?;
                api_max_requests = parse_daemon_control_arg(&value)?;
                api_max_requests_overridden = true;
            }
            "--api-idle-timeout-ms" => {
                let value = iter
                    .next()
                    .ok_or(ConfigError::MissingArgumentValue("--api-idle-timeout-ms"))?;
                api_idle_timeout_ms = parse_daemon_control_arg(&value)?;
                api_idle_timeout_ms_overridden = true;
            }
            "--observability-endpoint-bind" => {
                observability_endpoint_bind_addr = Some(iter.next().ok_or(
                    ConfigError::MissingArgumentValue("--observability-endpoint-bind"),
                )?);
            }
            "--observability-endpoint-metrics-path" => {
                observability_endpoint_metrics_path = iter.next().ok_or(
                    ConfigError::MissingArgumentValue("--observability-endpoint-metrics-path"),
                )?;
                observability_endpoint_metrics_path_overridden = true;
            }
            "--observability-endpoint-health-path" => {
                observability_endpoint_health_path = iter.next().ok_or(
                    ConfigError::MissingArgumentValue("--observability-endpoint-health-path"),
                )?;
                observability_endpoint_health_path_overridden = true;
            }
            "--observability-endpoint-max-requests" => {
                let value = iter.next().ok_or(ConfigError::MissingArgumentValue(
                    "--observability-endpoint-max-requests",
                ))?;
                observability_endpoint_max_requests = parse_daemon_control_arg(&value)?;
                observability_endpoint_max_requests_overridden = true;
            }
            "--observability-endpoint-idle-timeout-ms" => {
                let value = iter.next().ok_or(ConfigError::MissingArgumentValue(
                    "--observability-endpoint-idle-timeout-ms",
                ))?;
                observability_endpoint_idle_timeout_ms = parse_daemon_control_arg(&value)?;
                observability_endpoint_idle_timeout_ms_overridden = true;
            }
            "--output" => {
                let value = iter
                    .next()
                    .ok_or(ConfigError::MissingArgumentValue("--output"))?;
                output_mode = OutputMode::parse(&value)?;
            }
            "--diagnostics" => {
                let value = iter
                    .next()
                    .ok_or(ConfigError::MissingArgumentValue("--diagnostics"))?;
                diagnostics_mode = DiagnosticsMode::parse(&value)?;
            }
            unknown => {
                return Err(ConfigError::UnknownArgument(unknown.to_owned()));
            }
        }
    }

    if let Some(selected_profile) = profile {
        if !role_overridden {
            role = Some(selected_profile.default_role());
        }
        if !chain_id_overridden {
            chain_id = "kamn-localnet".to_owned();
        }
        if !chain_version_overridden {
            chain_version = "v0.1.0".to_owned();
        }
        if !storage_dir_overridden {
            storage_dir = selected_profile.default_storage_dir().to_owned();
        }
        if !gossip_overridden {
            enable_gossip = true;
        }
        if !sync_mode_overridden {
            sync_mode = SyncMode::Fast;
        }
    }

    let role = role.ok_or(ConfigError::MissingArgumentValue("--role"))?;

    if runtime_mode.kind == RuntimeModeKind::Planning {
        if expected_state_hash.is_none() {
            return Err(ConfigError::MissingArgumentValue("--expected-state-hash"));
        }
        if proposals.is_empty() {
            return Err(ConfigError::MissingArgumentValue("--proposal"));
        }
    }
    if runtime_mode.kind == RuntimeModeKind::RecoveryCheck {
        if expected_state_version.is_none() {
            return Err(ConfigError::MissingArgumentValue(
                "--expected-state-version",
            ));
        }
        if expected_state_hash.is_none() {
            return Err(ConfigError::MissingArgumentValue("--expected-state-hash"));
        }
        if rejoin_attempts.is_empty() {
            return Err(ConfigError::MissingArgumentValue("--rejoin-attempt"));
        }
    }
    if runtime_mode.kind == RuntimeModeKind::Daemon {
        if daemon_max_ticks.is_none() {
            return Err(ConfigError::MissingArgumentValue("--daemon-max-ticks"));
        }
        if daemon_tick_interval_ms.is_none() {
            return Err(ConfigError::MissingArgumentValue(
                "--daemon-tick-interval-ms",
            ));
        }
        if !daemon_lifecycle_events.is_empty() && daemon_peer_id.is_none() {
            return Err(ConfigError::MissingArgumentValue("--daemon-peer-id"));
        }
        if !daemon_shutdown_signal_ticks.is_empty() {
            if daemon_shutdown_drain_ticks.is_none() {
                return Err(ConfigError::MissingArgumentValue(
                    "--daemon-shutdown-drain-ticks",
                ));
            }
            if daemon_shutdown_timeout_ticks.is_none() {
                return Err(ConfigError::MissingArgumentValue(
                    "--daemon-shutdown-timeout-ticks",
                ));
            }
        } else if (daemon_shutdown_drain_ticks.is_some() || daemon_shutdown_timeout_ticks.is_some())
            && !daemon_shutdown_os_signals
        {
            return Err(ConfigError::MissingArgumentValue(
                "--daemon-shutdown-signal-tick",
            ));
        }
    }
    if runtime_mode.kind == RuntimeModeKind::Api && api_bind_addr.is_none() {
        return Err(ConfigError::MissingArgumentValue("--api-bind"));
    }
    if runtime_mode.kind == RuntimeModeKind::KolmeLive {
        if kolme_live_base_url.is_none() {
            return Err(ConfigError::MissingArgumentValue("--kolme-live-base-url"));
        }
        if kolme_live_provider_hint.is_none() {
            return Err(ConfigError::MissingArgumentValue(
                "--kolme-live-provider-hint",
            ));
        }
        if kolme_live_signing_profile.is_none() {
            return Err(ConfigError::MissingArgumentValue(
                "--kolme-live-signing-profile",
            ));
        }
        let provider_hint =
            kolme_live_provider_hint
                .as_deref()
                .ok_or(ConfigError::MissingArgumentValue(
                    "--kolme-live-provider-hint",
                ))?;
        if provider_hint.contains(KOLME_IN_MEMORY_PROVIDER_MARKER) {
            return Err(ConfigError::InvalidKolmeLiveProviderHint(
                provider_hint.to_owned(),
            ));
        }
        let signing_profile =
            kolme_live_signing_profile
                .as_deref()
                .ok_or(ConfigError::MissingArgumentValue(
                    "--kolme-live-signing-profile",
                ))?;
        if signing_profile != KOLME_LIVE_SIGNING_PROFILE {
            return Err(ConfigError::InvalidKolmeLiveSigningProfile(
                signing_profile.to_owned(),
            ));
        }
        let key_source =
            kolme_live_signer_key_source
                .as_deref()
                .ok_or(ConfigError::MissingArgumentValue(
                    "--kolme-live-signer-key-source",
                ))?;
        normalize_kolme_live_signer_key_source(key_source)?;
        if kolme_live_strict_signer_contracts {
            let signer_profile =
                kolme_live_signer_profile
                    .as_deref()
                    .ok_or(ConfigError::MissingArgumentValue(
                        "--kolme-live-signer-profile",
                    ))?;
            normalize_kolme_live_signer_profile_selector(signer_profile)?;
        }
    }
    if api_bind_addr.is_none() && (api_max_requests_overridden || api_idle_timeout_ms_overridden) {
        return Err(ConfigError::MissingArgumentValue("--api-bind"));
    }
    if observability_endpoint_bind_addr.is_none()
        && (observability_endpoint_metrics_path_overridden
            || observability_endpoint_health_path_overridden
            || observability_endpoint_max_requests_overridden
            || observability_endpoint_idle_timeout_ms_overridden)
    {
        return Err(ConfigError::MissingArgumentValue(
            "--observability-endpoint-bind",
        ));
    }
    if observability_endpoint_bind_addr.is_some() {
        if !observability_endpoint_metrics_path.starts_with('/') {
            return Err(ConfigError::RuntimeDaemonLifecycle(
                "observability endpoint metrics path must start with '/'".to_owned(),
            ));
        }
        if !observability_endpoint_health_path.starts_with('/') {
            return Err(ConfigError::RuntimeDaemonLifecycle(
                "observability endpoint health path must start with '/'".to_owned(),
            ));
        }
    }

    Ok(NodeCli {
        profile,
        role,
        chain_id,
        chain_version,
        storage_dir,
        enable_gossip,
        sync_mode,
        runtime_mode,
        expected_state_version,
        expected_state_hash,
        proposals,
        rejoin_attempts,
        daemon_max_ticks,
        daemon_tick_interval_ms,
        daemon_shutdown_signal_ticks,
        daemon_shutdown_os_signals,
        daemon_shutdown_drain_ticks,
        daemon_shutdown_timeout_ticks,
        daemon_peer_id,
        daemon_lifecycle_events,
        kolme_live_base_url,
        kolme_live_provider_hint,
        kolme_live_signing_profile,
        kolme_live_strict_signer_contracts,
        kolme_live_signer_profile,
        kolme_live_signer_key_source,
        api_bind_addr,
        api_max_requests,
        api_idle_timeout_ms,
        observability_endpoint_bind_addr,
        observability_endpoint_metrics_path,
        observability_endpoint_health_path,
        observability_endpoint_max_requests,
        observability_endpoint_idle_timeout_ms,
        output_mode,
        diagnostics_mode,
    })
}

fn parse_state_version_arg(value: &str) -> Result<u64, ConfigError> {
    let state_version = value
        .parse::<u64>()
        .map_err(|_| ConfigError::InvalidExpectedStateVersion(value.to_owned()))?;
    if state_version == 0 {
        return Err(ConfigError::InvalidExpectedStateVersion(value.to_owned()));
    }
    Ok(state_version)
}

fn parse_proposal_candidate(value: &str) -> Result<ProposalCandidate, ConfigError> {
    let parts = value.split('|').collect::<Vec<&str>>();
    if parts.len() != 4 {
        return Err(ConfigError::InvalidProposalArgument(value.to_owned()));
    }
    let nonce = parts[2]
        .parse::<u64>()
        .map_err(|_| ConfigError::InvalidProposalArgument(value.to_owned()))?;
    ProposalCandidate::new(parts[0], parts[1], nonce, parts[3])
        .map_err(|error| ConfigError::RuntimePlanner(error.to_string()))
}

fn parse_rejoin_attempt(value: &str) -> Result<RejoinAttempt, ConfigError> {
    let parts = value.split('|').collect::<Vec<&str>>();
    if parts.len() != 4 {
        return Err(ConfigError::InvalidRejoinAttemptArgument(value.to_owned()));
    }
    let state_version = parts[1]
        .parse::<u64>()
        .map_err(|_| ConfigError::InvalidRejoinAttemptArgument(value.to_owned()))?;
    RejoinAttempt::new(parts[0], state_version, parts[2], parts[3])
        .map_err(|_| ConfigError::InvalidRejoinAttemptArgument(value.to_owned()))
}

fn parse_daemon_control_arg(value: &str) -> Result<u64, ConfigError> {
    let parsed = value
        .parse::<u64>()
        .map_err(|_| ConfigError::InvalidDaemonControlArgument(value.to_owned()))?;
    if parsed == 0 {
        return Err(ConfigError::InvalidDaemonControlArgument(value.to_owned()));
    }
    Ok(parsed)
}

fn parse_daemon_lifecycle_event(value: &str) -> Result<PeerLifecycleEvent, ConfigError> {
    match value {
        "start-connect" => Ok(PeerLifecycleEvent::StartConnect),
        "handshake-succeeded" => Ok(PeerLifecycleEvent::HandshakeSucceeded),
        "heartbeat-missed" => Ok(PeerLifecycleEvent::HeartbeatMissed),
        "heartbeat-restored" => Ok(PeerLifecycleEvent::HeartbeatRestored),
        "disconnect" => Ok(PeerLifecycleEvent::Disconnect),
        "rejoin" => Ok(PeerLifecycleEvent::Rejoin),
        _ => Err(ConfigError::InvalidDaemonLifecycleEvent(value.to_owned())),
    }
}
