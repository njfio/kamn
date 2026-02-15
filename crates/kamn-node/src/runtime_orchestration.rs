use super::*;

fn daemon_lifecycle_event_as_str(event: PeerLifecycleEvent) -> &'static str {
    match event {
        PeerLifecycleEvent::StartConnect => "start-connect",
        PeerLifecycleEvent::HandshakeSucceeded => "handshake-succeeded",
        PeerLifecycleEvent::HeartbeatMissed => "heartbeat-missed",
        PeerLifecycleEvent::HeartbeatRestored => "heartbeat-restored",
        PeerLifecycleEvent::Disconnect => "disconnect",
        PeerLifecycleEvent::Rejoin => "rejoin",
    }
}

fn peer_lifecycle_state_as_str(state: PeerLifecycleState) -> &'static str {
    match state {
        PeerLifecycleState::Disconnected => "disconnected",
        PeerLifecycleState::Connecting => "connecting",
        PeerLifecycleState::Active => "active",
        PeerLifecycleState::Degraded => "degraded",
    }
}

pub(crate) fn build_runtime_execution_id(
    runtime_mode: RuntimeMode,
    chain_id: &str,
    role: &str,
) -> String {
    format!("node-runtime:{}:{chain_id}:{role}", runtime_mode.as_str())
}

const RUNTIME_TRANSPORT_PROFILE_POLICY_STORE: &str = "runtime-transport-profile";
const RUNTIME_TRANSPORT_PROFILE_GOSSIP_DISABLED_FOR_PRODUCTION_REASON: &str =
    "runtime_transport_profile_gossip_disabled_for_production";
const RUNTIME_TRANSPORT_PROFILE_IN_MEMORY_FALLBACK_FORBIDDEN_REASON: &str =
    "runtime_transport_profile_in_memory_fallback_forbidden";
const RUNTIME_TRANSPORT_PROFILE_LIVE_MARKER_MISSING_REASON: &str =
    "runtime_transport_profile_live_marker_missing";
const RUNTIME_TRANSPORT_PROFILE_LIVE_PROVIDER_MISSING_REASON: &str =
    "runtime_transport_profile_live_provider_missing";

fn runtime_mode_requires_live_transport_profile(runtime_mode: RuntimeMode) -> bool {
    matches!(
        runtime_mode.kind,
        RuntimeModeKind::Daemon
            | RuntimeModeKind::Api
            | RuntimeModeKind::Full
            | RuntimeModeKind::KolmeLive
    )
}

pub(crate) fn select_runtime_transport_profile_for_runtime_mode(
    runtime_mode: RuntimeMode,
    enable_gossip: bool,
) -> Option<RuntimeTransportProfile> {
    if !enable_gossip {
        return None;
    }
    if runtime_mode_requires_live_transport_profile(runtime_mode) {
        return Some(RuntimeTransportProfile::Libp2pLive);
    }
    None
}

pub(crate) fn classify_production_transport_profile_violation(
    runtime_mode: RuntimeMode,
    enable_gossip: bool,
    components: &[String],
) -> Option<&'static str> {
    if !runtime_mode_requires_live_transport_profile(runtime_mode) {
        return None;
    }
    if !enable_gossip {
        return Some(RUNTIME_TRANSPORT_PROFILE_GOSSIP_DISABLED_FOR_PRODUCTION_REASON);
    }

    let has_component = |expected: &str| components.iter().any(|component| component == expected);
    if has_component("p2p-transport-profile:in-memory-deterministic")
        || has_component("p2p-in-memory-transport-fallback")
    {
        return Some(RUNTIME_TRANSPORT_PROFILE_IN_MEMORY_FALLBACK_FORBIDDEN_REASON);
    }
    if !has_component("p2p-transport-profile:libp2p-live") {
        return Some(RUNTIME_TRANSPORT_PROFILE_LIVE_MARKER_MISSING_REASON);
    }
    if !has_component("p2p-live-libp2p-provider") {
        return Some(RUNTIME_TRANSPORT_PROFILE_LIVE_PROVIDER_MISSING_REASON);
    }
    None
}

fn enforce_production_transport_profile_policy(
    runtime_mode: RuntimeMode,
    plan: &kamn_core::BootstrapPlan,
) -> Result<(), ConfigError> {
    let components = plan
        .wiring
        .all_components()
        .into_iter()
        .map(str::to_owned)
        .collect::<Vec<String>>();
    if let Some(reason_code) = classify_production_transport_profile_violation(
        runtime_mode,
        plan.config.enable_gossip,
        components.as_slice(),
    ) {
        return Err(ConfigError::RuntimeStoreCompatibility {
            store: RUNTIME_TRANSPORT_PROFILE_POLICY_STORE,
            reason_code,
            detail: format!(
                "runtime mode {} requires live libp2p transport profile wiring",
                runtime_mode.as_str()
            ),
        });
    }
    Ok(())
}

