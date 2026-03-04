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
#[path = "cli_parse_loop.rs"]
mod cli_parse_loop;
#[path = "cli_parse_state.rs"]
mod cli_parse_state;
#[path = "cli_planning_recovery_option_parsing.rs"]
mod cli_planning_recovery_option_parsing;
#[path = "cli_post_parse_guards.rs"]
mod cli_post_parse_guards;
#[path = "cli_runtime_mode_validation.rs"]
mod cli_runtime_mode_validation;
#[path = "cli_value_parsers.rs"]
mod cli_value_parsers;

use cli_config_layering::build_layered_cli_args;
use cli_parse_loop::parse_layered_args_into_state;
use cli_parse_state::CliParseState;
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
    let mut state = CliParseState::new();

    parse_layered_args_into_state(layered_args, &mut state)?;

    apply_profile_defaults(ProfileDefaultsInputs {
        profile: state.profile,
        role: &mut state.role,
        chain_id: &mut state.chain_id,
        chain_version: &mut state.chain_version,
        storage_dir: &mut state.storage_dir,
        enable_gossip: &mut state.enable_gossip,
        sync_mode: &mut state.sync_mode,
        role_overridden: state.role_overridden,
        chain_id_overridden: state.chain_id_overridden,
        chain_version_overridden: state.chain_version_overridden,
        storage_dir_overridden: state.storage_dir_overridden,
        gossip_overridden: state.gossip_overridden,
        sync_mode_overridden: state.sync_mode_overridden,
    });

    let role = state
        .role
        .take()
        .ok_or(ConfigError::MissingArgumentValue("--role"))?;

    validate_runtime_mode_requirements(RuntimeModeValidationInputs {
        runtime_mode: state.runtime_mode,
        expected_state_version: state.expected_state_version,
        expected_state_hash: state.expected_state_hash.as_deref(),
        proposals_len: state.proposals.len(),
        rejoin_attempts_len: state.rejoin_attempts.len(),
        daemon_max_ticks: state.daemon_max_ticks,
        daemon_tick_interval_ms: state.daemon_tick_interval_ms,
        daemon_shutdown_signal_ticks_len: state.daemon_shutdown_signal_ticks.len(),
        daemon_shutdown_os_signals: state.daemon_shutdown_os_signals,
        daemon_shutdown_drain_ticks: state.daemon_shutdown_drain_ticks,
        daemon_shutdown_timeout_ticks: state.daemon_shutdown_timeout_ticks,
        daemon_peer_id_present: state.daemon_peer_id.is_some(),
        daemon_lifecycle_events_len: state.daemon_lifecycle_events.len(),
        api_bind_addr_present: state.api_bind_addr.is_some(),
        kolme_live_base_url: state.kolme_live_base_url.as_deref(),
        kolme_live_provider_hint: state.kolme_live_provider_hint.as_deref(),
        kolme_live_signing_profile: state.kolme_live_signing_profile.as_deref(),
        kolme_live_strict_signer_contracts: state.kolme_live_strict_signer_contracts,
        kolme_live_signer_profile: state.kolme_live_signer_profile.as_deref(),
        kolme_live_signer_key_source: state.kolme_live_signer_key_source.as_deref(),
    })?;

    validate_endpoint_guards(EndpointGuardInputs {
        api_bind_addr_present: state.api_bind_addr.is_some(),
        api_max_requests_overridden: state.api_max_requests_overridden,
        api_idle_timeout_ms_overridden: state.api_idle_timeout_ms_overridden,
        api_body_limit_bytes_overridden: state.api_body_limit_bytes_overridden,
        api_concurrency_limit_overridden: state.api_concurrency_limit_overridden,
        api_rate_limit_per_second_overridden: state.api_rate_limit_per_second_overridden,
        observability_endpoint_bind_addr_present: state.observability_endpoint_bind_addr.is_some(),
        observability_endpoint_metrics_path_overridden: state
            .observability_endpoint_metrics_path_overridden,
        observability_endpoint_health_path_overridden: state
            .observability_endpoint_health_path_overridden,
        observability_endpoint_max_requests_overridden: state
            .observability_endpoint_max_requests_overridden,
        observability_endpoint_idle_timeout_ms_overridden: state
            .observability_endpoint_idle_timeout_ms_overridden,
        observability_endpoint_metrics_path: state.observability_endpoint_metrics_path.as_str(),
        observability_endpoint_health_path: state.observability_endpoint_health_path.as_str(),
    })?;

    Ok(state.into_node_cli(role))
}
