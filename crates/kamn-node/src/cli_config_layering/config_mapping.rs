use super::ConfigError;

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

pub(super) fn map_config_entry_to_args(
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
        "api_body_limit_bytes" => push_key_value_flag(&mut mapped, value, "--api-body-limit-bytes"),
        "api_concurrency_limit" => {
            push_key_value_flag(&mut mapped, value, "--api-concurrency-limit")
        }
        "api_rate_limit_per_second" => {
            push_key_value_flag(&mut mapped, value, "--api-rate-limit-per-second")
        }
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