pub(crate) fn daemon_shutdown_drain_status(completion_reason: &str) -> &'static str {
    if completion_reason.starts_with("graceful-shutdown:signal@") {
        "completed"
    } else if completion_reason.starts_with("graceful-shutdown-timeout:signal@") {
        "timeout"
    } else {
        "not-signaled"
    }
}

pub(crate) fn daemon_shutdown_snapshot_flush_status(completion_reason: &str) -> &'static str {
    if completion_reason.starts_with("graceful-shutdown:signal@") {
        "snapshot-flushed"
    } else if completion_reason.starts_with("graceful-shutdown-timeout:signal@") {
        "snapshot-flush-timeout"
    } else {
        "snapshot-not-requested"
    }
}

pub(crate) fn daemon_shutdown_signal_tick(completion_reason: &str) -> Option<&str> {
    completion_reason
        .strip_prefix("graceful-shutdown:signal@")
        .or_else(|| completion_reason.strip_prefix("graceful-shutdown-timeout:signal@"))
        .map(|value| value.split(';').next().unwrap_or(value))
}

pub(crate) fn daemon_shutdown_reason_field<'a>(
    completion_reason: &'a str,
    key: &str,
) -> Option<&'a str> {
    completion_reason.split(';').find_map(|segment| {
        let (field, value) = segment.split_once('=')?;
        if field == key {
            return Some(value);
        }
        None
    })
}

pub(crate) fn classify_full_bootstrap_component_contract_violation(
    components: &[&str],
) -> Option<&'static str> {
    if components.len() != FULL_RUNTIME_BOOTSTRAP_COMPONENT_SEQUENCE.len() {
        return Some("full_supervisor_bootstrap_component_count_mismatch");
    }
    for (component, expected_component) in components
        .iter()
        .zip(FULL_RUNTIME_BOOTSTRAP_COMPONENT_SEQUENCE.iter())
    {
        if component != expected_component {
            return Some("full_supervisor_bootstrap_component_order_mismatch");
        }
    }
    None
}

pub(crate) fn validate_full_bootstrap_component_contract(
    components: &[&str],
) -> Result<(), ConfigError> {
    if let Some(reason) = classify_full_bootstrap_component_contract_violation(components) {
        return Err(ConfigError::RuntimeDaemonLifecycle(format!(
            "full_supervisor_invariant_violation:{reason}"
        )));
    }
    Ok(())
}

pub(crate) fn classify_full_supervisor_stop_contract_violation(
    completion_reason: &str,
    shutdown_drain_status: &str,
    shutdown_snapshot_flush_status: &str,
) -> Option<&'static str> {
    if !matches!(
        shutdown_drain_status,
        "completed" | "timeout" | "not-signaled"
    ) {
        return Some("full_supervisor_stop_invalid_shutdown_drain_status");
    }
    if !matches!(
        shutdown_snapshot_flush_status,
        "snapshot-flushed" | "snapshot-flush-timeout" | "snapshot-not-requested"
    ) {
        return Some("full_supervisor_stop_invalid_shutdown_snapshot_flush_status");
    }
    if completion_reason == "tick-budget-exhausted"
        || completion_reason.starts_with("tick-budget-exhausted;ignored_signals=")
    {
        if shutdown_drain_status != "not-signaled" {
            return Some("full_supervisor_stop_not_signaled_status_mismatch");
        }
        if shutdown_snapshot_flush_status != "snapshot-not-requested" {
            return Some("full_supervisor_stop_not_signaled_snapshot_flush_mismatch");
        }
        return None;
    }
    if completion_reason.starts_with("graceful-shutdown:signal@") {
        if daemon_shutdown_signal_tick(completion_reason).is_none() {
            return Some("full_supervisor_stop_missing_signal_tick");
        }
        if daemon_shutdown_reason_field(completion_reason, "drain_ticks").is_none() {
            return Some("full_supervisor_stop_missing_drain_ticks");
        }
        if daemon_shutdown_reason_field(completion_reason, "timeout_ticks").is_none() {
            return Some("full_supervisor_stop_missing_timeout_ticks");
        }
        if daemon_shutdown_reason_field(completion_reason, "ignored_signals").is_none() {
            return Some("full_supervisor_stop_missing_ignored_signals");
        }
        if shutdown_drain_status != "completed" {
            return Some("full_supervisor_stop_graceful_status_mismatch");
        }
        if shutdown_snapshot_flush_status != "snapshot-flushed" {
            return Some("full_supervisor_stop_graceful_snapshot_flush_status_mismatch");
        }
        return None;
    }
    if completion_reason.starts_with("graceful-shutdown-timeout:signal@") {
        if daemon_shutdown_signal_tick(completion_reason).is_none() {
            return Some("full_supervisor_stop_missing_signal_tick");
        }
        if daemon_shutdown_reason_field(completion_reason, "drain_ticks").is_none() {
            return Some("full_supervisor_stop_missing_drain_ticks");
        }
        if daemon_shutdown_reason_field(completion_reason, "timeout_ticks").is_none() {
            return Some("full_supervisor_stop_missing_timeout_ticks");
        }
        if daemon_shutdown_reason_field(completion_reason, "ignored_signals").is_none() {
            return Some("full_supervisor_stop_missing_ignored_signals");
        }
        if shutdown_drain_status != "timeout" {
            return Some("full_supervisor_stop_graceful_timeout_status_mismatch");
        }
        if shutdown_snapshot_flush_status != "snapshot-flush-timeout" {
            return Some("full_supervisor_stop_graceful_timeout_snapshot_flush_status_mismatch");
        }
        return None;
    }
    Some("full_supervisor_stop_unknown_completion_reason")
}

