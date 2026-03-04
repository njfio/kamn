use super::*;

pub(super) struct FullRuntimeModeExecutionContext {
    pub profile: Option<LocalProfile>,
    pub diagnostics_mode: DiagnosticsMode,
    pub runtime_mode: RuntimeMode,
    pub execution_id: String,
    pub daemon_runtime_options: DaemonRuntimeOptions,
    pub api_bind_addr: Option<String>,
    pub api_max_requests: u64,
    pub api_idle_timeout_ms: u64,
    pub api_body_limit_bytes: u64,
    pub api_concurrency_limit: u64,
    pub api_rate_limit_per_second: u64,
    pub observability_endpoint_bind_addr: Option<String>,
    pub observability_endpoint_metrics_path: String,
    pub observability_endpoint_health_path: String,
    pub observability_endpoint_max_requests: u64,
    pub observability_endpoint_idle_timeout_ms: u64,
}

pub(super) struct KolmeLiveRuntimeModeExecutionContext {
    pub daemon_max_ticks: Option<u64>,
    pub daemon_tick_interval_ms: Option<u64>,
    pub kolme_live_base_url: Option<String>,
    pub kolme_live_provider_hint: Option<String>,
    pub kolme_live_signing_profile: Option<String>,
    pub kolme_live_strict_signer_contracts: bool,
    pub kolme_live_signer_profile: Option<String>,
    pub kolme_live_signer_key_source: Option<String>,
}

fn shutdown_reason_field_or_default<'a>(completion_reason: &'a str, field: &str) -> &'a str {
    daemon_phase::daemon_shutdown_reason_field(completion_reason, field).unwrap_or("0")
}

fn finish_service_api_lane_if_present(
    lane: Option<full_supervisor::FullSupervisorServiceApiLane>,
    execution_id: &str,
) -> Result<(), ConfigError> {
    if let Some(lane) = lane {
        return full_supervisor::finish_full_supervisor_service_api_lane(lane, execution_id);
    }
    Ok(())
}

fn finish_observability_lane_if_present(
    lane: Option<full_supervisor::FullSupervisorObservabilityLane>,
    execution_id: &str,
) -> Result<(), ConfigError> {
    if let Some(lane) = lane {
        return full_supervisor::finish_full_supervisor_observability_lane(lane, execution_id);
    }
    Ok(())
}

pub(super) fn build_daemon_runtime_options(
    daemon_max_ticks: Option<u64>,
    daemon_tick_interval_ms: Option<u64>,
    daemon_shutdown_signal_ticks: Vec<u64>,
    daemon_shutdown_os_signals: bool,
    daemon_shutdown_drain_ticks: Option<u64>,
    daemon_shutdown_timeout_ticks: Option<u64>,
    daemon_peer_id: Option<String>,
    daemon_lifecycle_events: Vec<PeerLifecycleEvent>,
    api_bind_addr: Option<&str>,
    service_api_signature_state_hash: &str,
) -> Result<DaemonRuntimeOptions, ConfigError> {
    let service_api_state_file =
        full_supervisor::resolve_daemon_service_api_state_file(api_bind_addr)?;
    let service_api_relay_spool_file =
        full_supervisor::resolve_daemon_service_api_relay_spool_file(
            service_api_state_file.as_deref(),
        )?;

    Ok(DaemonRuntimeOptions {
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
        service_api_signature_state_hash: service_api_signature_state_hash.to_owned(),
    })
}

