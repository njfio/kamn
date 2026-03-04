use super::*;

mod daemon_phase;
mod full_supervisor;
mod runtime_mode_handlers;
mod runtime_policy_contracts;

#[cfg(test)]
pub(crate) use daemon_phase::execute_daemon_convergence_projection_for_test;
#[cfg(test)]
pub(crate) use daemon_phase::execute_daemon_phase6_runtime_projection_for_test;
use daemon_phase::execute_daemon_runtime;
#[cfg(test)]
pub(crate) use daemon_phase::live_postgres_multi_host_execution_bundle_row_count_for_test;
#[cfg(test)]
pub(crate) use daemon_phase::live_postgres_multi_host_execution_bundle_selector_rows_fingerprint_for_test;
#[cfg(test)]
pub(crate) use daemon_phase::live_postgres_multi_host_execution_bundle_selector_rows_for_test;
#[cfg(test)]
pub(crate) use daemon_phase::validate_live_postgres_selector_bundle_for_test;
use runtime_mode_handlers::{
    build_daemon_runtime_options, execute_full_runtime_mode, execute_kolme_live_runtime_mode,
    DaemonRuntimeOptionsContext, FullRuntimeModeExecutionContext,
    KolmeLiveRuntimeModeExecutionContext,
};
#[cfg(test)]
pub(crate) use runtime_policy_contracts::classify_full_bootstrap_component_contract_violation;
#[cfg(test)]
pub(crate) use runtime_policy_contracts::classify_full_supervisor_stop_contract_violation;
#[cfg(test)]
pub(crate) use runtime_policy_contracts::classify_kolme_live_signer_key_source_policy_violation;
#[cfg(test)]
pub(crate) use runtime_policy_contracts::classify_production_transport_profile_violation;
#[cfg(test)]
pub(crate) use runtime_policy_contracts::classify_shutdown_checkpoint_reconciliation_violation;
#[cfg(test)]
pub(crate) use runtime_policy_contracts::enforce_kolme_live_signer_contract_policy;
#[cfg(test)]
pub(crate) use runtime_policy_contracts::enforce_kolme_live_signer_key_source_policy;
use runtime_policy_contracts::enforce_production_transport_profile_policy;
#[cfg(test)]
pub(crate) use runtime_policy_contracts::resolve_kolme_live_allow_local_signer_testing_override;
pub(crate) use runtime_policy_contracts::select_runtime_transport_profile_for_runtime_mode;
pub(crate) use runtime_policy_contracts::should_use_os_signal_shutdown;
#[cfg(test)]
pub(crate) use runtime_policy_contracts::validate_full_supervisor_stop_contract;
pub(crate) use runtime_policy_contracts::validate_shutdown_checkpoint_reconciliation;

pub(crate) fn build_runtime_execution_id(
    runtime_mode: RuntimeMode,
    chain_id: &str,
    role: &str,
) -> String {
    format!("node-runtime:{}:{chain_id}:{role}", runtime_mode.as_str())
}