pub(crate) fn validate_full_supervisor_stop_contract(
    completion_reason: &str,
    shutdown_drain_status: &str,
    shutdown_snapshot_flush_status: &str,
) -> Result<(), ConfigError> {
    if let Some(reason) = classify_full_supervisor_stop_contract_violation(
        completion_reason,
        shutdown_drain_status,
        shutdown_snapshot_flush_status,
    ) {
        return Err(ConfigError::RuntimeDaemonLifecycle(format!(
            "full_supervisor_invariant_violation:{reason}"
        )));
    }
    Ok(())
}

fn execute_daemon_runtime(
    runtime_mode: RuntimeMode,
    execution_id: &str,
    options: DaemonRuntimeOptions,
) -> Result<DaemonExecution, ConfigError> {
    let DaemonRuntimeOptions {
        daemon_max_ticks,
        daemon_tick_interval_ms,
        daemon_shutdown_signal_ticks,
        daemon_shutdown_os_signals,
        daemon_shutdown_drain_ticks,
        daemon_shutdown_timeout_ticks,
        daemon_peer_id,
        daemon_lifecycle_events,
    } = options;
    let max_ticks =
        daemon_max_ticks.ok_or(ConfigError::MissingArgumentValue("--daemon-max-ticks"))?;
    let tick_interval_ms = daemon_tick_interval_ms.ok_or(ConfigError::MissingArgumentValue(
        "--daemon-tick-interval-ms",
    ))?;
    let max_ticks_label = max_ticks.to_string();
    let tick_interval_ms_label = tick_interval_ms.to_string();
    log_info(
        "node.runtime.daemon.execute.start",
        &[
            ("runtime_mode", runtime_mode.as_str()),
            ("max_ticks", max_ticks_label.as_str()),
            ("tick_interval_ms", tick_interval_ms_label.as_str()),
            ("execution_id", execution_id),
        ],
    )?;
    let (peer_id, peer_lifecycle_final_state, peer_lifecycle_applied_events) = match daemon_peer_id
    {
        Some(peer_id) => {
            let mut lifecycle = PeerLifecycle::new(&peer_id)
                .map_err(|error| ConfigError::RuntimeDaemonLifecycle(error.to_string()))?;
            let mut applied_events = Vec::with_capacity(daemon_lifecycle_events.len());
            for event in daemon_lifecycle_events {
                lifecycle
                    .transition(event)
                    .map_err(|error| ConfigError::RuntimeDaemonLifecycle(error.to_string()))?;
                applied_events.push(daemon_lifecycle_event_as_str(event).to_owned());
            }
            (
                Some(peer_id),
                Some(peer_lifecycle_state_as_str(lifecycle.state()).to_owned()),
                Some(applied_events),
            )
        }
        None => (None, None, None),
    };
    let daemon_completion = if daemon_shutdown_os_signals && daemon_shutdown_signal_ticks.is_empty()
    {
        evaluate_daemon_completion_with_os_signals(
            max_ticks,
            tick_interval_ms,
            daemon_shutdown_drain_ticks,
            daemon_shutdown_timeout_ticks,
        )
        .map_err(|error| ConfigError::RuntimeDaemonLifecycle(error.to_string()))?
    } else {
        evaluate_daemon_completion(
            max_ticks,
            daemon_shutdown_signal_ticks.as_slice(),
            daemon_shutdown_drain_ticks,
            daemon_shutdown_timeout_ticks,
        )
    };
    let daemon_observability = build_daemon_observability_telemetry(
        tick_interval_ms,
        daemon_completion.completion_reason.as_str(),
    )
    .map_err(|error| ConfigError::RuntimeDaemonLifecycle(error.to_string()))?;
    let shutdown_drain_status =
        daemon_shutdown_drain_status(daemon_completion.completion_reason.as_str());
    let shutdown_snapshot_flush_status =
        daemon_shutdown_snapshot_flush_status(daemon_completion.completion_reason.as_str());
    let shutdown_signal_tick =
        daemon_shutdown_signal_tick(daemon_completion.completion_reason.as_str()).unwrap_or("none");
    let shutdown_drain_ticks =
        daemon_shutdown_reason_field(daemon_completion.completion_reason.as_str(), "drain_ticks")
            .unwrap_or("0");
    let shutdown_timeout_ticks = daemon_shutdown_reason_field(
        daemon_completion.completion_reason.as_str(),
        "timeout_ticks",
    )
    .unwrap_or("0");
    let shutdown_ignored_signals = daemon_shutdown_reason_field(
        daemon_completion.completion_reason.as_str(),
        "ignored_signals",
    )
    .unwrap_or("0");
    let executed_ticks_label = daemon_completion.executed_ticks.to_string();
    log_info(
        "node.runtime.daemon.execute.complete",
        &[
            ("runtime_mode", runtime_mode.as_str()),
            ("executed_ticks", executed_ticks_label.as_str()),
            (
                "completion_reason",
                daemon_completion.completion_reason.as_str(),
            ),
            ("shutdown_drain_status", shutdown_drain_status),
            (
                "shutdown_snapshot_flush_status",
                shutdown_snapshot_flush_status,
            ),
            ("shutdown_signal_tick", shutdown_signal_tick),
            ("shutdown_drain_ticks", shutdown_drain_ticks),
            ("shutdown_timeout_ticks", shutdown_timeout_ticks),
            ("shutdown_ignored_signals", shutdown_ignored_signals),
            ("execution_id", execution_id),
        ],
    )?;
    Ok(DaemonExecution {
        max_ticks,
        tick_interval_ms,
        executed_ticks: daemon_completion.executed_ticks,
        completion_reason: daemon_completion.completion_reason,
        observability_latency_p50_ms: daemon_observability.latency_p50_ms,
        observability_latency_p99_ms: daemon_observability.latency_p99_ms,
        observability_throughput_tps: daemon_observability.throughput_tps,
        observability_error_rate_bps: daemon_observability.error_rate_bps,
        observability_availability_bps: daemon_observability.availability_bps,
        observability_health: daemon_observability.health,
        observability_alert_count: daemon_observability.alert_count,
        observability_reason_code: daemon_observability.reason_code,
        observability_transport_checkpoint_failures: daemon_observability
            .transport_checkpoint_failures,
        observability_signer_checkpoint_failures: daemon_observability.signer_checkpoint_failures,
        observability_commit_checkpoint_failures: daemon_observability.commit_checkpoint_failures,
        peer_id,
        peer_lifecycle_final_state,
        peer_lifecycle_applied_events,
    })
}

