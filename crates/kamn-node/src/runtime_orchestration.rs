use super::*;

mod daemon_phase;
mod full_supervisor;
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
use daemon_phase::{
    daemon_shutdown_drain_status, daemon_shutdown_reason_field, daemon_shutdown_signal_tick,
    daemon_shutdown_snapshot_flush_status,
};
use full_supervisor::{
    build_full_supervisor_provisional_observability_snapshot,
    execute_full_supervisor_daemon_runtime, finish_full_supervisor_observability_lane,
    finish_full_supervisor_service_api_lane, full_supervisor_lane_idle_timeout_floor_ms,
    request_full_supervisor_lane_shutdown_probes, resolve_daemon_service_api_relay_spool_file,
    resolve_daemon_service_api_state_file, start_full_supervisor_observability_lane,
    start_full_supervisor_service_api_lane, FullSupervisorObservabilityLane,
    FullSupervisorServiceApiLane,
    FULL_SUPERVISOR_OBSERVABILITY_LANE_MAX_REQUESTS_CONTRACT_VIOLATION,
    FULL_SUPERVISOR_SERVICE_API_LANE_MAX_REQUESTS_CONTRACT_VIOLATION,
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
pub(crate) use runtime_policy_contracts::enforce_kolme_live_signer_contract_policy;
pub(crate) use runtime_policy_contracts::enforce_kolme_live_signer_key_source_policy;
pub(crate) use runtime_policy_contracts::resolve_kolme_live_allow_local_signer_testing_override;
pub(crate) use runtime_policy_contracts::select_runtime_transport_profile_for_runtime_mode;
pub(crate) use runtime_policy_contracts::should_use_os_signal_shutdown;
pub(crate) use runtime_policy_contracts::validate_full_supervisor_stop_contract;
pub(crate) use runtime_policy_contracts::validate_shutdown_checkpoint_reconciliation;
use runtime_policy_contracts::{
    enforce_production_transport_profile_policy, validate_full_bootstrap_component_contract,
};

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
            let service_api_state_file =
                resolve_daemon_service_api_state_file(api_bind_addr.as_deref())?;
            let service_api_relay_spool_file =
                resolve_daemon_service_api_relay_spool_file(service_api_state_file.as_deref())?;
            let daemon_execution = execute_daemon_runtime(
                runtime_mode,
                execution_id.as_str(),
                DaemonRuntimeOptions {
                    daemon_max_ticks,
                    daemon_tick_interval_ms,
                    daemon_shutdown_signal_ticks,
                    daemon_shutdown_os_signals,
                    daemon_shutdown_drain_ticks,
                    daemon_shutdown_timeout_ticks,
                    daemon_peer_id,
                    daemon_lifecycle_events,
                    service_api_state_file,
                    service_api_relay_spool_file,
                    service_api_signature_state_hash: service_api_signature_state_hash.clone(),
                },
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
        RuntimeModeKind::Full => {
            let full_supervisor_lane_idle_timeout_floor_ms =
                full_supervisor_lane_idle_timeout_floor_ms(
                    daemon_max_ticks,
                    daemon_tick_interval_ms,
                );
            validate_full_bootstrap_component_contract(
                FULL_RUNTIME_BOOTSTRAP_COMPONENT_SEQUENCE.as_slice(),
            )?;
            log_info(
                "node.runtime.full.bootstrap.start",
                &[("execution_id", execution_id.as_str())],
            )?;
            for (stage_index, component) in FULL_RUNTIME_BOOTSTRAP_COMPONENT_SEQUENCE
                .into_iter()
                .enumerate()
            {
                let stage_index_label = (stage_index + 1).to_string();
                log_info(
                    "node.runtime.full.bootstrap.component.ready",
                    &[
                        ("component", component),
                        ("stage_index", stage_index_label.as_str()),
                        ("execution_id", execution_id.as_str()),
                    ],
                )?;
            }
            let provisional_report = build_bootstrap_report(
                &plan,
                profile,
                diagnostics_mode,
                runtime_mode,
                RuntimeExecutionBundle::default(),
            );

            let mut service_api_lane: Option<FullSupervisorServiceApiLane> = None;
            if let Some(bind_addr) = api_bind_addr.as_ref() {
                if api_max_requests != 1 {
                    return Err(ConfigError::RuntimeDaemonLifecycle(format!(
                        "{FULL_SUPERVISOR_SERVICE_API_LANE_MAX_REQUESTS_CONTRACT_VIOLATION}:expected=1,actual={api_max_requests}"
                    )));
                }
                let lane_config = ServiceApiEndpointConfig {
                    bind_addr: bind_addr.clone(),
                    // Reserve request budget for startup probe, inter-tick probe, and shutdown probe.
                    max_requests: api_max_requests.saturating_add(2),
                    idle_timeout_ms: api_idle_timeout_ms
                        .max(full_supervisor_lane_idle_timeout_floor_ms),
                    body_limit_bytes: api_body_limit_bytes,
                    concurrency_limit: api_concurrency_limit,
                    rate_limit_per_second: api_rate_limit_per_second,
                };
                let lane_snapshot = build_service_api_snapshot(&provisional_report);
                service_api_lane = Some(start_full_supervisor_service_api_lane(
                    lane_config,
                    lane_snapshot,
                    execution_id.as_str(),
                )?);
            }

            let mut observability_lane: Option<FullSupervisorObservabilityLane> = None;
            if let Some(bind_addr) = observability_endpoint_bind_addr.as_ref() {
                if observability_endpoint_max_requests != 1 {
                    return Err(ConfigError::RuntimeDaemonLifecycle(format!(
                        "{FULL_SUPERVISOR_OBSERVABILITY_LANE_MAX_REQUESTS_CONTRACT_VIOLATION}:expected=1,actual={observability_endpoint_max_requests}"
                    )));
                }
                let lane_config = ObservabilityEndpointConfig {
                    bind_addr: bind_addr.clone(),
                    metrics_path: observability_endpoint_metrics_path.clone(),
                    health_path: observability_endpoint_health_path.clone(),
                    // Reserve request budget for startup probe, inter-tick probe, and shutdown probe.
                    max_requests: observability_endpoint_max_requests.saturating_add(2),
                    idle_timeout_ms: observability_endpoint_idle_timeout_ms
                        .max(full_supervisor_lane_idle_timeout_floor_ms),
                };
                let lane_snapshot = build_runtime_observability_snapshot(&provisional_report)
                    .unwrap_or_else(|| {
                        build_full_supervisor_provisional_observability_snapshot(runtime_mode)
                    });
                observability_lane = Some(start_full_supervisor_observability_lane(
                    lane_config,
                    lane_snapshot,
                    execution_id.as_str(),
                )?);
            }

            let service_api_state_file =
                resolve_daemon_service_api_state_file(api_bind_addr.as_deref())?;
            let service_api_relay_spool_file =
                resolve_daemon_service_api_relay_spool_file(service_api_state_file.as_deref())?;
            let daemon_execution = match execute_full_supervisor_daemon_runtime(
                runtime_mode,
                execution_id.as_str(),
                DaemonRuntimeOptions {
                    daemon_max_ticks,
                    daemon_tick_interval_ms,
                    daemon_shutdown_signal_ticks,
                    daemon_shutdown_os_signals,
                    daemon_shutdown_drain_ticks,
                    daemon_shutdown_timeout_ticks,
                    daemon_peer_id,
                    daemon_lifecycle_events,
                    service_api_state_file,
                    service_api_relay_spool_file,
                    service_api_signature_state_hash: service_api_signature_state_hash.clone(),
                },
                service_api_lane.as_ref(),
                observability_lane.as_ref(),
            ) {
                Ok(execution) => execution,
                Err(error) => {
                    if let Some(lane) = service_api_lane {
                        let _ =
                            finish_full_supervisor_service_api_lane(lane, execution_id.as_str());
                    }
                    if let Some(lane) = observability_lane {
                        let _ =
                            finish_full_supervisor_observability_lane(lane, execution_id.as_str());
                    }
                    return Err(error);
                }
            };
            log_info(
                "node.runtime.full.bootstrap.ready",
                &[("execution_id", execution_id.as_str())],
            )?;
            let stop_reason = "daemon-execution-complete";
            let completion_reason = daemon_execution.completion_reason.as_str();
            let shutdown_drain_status = daemon_shutdown_drain_status(completion_reason);
            let shutdown_snapshot_flush_status =
                daemon_shutdown_snapshot_flush_status(completion_reason);
            let shutdown_signal_tick =
                daemon_shutdown_signal_tick(completion_reason).unwrap_or("none");
            let shutdown_drain_ticks =
                daemon_shutdown_reason_field(completion_reason, "drain_ticks").unwrap_or("0");
            let shutdown_timeout_ticks =
                daemon_shutdown_reason_field(completion_reason, "timeout_ticks").unwrap_or("0");
            let shutdown_ignored_signals =
                daemon_shutdown_reason_field(completion_reason, "ignored_signals").unwrap_or("0");
            let stop_contract_result = validate_full_supervisor_stop_contract(
                completion_reason,
                shutdown_drain_status,
                shutdown_snapshot_flush_status,
            );
            log_info(
                "node.runtime.full.supervisor.stop.requested",
                &[
                    ("stop_reason", stop_reason),
                    ("daemon_completion_reason", completion_reason),
                    (
                        "shutdown_snapshot_flush_status",
                        shutdown_snapshot_flush_status,
                    ),
                    ("shutdown_signal_tick", shutdown_signal_tick),
                    ("shutdown_drain_ticks", shutdown_drain_ticks),
                    ("shutdown_timeout_ticks", shutdown_timeout_ticks),
                    ("shutdown_ignored_signals", shutdown_ignored_signals),
                    ("execution_id", execution_id.as_str()),
                ],
            )?;
            log_info(
                "node.runtime.full.supervisor.stop.complete",
                &[
                    ("stop_reason", stop_reason),
                    ("daemon_completion_reason", completion_reason),
                    ("shutdown_drain_status", shutdown_drain_status),
                    (
                        "shutdown_snapshot_flush_status",
                        shutdown_snapshot_flush_status,
                    ),
                    ("shutdown_signal_tick", shutdown_signal_tick),
                    ("shutdown_drain_ticks", shutdown_drain_ticks),
                    ("shutdown_timeout_ticks", shutdown_timeout_ticks),
                    ("shutdown_ignored_signals", shutdown_ignored_signals),
                    ("execution_id", execution_id.as_str()),
                ],
            )?;
            request_full_supervisor_lane_shutdown_probes(
                service_api_lane.as_ref(),
                observability_lane.as_ref(),
            );
            let service_api_lane_result = if let Some(lane) = service_api_lane {
                finish_full_supervisor_service_api_lane(lane, execution_id.as_str())
            } else {
                Ok(())
            };
            let observability_lane_result = if let Some(lane) = observability_lane {
                finish_full_supervisor_observability_lane(lane, execution_id.as_str())
            } else {
                Ok(())
            };
            stop_contract_result?;
            service_api_lane_result?;
            observability_lane_result?;
            RuntimeExecutionBundle {
                daemon: Some(daemon_execution),
                ..RuntimeExecutionBundle::default()
            }
        }
        RuntimeModeKind::KolmeLive => {
            let base_url = kolme_live_base_url
                .ok_or(ConfigError::MissingArgumentValue("--kolme-live-base-url"))?;
            let provider_hint = kolme_live_provider_hint.ok_or(
                ConfigError::MissingArgumentValue("--kolme-live-provider-hint"),
            )?;
            let signing_profile = kolme_live_signing_profile.ok_or(
                ConfigError::MissingArgumentValue("--kolme-live-signing-profile"),
            )?;
            let allow_local_signer_testing_override =
                resolve_kolme_live_allow_local_signer_testing_override()?;
            enforce_kolme_live_signer_contract_policy(
                kolme_live_strict_signer_contracts,
                allow_local_signer_testing_override,
                cfg!(test),
            )?;
            let declared_signer_profile = kolme_live_signer_profile
                .as_deref()
                .map(normalize_kolme_live_signer_profile_selector)
                .transpose()?;
            let declared_signer_key_source = kolme_live_signer_key_source
                .as_deref()
                .map(normalize_kolme_live_signer_key_source)
                .transpose()?;
            let strict_signer_profile = if kolme_live_strict_signer_contracts {
                Some(
                    declared_signer_profile.ok_or(ConfigError::MissingArgumentValue(
                        "--kolme-live-signer-profile",
                    ))?,
                )
            } else {
                declared_signer_profile
            };
            let strict_signer_key_source = if kolme_live_strict_signer_contracts {
                Some(
                    declared_signer_key_source.ok_or(ConfigError::MissingArgumentValue(
                        "--kolme-live-signer-key-source",
                    ))?,
                )
            } else {
                declared_signer_key_source
            };
            enforce_kolme_live_signer_key_source_policy(
                kolme_live_strict_signer_contracts,
                strict_signer_key_source,
                allow_local_signer_testing_override,
                cfg!(test),
            )?;
            let _signer_preflight = enforce_kolme_live_signer_preflight(
                strict_signer_profile,
                strict_signer_key_source,
            )?;
            let kolme_live_execution =
                if daemon_max_ticks.is_some() || daemon_tick_interval_ms.is_some() {
                    let max_cycles = daemon_max_ticks
                        .ok_or(ConfigError::MissingArgumentValue("--daemon-max-ticks"))?;
                    let cycle_interval_ms = daemon_tick_interval_ms.ok_or(
                        ConfigError::MissingArgumentValue("--daemon-tick-interval-ms"),
                    )?;
                    execute_kolme_live_runtime_continuous(
                        &plan,
                        base_url,
                        provider_hint,
                        signing_profile,
                        strict_signer_profile,
                        strict_signer_key_source,
                        KolmeLiveContinuousMode {
                            max_cycles,
                            cycle_interval_ms,
                        },
                    )?
                } else {
                    execute_kolme_live_runtime(
                        &plan,
                        base_url,
                        provider_hint,
                        signing_profile,
                        strict_signer_profile,
                        strict_signer_key_source,
                    )?
                };
            RuntimeExecutionBundle {
                kolme_live: Some(kolme_live_execution),
                ..RuntimeExecutionBundle::default()
            }
        }
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
