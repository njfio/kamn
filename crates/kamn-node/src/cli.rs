use super::{
    normalize_kolme_live_signer_key_source, normalize_kolme_live_signer_profile_selector,
    ConfigError, DiagnosticsMode, LocalProfile, NodeCli, NodeRole, OutputMode, PeerLifecycleEvent,
    ProposalCandidate, RejoinAttempt, RuntimeMode, RuntimeModeKind, SyncMode,
    DEFAULT_OBSERVABILITY_ENDPOINT_HEALTH_PATH, DEFAULT_OBSERVABILITY_ENDPOINT_IDLE_TIMEOUT_MS,
    DEFAULT_OBSERVABILITY_ENDPOINT_MAX_REQUESTS, DEFAULT_OBSERVABILITY_ENDPOINT_METRICS_PATH,
    DEFAULT_SERVICE_API_BODY_LIMIT_BYTES, DEFAULT_SERVICE_API_CONCURRENCY_LIMIT,
    DEFAULT_SERVICE_API_IDLE_TIMEOUT_MS, DEFAULT_SERVICE_API_MAX_REQUESTS,
    DEFAULT_SERVICE_API_RATE_LIMIT_PER_SECOND, KOLME_IN_MEMORY_PROVIDER_MARKER,
    KOLME_LIVE_SIGNING_PROFILE,
};

#[path = "cli_config_layering.rs"]
mod cli_config_layering;
#[path = "cli_post_parse_guards.rs"]
mod cli_post_parse_guards;
#[path = "cli_runtime_mode_validation.rs"]
mod cli_runtime_mode_validation;
#[path = "cli_value_parsers.rs"]
mod cli_value_parsers;

use cli_config_layering::build_layered_cli_args;
use cli_post_parse_guards::{
    apply_profile_defaults, validate_endpoint_guards, EndpointGuardInputs, ProfileDefaultsInputs,
};
use cli_runtime_mode_validation::{
    validate_runtime_mode_requirements, RuntimeModeValidationInputs,
};
use cli_value_parsers::{
    parse_daemon_control_arg, parse_daemon_lifecycle_event, parse_proposal_candidate,
    parse_rejoin_attempt, parse_state_version_arg,
};

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
    let mut api_body_limit_bytes = DEFAULT_SERVICE_API_BODY_LIMIT_BYTES;
    let mut api_concurrency_limit = DEFAULT_SERVICE_API_CONCURRENCY_LIMIT;
    let mut api_rate_limit_per_second = DEFAULT_SERVICE_API_RATE_LIMIT_PER_SECOND;
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
    let mut api_body_limit_bytes_overridden = false;
    let mut api_concurrency_limit_overridden = false;
    let mut api_rate_limit_per_second_overridden = false;
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
            "--api-body-limit-bytes" => {
                let value = iter
                    .next()
                    .ok_or(ConfigError::MissingArgumentValue("--api-body-limit-bytes"))?;
                api_body_limit_bytes = parse_daemon_control_arg(&value)?;
                api_body_limit_bytes_overridden = true;
            }
            "--api-concurrency-limit" => {
                let value = iter
                    .next()
                    .ok_or(ConfigError::MissingArgumentValue("--api-concurrency-limit"))?;
                api_concurrency_limit = parse_daemon_control_arg(&value)?;
                api_concurrency_limit_overridden = true;
            }
            "--api-rate-limit-per-second" => {
                let value = iter.next().ok_or(ConfigError::MissingArgumentValue(
                    "--api-rate-limit-per-second",
                ))?;
                api_rate_limit_per_second = parse_daemon_control_arg(&value)?;
                api_rate_limit_per_second_overridden = true;
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

    apply_profile_defaults(ProfileDefaultsInputs {
        profile,
        role: &mut role,
        chain_id: &mut chain_id,
        chain_version: &mut chain_version,
        storage_dir: &mut storage_dir,
        enable_gossip: &mut enable_gossip,
        sync_mode: &mut sync_mode,
        role_overridden,
        chain_id_overridden,
        chain_version_overridden,
        storage_dir_overridden,
        gossip_overridden,
        sync_mode_overridden,
    });

    let role = role.ok_or(ConfigError::MissingArgumentValue("--role"))?;

    validate_runtime_mode_requirements(RuntimeModeValidationInputs {
        runtime_mode,
        expected_state_version,
        expected_state_hash: expected_state_hash.as_deref(),
        proposals_len: proposals.len(),
        rejoin_attempts_len: rejoin_attempts.len(),
        daemon_max_ticks,
        daemon_tick_interval_ms,
        daemon_shutdown_signal_ticks_len: daemon_shutdown_signal_ticks.len(),
        daemon_shutdown_os_signals,
        daemon_shutdown_drain_ticks,
        daemon_shutdown_timeout_ticks,
        daemon_peer_id_present: daemon_peer_id.is_some(),
        daemon_lifecycle_events_len: daemon_lifecycle_events.len(),
        api_bind_addr_present: api_bind_addr.is_some(),
        kolme_live_base_url: kolme_live_base_url.as_deref(),
        kolme_live_provider_hint: kolme_live_provider_hint.as_deref(),
        kolme_live_signing_profile: kolme_live_signing_profile.as_deref(),
        kolme_live_strict_signer_contracts,
        kolme_live_signer_profile: kolme_live_signer_profile.as_deref(),
        kolme_live_signer_key_source: kolme_live_signer_key_source.as_deref(),
    })?;
    validate_endpoint_guards(EndpointGuardInputs {
        api_bind_addr_present: api_bind_addr.is_some(),
        api_max_requests_overridden,
        api_idle_timeout_ms_overridden,
        api_body_limit_bytes_overridden,
        api_concurrency_limit_overridden,
        api_rate_limit_per_second_overridden,
        observability_endpoint_bind_addr_present: observability_endpoint_bind_addr.is_some(),
        observability_endpoint_metrics_path_overridden,
        observability_endpoint_health_path_overridden,
        observability_endpoint_max_requests_overridden,
        observability_endpoint_idle_timeout_ms_overridden,
        observability_endpoint_metrics_path: observability_endpoint_metrics_path.as_str(),
        observability_endpoint_health_path: observability_endpoint_health_path.as_str(),
    })?;

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
        api_body_limit_bytes,
        api_concurrency_limit,
        api_rate_limit_per_second,
        observability_endpoint_bind_addr,
        observability_endpoint_metrics_path,
        observability_endpoint_health_path,
        observability_endpoint_max_requests,
        observability_endpoint_idle_timeout_ms,
        output_mode,
        diagnostics_mode,
    })
}