pub(crate) fn resolve_kolme_live_allow_local_signer_testing_override() -> Result<bool, ConfigError>
{
    match env::var(KOLME_LIVE_ALLOW_LOCAL_SIGNER_TESTING_ENV) {
        Ok(value) => {
            let trimmed = value.trim();
            if trimmed.is_empty() {
                return Err(ConfigError::RuntimeKolmeLive(format!(
                    "{KOLME_LIVE_ALLOW_LOCAL_SIGNER_TESTING_ENV} must not be empty when present (legacy_local_signer_path_override_invalid)"
                )));
            }
            match trimmed {
                "true" => Ok(true),
                "false" => Ok(false),
                _ => Err(ConfigError::RuntimeKolmeLive(format!(
                    "{KOLME_LIVE_ALLOW_LOCAL_SIGNER_TESTING_ENV} must be 'true' or 'false', found '{trimmed}' (legacy_local_signer_path_override_invalid)"
                ))),
            }
        }
        Err(env::VarError::NotPresent) => Ok(false),
        Err(env::VarError::NotUnicode(_)) => Err(ConfigError::RuntimeKolmeLive(format!(
            "{KOLME_LIVE_ALLOW_LOCAL_SIGNER_TESTING_ENV} must be valid utf-8 when present (legacy_local_signer_path_override_invalid)"
        ))),
    }
}

