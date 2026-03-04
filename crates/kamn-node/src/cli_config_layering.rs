use super::ConfigError;
use std::{env, fs};

#[path = "cli_config_layering/config_mapping.rs"]
mod config_mapping;

use config_mapping::map_config_entry_to_args;

const CONFIG_FILE_FLAG: &str = "--config-file";
const NODE_CONFIG_FILE_ENV: &str = "KAMN_NODE_CONFIG_FILE";
const DUAL_CONFIG_FILE_SOURCE_ERROR: &str =
    "both --config-file and KAMN_NODE_CONFIG_FILE are set; declare one config source";

const ENV_OVERRIDE_MAPPINGS: [(&str, &str); 31] = [
    ("KAMN_NODE_PROFILE", "profile"),
    ("KAMN_NODE_ROLE", "role"),
    ("KAMN_NODE_CHAIN_ID", "chain_id"),
    ("KAMN_NODE_CHAIN_VERSION", "chain_version"),
    ("KAMN_NODE_STORAGE_DIR", "storage_dir"),
    ("KAMN_NODE_ENABLE_GOSSIP", "enable_gossip"),
    ("KAMN_NODE_SYNC_MODE", "sync_mode"),
    ("KAMN_NODE_RUNTIME_MODE", "runtime_mode"),
    ("KAMN_NODE_DAEMON_MAX_TICKS", "daemon_max_ticks"),
    (
        "KAMN_NODE_DAEMON_TICK_INTERVAL_MS",
        "daemon_tick_interval_ms",
    ),
    ("KAMN_NODE_EXPECTED_STATE_VERSION", "expected_state_version"),
    ("KAMN_NODE_EXPECTED_STATE_HASH", "expected_state_hash"),
    ("KAMN_NODE_API_BIND", "api_bind"),
    ("KAMN_NODE_API_MAX_REQUESTS", "api_max_requests"),
    ("KAMN_NODE_API_IDLE_TIMEOUT_MS", "api_idle_timeout_ms"),
    ("KAMN_NODE_API_BODY_LIMIT_BYTES", "api_body_limit_bytes"),
    ("KAMN_NODE_API_CONCURRENCY_LIMIT", "api_concurrency_limit"),
    (
        "KAMN_NODE_API_RATE_LIMIT_PER_SECOND",
        "api_rate_limit_per_second",
    ),
    (
        "KAMN_NODE_OBSERVABILITY_ENDPOINT_BIND",
        "observability_endpoint_bind",
    ),
    (
        "KAMN_NODE_OBSERVABILITY_ENDPOINT_METRICS_PATH",
        "observability_endpoint_metrics_path",
    ),
    (
        "KAMN_NODE_OBSERVABILITY_ENDPOINT_HEALTH_PATH",
        "observability_endpoint_health_path",
    ),
    (
        "KAMN_NODE_OBSERVABILITY_ENDPOINT_MAX_REQUESTS",
        "observability_endpoint_max_requests",
    ),
    (
        "KAMN_NODE_OBSERVABILITY_ENDPOINT_IDLE_TIMEOUT_MS",
        "observability_endpoint_idle_timeout_ms",
    ),
    ("KAMN_NODE_KOLME_LIVE_BASE_URL", "kolme_live_base_url"),
    (
        "KAMN_NODE_KOLME_LIVE_PROVIDER_HINT",
        "kolme_live_provider_hint",
    ),
    (
        "KAMN_NODE_KOLME_LIVE_SIGNING_PROFILE",
        "kolme_live_signing_profile",
    ),
    (
        "KAMN_NODE_KOLME_LIVE_STRICT_SIGNER_CONTRACTS",
        "kolme_live_strict_signer_contracts",
    ),
    (
        "KAMN_NODE_KOLME_LIVE_SIGNER_PROFILE",
        "kolme_live_signer_profile",
    ),
    (
        "KAMN_NODE_KOLME_LIVE_SIGNER_KEY_SOURCE",
        "kolme_live_signer_key_source",
    ),
    ("KAMN_NODE_OUTPUT", "output"),
    ("KAMN_NODE_DIAGNOSTICS", "diagnostics"),
];

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
        let source = format!("config file {path} line {}", line_index + 1);
        args.extend(map_config_entry_to_args(
            key.trim(),
            value.trim(),
            source.as_str(),
        )?);
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
    for (env_name, key) in ENV_OVERRIDE_MAPPINGS {
        append_env_override(&mut args, env_name, key)?;
    }
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

pub(super) fn build_layered_cli_args(raw_args: Vec<String>) -> Result<Vec<String>, ConfigError> {
    let (config_file_from_cli, args_without_config) = extract_config_file_path(raw_args)?;
    let mut layered_args = Vec::new();
    let bin = args_without_config
        .first()
        .cloned()
        .unwrap_or_else(|| "kamn-node".to_owned());
    layered_args.push(bin);
    let config_file_from_env = read_env_var_trimmed(NODE_CONFIG_FILE_ENV)?;
    if config_file_from_cli.is_some() && config_file_from_env.is_some() {
        return Err(ConfigError::InvalidNodeConfig(
            DUAL_CONFIG_FILE_SOURCE_ERROR.to_owned(),
        ));
    }
    if let Some(path) = config_file_from_cli.or(config_file_from_env).as_deref() {
        layered_args.extend(parse_config_file_args(path)?);
    }
    layered_args.extend(collect_env_override_args()?);
    layered_args.extend(args_without_config.into_iter().skip(1));
    Ok(layered_args)
}
