use super::{
    cli_core_common_option_parsing::{try_parse_core_common_option, CoreCommonOptionState},
    cli_daemon_option_parsing::{try_parse_daemon_option, DaemonOptionState},
    cli_endpoint_option_parsing::{try_parse_endpoint_option, EndpointOptionState},
    cli_kolme_live_option_parsing::{try_parse_kolme_live_option, KolmeLiveOptionState},
    cli_parse_state::CliParseState,
    cli_planning_recovery_option_parsing::{
        try_parse_planning_recovery_option, PlanningRecoveryOptionState,
    },
    ConfigError,
};

pub(super) fn parse_layered_args_into_state(
    layered_args: Vec<String>,
    state: &mut CliParseState,
) -> Result<(), ConfigError> {
    let mut iter = layered_args.into_iter();
    let _bin = iter.next();

    while let Some(arg) = iter.next() {
        if try_parse_core_common_option(
            arg.as_str(),
            &mut iter,
            &mut CoreCommonOptionState {
                role: &mut state.role,
                profile: &mut state.profile,
                chain_id: &mut state.chain_id,
                chain_version: &mut state.chain_version,
                storage_dir: &mut state.storage_dir,
                enable_gossip: &mut state.enable_gossip,
                sync_mode: &mut state.sync_mode,
                runtime_mode: &mut state.runtime_mode,
                output_mode: &mut state.output_mode,
                diagnostics_mode: &mut state.diagnostics_mode,
                role_overridden: &mut state.role_overridden,
                chain_id_overridden: &mut state.chain_id_overridden,
                chain_version_overridden: &mut state.chain_version_overridden,
                storage_dir_overridden: &mut state.storage_dir_overridden,
                gossip_overridden: &mut state.gossip_overridden,
                sync_mode_overridden: &mut state.sync_mode_overridden,
            },
        )? {
            continue;
        }
        if try_parse_daemon_option(
            arg.as_str(),
            &mut iter,
            &mut DaemonOptionState {
                daemon_max_ticks: &mut state.daemon_max_ticks,
                daemon_tick_interval_ms: &mut state.daemon_tick_interval_ms,
                daemon_shutdown_signal_ticks: &mut state.daemon_shutdown_signal_ticks,
                daemon_shutdown_os_signals: &mut state.daemon_shutdown_os_signals,
                daemon_shutdown_drain_ticks: &mut state.daemon_shutdown_drain_ticks,
                daemon_shutdown_timeout_ticks: &mut state.daemon_shutdown_timeout_ticks,
                daemon_peer_id: &mut state.daemon_peer_id,
                daemon_lifecycle_events: &mut state.daemon_lifecycle_events,
            },
        )? {
            continue;
        }
        if try_parse_endpoint_option(
            arg.as_str(),
            &mut iter,
            &mut EndpointOptionState {
                api_bind_addr: &mut state.api_bind_addr,
                api_max_requests: &mut state.api_max_requests,
                api_idle_timeout_ms: &mut state.api_idle_timeout_ms,
                api_body_limit_bytes: &mut state.api_body_limit_bytes,
                api_concurrency_limit: &mut state.api_concurrency_limit,
                api_rate_limit_per_second: &mut state.api_rate_limit_per_second,
                observability_endpoint_bind_addr: &mut state.observability_endpoint_bind_addr,
                observability_endpoint_metrics_path: &mut state.observability_endpoint_metrics_path,
                observability_endpoint_health_path: &mut state.observability_endpoint_health_path,
                observability_endpoint_max_requests: &mut state.observability_endpoint_max_requests,
                observability_endpoint_idle_timeout_ms: &mut state
                    .observability_endpoint_idle_timeout_ms,
                api_max_requests_overridden: &mut state.api_max_requests_overridden,
                api_idle_timeout_ms_overridden: &mut state.api_idle_timeout_ms_overridden,
                api_body_limit_bytes_overridden: &mut state.api_body_limit_bytes_overridden,
                api_concurrency_limit_overridden: &mut state.api_concurrency_limit_overridden,
                api_rate_limit_per_second_overridden: &mut state
                    .api_rate_limit_per_second_overridden,
                observability_endpoint_metrics_path_overridden: &mut state
                    .observability_endpoint_metrics_path_overridden,
                observability_endpoint_health_path_overridden: &mut state
                    .observability_endpoint_health_path_overridden,
                observability_endpoint_max_requests_overridden: &mut state
                    .observability_endpoint_max_requests_overridden,
                observability_endpoint_idle_timeout_ms_overridden: &mut state
                    .observability_endpoint_idle_timeout_ms_overridden,
            },
        )? {
            continue;
        }
        if try_parse_kolme_live_option(
            arg.as_str(),
            &mut iter,
            &mut KolmeLiveOptionState {
                kolme_live_base_url: &mut state.kolme_live_base_url,
                kolme_live_provider_hint: &mut state.kolme_live_provider_hint,
                kolme_live_signing_profile: &mut state.kolme_live_signing_profile,
                kolme_live_strict_signer_contracts: &mut state.kolme_live_strict_signer_contracts,
                kolme_live_signer_profile: &mut state.kolme_live_signer_profile,
                kolme_live_signer_key_source: &mut state.kolme_live_signer_key_source,
            },
        )? {
            continue;
        }
        if try_parse_planning_recovery_option(
            arg.as_str(),
            &mut iter,
            &mut PlanningRecoveryOptionState {
                expected_state_version: &mut state.expected_state_version,
                expected_state_hash: &mut state.expected_state_hash,
                proposals: &mut state.proposals,
                rejoin_attempts: &mut state.rejoin_attempts,
            },
        )? {
            continue;
        }
        return Err(ConfigError::UnknownArgument(arg));
    }

    Ok(())
}