pub(crate) fn enforce_kolme_live_signer_contract_policy(
    strict_signer_contracts_enabled: bool,
    allow_local_signer_testing_override: bool,
    is_test_build: bool,
) -> Result<(), ConfigError> {
    if strict_signer_contracts_enabled || allow_local_signer_testing_override || is_test_build {
        return Ok(());
    }

    Err(ConfigError::RuntimeKolmeLive(format!(
        "--kolme-live-strict-signer-contracts is required for runtime-mode kolme-live; set {KOLME_LIVE_ALLOW_LOCAL_SIGNER_TESTING_ENV}=true only for explicit local testing override (legacy_local_signer_path_forbidden)"
    )))
}

pub(crate) fn classify_kolme_live_signer_key_source_policy_violation(
    strict_signer_contracts_enabled: bool,
    strict_signer_key_source: Option<&str>,
    allow_local_signer_testing_override: bool,
    is_test_build: bool,
) -> Option<&'static str> {
    if !strict_signer_contracts_enabled || allow_local_signer_testing_override || is_test_build {
        return None;
    }
    if strict_signer_key_source == Some(KOLME_LIVE_SIGNER_KEY_SOURCE_ENV_LOCAL) {
        return Some(KOLME_LIVE_ENV_LOCAL_SIGNER_KEY_SOURCE_FORBIDDEN_REASON_CODE);
    }
    None
}

pub(crate) fn enforce_kolme_live_signer_key_source_policy(
    strict_signer_contracts_enabled: bool,
    strict_signer_key_source: Option<&str>,
    allow_local_signer_testing_override: bool,
    is_test_build: bool,
) -> Result<(), ConfigError> {
    let reason_code = classify_kolme_live_signer_key_source_policy_violation(
        strict_signer_contracts_enabled,
        strict_signer_key_source,
        allow_local_signer_testing_override,
        is_test_build,
    );
    if let Some(reason_code) = reason_code {
        return Err(ConfigError::RuntimeKolmeLive(format!(
            "--kolme-live-signer-key-source={KOLME_LIVE_SIGNER_KEY_SOURCE_ENV_LOCAL} is not allowed when --kolme-live-strict-signer-contracts is enabled for production-targeted runs; use --kolme-live-signer-key-source={KOLME_LIVE_SIGNER_KEY_SOURCE_MANAGED_EXTERNAL} or set {KOLME_LIVE_ALLOW_LOCAL_SIGNER_TESTING_ENV}=true for explicit local testing override ({reason_code})"
        )));
    }
    Ok(())
}

pub(crate) fn execute(cli: NodeCli) -> Result<NodeBootstrapReport, ConfigError> {
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
        api_bind_addr: _,
        api_max_requests: _,
        api_idle_timeout_ms: _,
        api_body_limit_bytes: _,
        api_concurrency_limit: _,
        api_rate_limit_per_second: _,
        observability_endpoint_bind_addr: _,
        observability_endpoint_metrics_path: _,
        observability_endpoint_health_path: _,
        observability_endpoint_max_requests: _,
        observability_endpoint_idle_timeout_ms: _,
        output_mode: _,
        diagnostics_mode,
    } = cli;
    let execution_id = build_runtime_execution_id(runtime_mode, chain_id.as_str(), role.as_str());

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
                },
            )?;
            log_info(
                "node.runtime.full.bootstrap.ready",
                &[("execution_id", execution_id.as_str())],
            )?;
            let stop_reason = "daemon-execution-complete";
            let completion_reason = daemon_execution.completion_reason.as_str();
            let shutdown_drain_status = daemon_shutdown_drain_status(completion_reason);
            let shutdown_snapshot_flush_status =
                daemon_shutdown_snapshot_flush_status(completion_reason);
            validate_full_supervisor_stop_contract(
                completion_reason,
                shutdown_drain_status,
                shutdown_snapshot_flush_status,
            )?;
            log_info(
                "node.runtime.full.supervisor.stop.requested",
                &[
                    ("stop_reason", stop_reason),
                    ("daemon_completion_reason", completion_reason),
                    (
                        "shutdown_snapshot_flush_status",
                        shutdown_snapshot_flush_status,
                    ),
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
                    ("execution_id", execution_id.as_str()),
                ],
            )?;
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