pub(super) fn execute_full_runtime_mode(
    plan: &kamn_core::BootstrapPlan,
    context: FullRuntimeModeExecutionContext,
) -> Result<RuntimeExecutionBundle, ConfigError> {
    let FullRuntimeModeExecutionContext {
        profile,
        diagnostics_mode,
        runtime_mode,
        execution_id,
        daemon_runtime_options,
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
    } = context;

    let full_supervisor_lane_idle_timeout_floor_ms =
        full_supervisor::full_supervisor_lane_idle_timeout_floor_ms(
            daemon_runtime_options.daemon_max_ticks,
            daemon_runtime_options.daemon_tick_interval_ms,
        );
    runtime_policy_contracts::validate_full_bootstrap_component_contract(
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
        plan,
        profile,
        diagnostics_mode,
        runtime_mode,
        RuntimeExecutionBundle::default(),
    );

    let mut service_api_lane: Option<full_supervisor::FullSupervisorServiceApiLane> = None;
    if let Some(bind_addr) = api_bind_addr.as_ref() {
        if api_max_requests != 1 {
            return Err(ConfigError::RuntimeDaemonLifecycle(format!(
                "{}:expected=1,actual={api_max_requests}",
                full_supervisor::FULL_SUPERVISOR_SERVICE_API_LANE_MAX_REQUESTS_CONTRACT_VIOLATION
            )));
        }
        let lane_config = ServiceApiEndpointConfig {
            bind_addr: bind_addr.clone(),
            // Reserve request budget for startup probe, inter-tick probe, and shutdown probe.
            max_requests: api_max_requests.saturating_add(2),
            idle_timeout_ms: api_idle_timeout_ms.max(full_supervisor_lane_idle_timeout_floor_ms),
            body_limit_bytes: api_body_limit_bytes,
            concurrency_limit: api_concurrency_limit,
            rate_limit_per_second: api_rate_limit_per_second,
        };
        let lane_snapshot = build_service_api_snapshot(&provisional_report);
        service_api_lane = Some(full_supervisor::start_full_supervisor_service_api_lane(
            lane_config,
            lane_snapshot,
            execution_id.as_str(),
        )?);
    }

    let mut observability_lane: Option<full_supervisor::FullSupervisorObservabilityLane> = None;
    if let Some(bind_addr) = observability_endpoint_bind_addr.as_ref() {
        if observability_endpoint_max_requests != 1 {
            return Err(ConfigError::RuntimeDaemonLifecycle(format!(
                "{}:expected=1,actual={observability_endpoint_max_requests}",
                full_supervisor::FULL_SUPERVISOR_OBSERVABILITY_LANE_MAX_REQUESTS_CONTRACT_VIOLATION
            )));
        }
        let lane_config = ObservabilityEndpointConfig {
            bind_addr: bind_addr.clone(),
            metrics_path: observability_endpoint_metrics_path,
            health_path: observability_endpoint_health_path,
            // Reserve request budget for startup probe, inter-tick probe, and shutdown probe.
            max_requests: observability_endpoint_max_requests.saturating_add(2),
            idle_timeout_ms: observability_endpoint_idle_timeout_ms
                .max(full_supervisor_lane_idle_timeout_floor_ms),
        };
        let lane_snapshot = build_runtime_observability_snapshot(&provisional_report)
            .unwrap_or_else(|| {
                full_supervisor::build_full_supervisor_provisional_observability_snapshot(
                    runtime_mode,
                )
            });
        observability_lane = Some(full_supervisor::start_full_supervisor_observability_lane(
            lane_config,
            lane_snapshot,
            execution_id.as_str(),
        )?);
    }

    let daemon_execution = match full_supervisor::execute_full_supervisor_daemon_runtime(
        runtime_mode,
        execution_id.as_str(),
        daemon_runtime_options,
        service_api_lane.as_ref(),
        observability_lane.as_ref(),
    ) {
        Ok(execution) => execution,
        Err(error) => {
            let _ = finish_service_api_lane_if_present(service_api_lane, execution_id.as_str());
            let _ = finish_observability_lane_if_present(observability_lane, execution_id.as_str());
            return Err(error);
        }
    };
    log_info(
        "node.runtime.full.bootstrap.ready",
        &[("execution_id", execution_id.as_str())],
    )?;
    let stop_reason = "daemon-execution-complete";
    let completion_reason = daemon_execution.completion_reason.as_str();
    let shutdown_drain_status = daemon_phase::daemon_shutdown_drain_status(completion_reason);
    let shutdown_snapshot_flush_status =
        daemon_phase::daemon_shutdown_snapshot_flush_status(completion_reason);
    let shutdown_signal_tick =
        daemon_phase::daemon_shutdown_signal_tick(completion_reason).unwrap_or("none");
    let shutdown_drain_ticks = shutdown_reason_field_or_default(completion_reason, "drain_ticks");
    let shutdown_timeout_ticks =
        shutdown_reason_field_or_default(completion_reason, "timeout_ticks");
    let shutdown_ignored_signals =
        shutdown_reason_field_or_default(completion_reason, "ignored_signals");
    let stop_contract_result = runtime_policy_contracts::validate_full_supervisor_stop_contract(
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
    full_supervisor::request_full_supervisor_lane_shutdown_probes(
        service_api_lane.as_ref(),
        observability_lane.as_ref(),
    );
    let service_api_lane_result =
        finish_service_api_lane_if_present(service_api_lane, execution_id.as_str());
    let observability_lane_result =
        finish_observability_lane_if_present(observability_lane, execution_id.as_str());
    stop_contract_result?;
    service_api_lane_result?;
    observability_lane_result?;

    Ok(RuntimeExecutionBundle {
        daemon: Some(daemon_execution),
        ..RuntimeExecutionBundle::default()
    })
}

pub(super) fn execute_kolme_live_runtime_mode(
    plan: &kamn_core::BootstrapPlan,
    context: KolmeLiveRuntimeModeExecutionContext,
) -> Result<RuntimeExecutionBundle, ConfigError> {
    let KolmeLiveRuntimeModeExecutionContext {
        daemon_max_ticks,
        daemon_tick_interval_ms,
        kolme_live_base_url,
        kolme_live_provider_hint,
        kolme_live_signing_profile,
        kolme_live_strict_signer_contracts,
        kolme_live_signer_profile,
        kolme_live_signer_key_source,
    } = context;

    let base_url =
        kolme_live_base_url.ok_or(ConfigError::MissingArgumentValue("--kolme-live-base-url"))?;
    let provider_hint = kolme_live_provider_hint.ok_or(ConfigError::MissingArgumentValue(
        "--kolme-live-provider-hint",
    ))?;
    let signing_profile = kolme_live_signing_profile.ok_or(ConfigError::MissingArgumentValue(
        "--kolme-live-signing-profile",
    ))?;
    let allow_local_signer_testing_override =
        runtime_policy_contracts::resolve_kolme_live_allow_local_signer_testing_override()?;
    runtime_policy_contracts::enforce_kolme_live_signer_contract_policy(
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
    runtime_policy_contracts::enforce_kolme_live_signer_key_source_policy(
        kolme_live_strict_signer_contracts,
        strict_signer_key_source,
        allow_local_signer_testing_override,
        cfg!(test),
    )?;
    let _signer_preflight =
        enforce_kolme_live_signer_preflight(strict_signer_profile, strict_signer_key_source)?;
    let kolme_live_execution = if daemon_max_ticks.is_some() || daemon_tick_interval_ms.is_some() {
        let max_cycles =
            daemon_max_ticks.ok_or(ConfigError::MissingArgumentValue("--daemon-max-ticks"))?;
        let cycle_interval_ms = daemon_tick_interval_ms.ok_or(
            ConfigError::MissingArgumentValue("--daemon-tick-interval-ms"),
        )?;
        execute_kolme_live_runtime_continuous(
            plan,
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
            plan,
            base_url,
            provider_hint,
            signing_profile,
            strict_signer_profile,
            strict_signer_key_source,
        )?
    };

    Ok(RuntimeExecutionBundle {
        kolme_live: Some(kolme_live_execution),
        ..RuntimeExecutionBundle::default()
    })
}
