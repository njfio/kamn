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
#[path = "cli_core_common_option_parsing.rs"]
mod cli_core_common_option_parsing;
#[path = "cli_daemon_option_parsing.rs"]
mod cli_daemon_option_parsing;
#[path = "cli_endpoint_option_parsing.rs"]
mod cli_endpoint_option_parsing;
#[path = "cli_kolme_live_option_parsing.rs"]
mod cli_kolme_live_option_parsing;
#[path = "cli_planning_recovery_option_parsing.rs"]
mod cli_planning_recovery_option_parsing;
#[path = "cli_post_parse_guards.rs"]
mod cli_post_parse_guards;
#[path = "cli_runtime_mode_validation.rs"]
mod cli_runtime_mode_validation;
#[path = "cli_value_parsers.rs"]
mod cli_value_parsers;

use cli_config_layering::build_layered_cli_args;
use cli_core_common_option_parsing::{try_parse_core_common_option, CoreCommonOptionState};
use cli_daemon_option_parsing::{try_parse_daemon_option, DaemonOptionState};
use cli_endpoint_option_parsing::{try_parse_endpoint_option, EndpointOptionState};
use cli_kolme_live_option_parsing::{try_parse_kolme_live_option, KolmeLiveOptionState};
use cli_planning_recovery_option_parsing::{
    try_parse_planning_recovery_option, PlanningRecoveryOptionState,
};
use cli_post_parse_guards::{
    apply_profile_defaults, validate_endpoint_guards, EndpointGuardInputs, ProfileDefaultsInputs,
};
use cli_runtime_mode_validation::{
    validate_runtime_mode_requirements, RuntimeModeValidationInputs,
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
        if try_parse_core_common_option(
            arg.as_str(),
            &mut iter,
            &mut CoreCommonOptionState {
                role: &mut role,
                profile: &mut profile,
                chain_id: &mut chain_id,
                chain_version: &mut chain_version,
                storage_dir: &mut storage_dir,
                enable_gossip: &mut enable_gossip,
                sync_mode: &mut sync_mode,
                runtime_mode: &mut runtime_mode,
                output_mode: &mut output_mode,
                diagnostics_mode: &mut diagnostics_mode,
                role_overridden: &mut role_overridden,
                chain_id_overridden: &mut chain_id_overridden,
                chain_version_overridden: &mut chain_version_overridden,
                storage_dir_overridden: &mut storage_dir_overridden,
                gossip_overridden: &mut gossip_overridden,
                sync_mode_overridden: &mut sync_mode_overridden,
            },
        )? {
            continue;
        }
        if try_parse_daemon_option(
            arg.as_str(),
            &mut iter,
            &mut DaemonOptionState {
                daemon_max_ticks: &mut daemon_max_ticks,
                daemon_tick_interval_ms: &mut daemon_tick_interval_ms,
                daemon_shutdown_signal_ticks: &mut daemon_shutdown_signal_ticks,
                daemon_shutdown_os_signals: &mut daemon_shutdown_os_signals,
                daemon_shutdown_drain_ticks: &mut daemon_shutdown_drain_ticks,
                daemon_shutdown_timeout_ticks: &mut daemon_shutdown_timeout_ticks,
                daemon_peer_id: &mut daemon_peer_id,
                daemon_lifecycle_events: &mut daemon_lifecycle_events,
            },
        )? {
            continue;
        }
        if try_parse_endpoint_option(
            arg.as_str(),
            &mut iter,
            &mut EndpointOptionState {
                api_bind_addr: &mut api_bind_addr,
                api_max_requests: &mut api_max_requests,
                api_idle_timeout_ms: &mut api_idle_timeout_ms,
                api_body_limit_bytes: &mut api_body_limit_bytes,
                api_concurrency_limit: &mut api_concurrency_limit,
                api_rate_limit_per_second: &mut api_rate_limit_per_second,
                observability_endpoint_bind_addr: &mut observability_endpoint_bind_addr,
                observability_endpoint_metrics_path: &mut observability_endpoint_metrics_path,
                observability_endpoint_health_path: &mut observability_endpoint_health_path,
                observability_endpoint_max_requests: &mut observability_endpoint_max_requests,
                observability_endpoint_idle_timeout_ms: &mut observability_endpoint_idle_timeout_ms,
                api_max_requests_overridden: &mut api_max_requests_overridden,
                api_idle_timeout_ms_overridden: &mut api_idle_timeout_ms_overridden,
                api_body_limit_bytes_overridden: &mut api_body_limit_bytes_overridden,
                api_concurrency_limit_overridden: &mut api_concurrency_limit_overridden,
                api_rate_limit_per_second_overridden: &mut api_rate_limit_per_second_overridden,
                observability_endpoint_metrics_path_overridden:
                    &mut observability_endpoint_metrics_path_overridden,
                observability_endpoint_health_path_overridden:
                    &mut observability_endpoint_health_path_overridden,
                observability_endpoint_max_requests_overridden:
                    &mut observability_endpoint_max_requests_overridden,
                observability_endpoint_idle_timeout_ms_overridden:
                    &mut observability_endpoint_idle_timeout_ms_overridden,
            },
        )? {
            continue;
        }
        if try_parse_kolme_live_option(
            arg.as_str(),
            &mut iter,
            &mut KolmeLiveOptionState {
                kolme_live_base_url: &mut kolme_live_base_url,
                kolme_live_provider_hint: &mut kolme_live_provider_hint,
                kolme_live_signing_profile: &mut kolme_live_signing_profile,
                kolme_live_strict_signer_contracts: &mut kolme_live_strict_signer_contracts,
                kolme_live_signer_profile: &mut kolme_live_signer_profile,
                kolme_live_signer_key_source: &mut kolme_live_signer_key_source,
            },
        )? {
            continue;
        }
        if try_parse_planning_recovery_option(
            arg.as_str(),
            &mut iter,
            &mut PlanningRecoveryOptionState {
                expected_state_version: &mut expected_state_version,
                expected_state_hash: &mut expected_state_hash,
                proposals: &mut proposals,
                rejoin_attempts: &mut rejoin_attempts,
            },
        )? {
            continue;
        }
        match arg.as_str() {
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