pub(crate) fn execute(cli: NodeCli) -> Result<NodeBootstrapReport, ConfigError> {
    initialize_log_config_from_env()?;
    let NodeCli {
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
        output_mode: _,
        diagnostics_mode,
    } = cli;
    let execution_id = build_runtime_execution_id(runtime_mode, chain_id.as_str(), role.as_str());
    let service_api_signature_state_hash = format!(
        "service-api:{}:{}",
        chain_id.as_str(),
        chain_version.as_str()
    );

    let config = NodeConfig {
        chain_id: chain_id.clone(),
        chain_version: chain_version.clone(),
        role,
        storage_dir: storage_dir.clone(),
        enable_gossip,
        sync_mode,
    };

    let selected_transport_profile =
        select_runtime_transport_profile_for_runtime_mode(runtime_mode, config.enable_gossip);
    let plan = match selected_transport_profile {
        Some(transport_profile) => bootstrap_with_transport_profile(config, transport_profile)?,
        None => bootstrap(config)?,
    };
    enforce_production_transport_profile_policy(runtime_mode, &plan)?;
    log_info(
        "node.runtime.mode.dispatch",
        &[
            ("runtime_mode", runtime_mode.as_str()),
            ("execution_id", execution_id.as_str()),
        ],
    )?;
    let runtime_execution = match runtime_mode.kind {
        RuntimeModeKind::Bootstrap => {
            log_info(
                "node.runtime.bootstrap.plan.ready",
                &[
                    ("chain_id", plan.config.chain_id.as_str()),
                    ("role", plan.config.role.as_str()),
                    ("execution_id", execution_id.as_str()),
                ],
            )?;
            RuntimeExecutionBundle::default()
        }
        RuntimeModeKind::Planning => {
            let expected_state_hash = expected_state_hash
                .ok_or(ConfigError::MissingArgumentValue("--expected-state-hash"))?;
            let planner = DeterministicProposalPlanner::new(&expected_state_hash);
            let proposal_plan = planner
                .plan(proposals)
                .map_err(|error| ConfigError::RuntimePlanner(error.to_string()))?;
            RuntimeExecutionBundle {
                planning: Some(PlanningExecution {
                    expected_state_hash,
                    candidate_count: proposal_plan.ordered_candidates().len(),
                    scheduled_candidate_ids: proposal_plan.ordered_candidate_ids(),
                }),
                ..RuntimeExecutionBundle::default()
            }
        }
        RuntimeModeKind::RecoveryCheck => {
            let expected_state_version = expected_state_version.ok_or(
                ConfigError::MissingArgumentValue("--expected-state-version"),
            )?;
            let expected_state_hash = expected_state_hash
                .ok_or(ConfigError::MissingArgumentValue("--expected-state-hash"))?;
            let mut guard = RecoveryRejoinGuard::new(expected_state_version, &expected_state_hash)
                .map_err(|error| ConfigError::RuntimeRecovery(error.to_string()))?;
            let mut decisions = Vec::with_capacity(rejoin_attempts.len());
            for attempt in rejoin_attempts {
                let decision = match guard
                    .evaluate(attempt)
                    .map_err(|error| ConfigError::RuntimeRecovery(error.to_string()))?
                {
                    RecoveryStatus::RejoinAccepted => "rejoin-accepted".to_owned(),
                    RecoveryStatus::CatchUpRequired {
                        from_version,
                        to_version,
                    } => {
                        format!("catch-up-required:{from_version}->{to_version}")
                    }
                };
                decisions.push(decision);
            }
            RuntimeExecutionBundle {
                recovery: Some(RecoveryExecution {
                    expected_state_version,
                    expected_state_hash,
                    attempt_count: decisions.len(),
                    decisions,
                }),
                ..RuntimeExecutionBundle::default()
            }
        }
        RuntimeModeKind::Daemon => {
            let daemon_runtime_options =
                build_daemon_runtime_options(DaemonRuntimeOptionsContext {
                    daemon_max_ticks,
                    daemon_tick_interval_ms,
                    daemon_shutdown_signal_ticks,
                    daemon_shutdown_os_signals,
                    daemon_shutdown_drain_ticks,
                    daemon_shutdown_timeout_ticks,
                    daemon_peer_id,
                    daemon_lifecycle_events,
                    api_bind_addr,
                    service_api_signature_state_hash,
                })?;
            let daemon_execution = execute_daemon_runtime(
                runtime_mode,
                execution_id.as_str(),
                daemon_runtime_options,
            )?;
            RuntimeExecutionBundle {
                daemon: Some(daemon_execution),
                ..RuntimeExecutionBundle::default()
            }
        }
        RuntimeModeKind::Api => {
            log_info(
                "node.runtime.service_api.mode.ready",
                &[
                    ("runtime_mode", runtime_mode.as_str()),
                    ("execution_id", execution_id.as_str()),
                ],
            )?;
            RuntimeExecutionBundle::default()
        }
        RuntimeModeKind::Full => execute_full_runtime_mode(
            &plan,
            FullRuntimeModeExecutionContext {
                profile,
                diagnostics_mode,
                runtime_mode,
                execution_id: execution_id.clone(),
                daemon_runtime_options: build_daemon_runtime_options(
                    DaemonRuntimeOptionsContext {
                        daemon_max_ticks,
                        daemon_tick_interval_ms,
                        daemon_shutdown_signal_ticks,
                        daemon_shutdown_os_signals,
                        daemon_shutdown_drain_ticks,
                        daemon_shutdown_timeout_ticks,
                        daemon_peer_id,
                        daemon_lifecycle_events,
                        api_bind_addr: api_bind_addr.clone(),
                        service_api_signature_state_hash,
                    },
                )?,
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
            },
        )?,
        RuntimeModeKind::KolmeLive => execute_kolme_live_runtime_mode(
            &plan,
            KolmeLiveRuntimeModeExecutionContext {
                daemon_max_ticks,
                daemon_tick_interval_ms,
                kolme_live_base_url,
                kolme_live_provider_hint,
                kolme_live_signing_profile,
                kolme_live_strict_signer_contracts,
                kolme_live_signer_profile,
                kolme_live_signer_key_source,
            },
        )?,
    };
    let report = build_bootstrap_report(
        &plan,
        profile,
        diagnostics_mode,
        runtime_mode,
        runtime_execution,
    );

    Ok(report)
}

#[cfg(test)]
#[path = "runtime_orchestration_tests.rs"]
mod tests;
