use super::*;
use std::collections::BTreeMap;

const DAEMON_PHASE6_RUNTIME_REASON_TAXONOMY_VERSION: &str =
    "kamn.runtime.daemon.phase6.reason-taxonomy.v1";
const DAEMON_PHASE6_RUNTIME_REASON_CODES_CSV: &str =
    "m10_phase6_scheduler_cycle_applied,m10_phase6_scheduler_cycle_deferred,m10_phase6_scheduler_signal_invalid,m10_phase6_execution_budget_due_candidates_exceeded";
const DAEMON_CONVERGENCE_REASON_TAXONOMY_VERSION: &str =
    "kamn.runtime.daemon.convergence.reason-taxonomy.v1";
const DAEMON_CONVERGENCE_REASON_CODES_CSV: &str =
    "convergence_promotion_gate_go,convergence_schema_drift_detected,convergence_error_path_drift_detected,convergence_concurrency_drift_detected,convergence_performance_budget_exceeded,convergence_cost_budget_exceeded";
const DAEMON_CONVERGENCE_DECISION_GO: &str = "go";
const DAEMON_CONVERGENCE_DECISION_NO_GO: &str = "no_go";
const DAEMON_CONVERGENCE_REASON_GO: &str = "convergence_promotion_gate_go";
const DAEMON_CONVERGENCE_REASON_SCHEMA_DRIFT: &str = "convergence_schema_drift_detected";
const DAEMON_CONVERGENCE_REASON_ERROR_PATH_DRIFT: &str = "convergence_error_path_drift_detected";
const DAEMON_CONVERGENCE_REASON_CONCURRENCY_DRIFT: &str = "convergence_concurrency_drift_detected";
const DAEMON_CONVERGENCE_REASON_PERFORMANCE_BUDGET: &str =
    "convergence_performance_budget_exceeded";
const DAEMON_CONVERGENCE_REASON_COST_BUDGET: &str = "convergence_cost_budget_exceeded";
const DAEMON_LIVE_POSTGRES_MULTI_HOST_EXECUTION_BUNDLE_SCHEMA_VERSION: &str =
    "kamn.runtime.daemon.phase6-live-postgres.multi-host-execution-bundle.v1";
const DAEMON_LIVE_POSTGRES_MULTI_HOST_EXECUTION_BUNDLE_SELECTOR_PREFIX: &str =
    "main_tests::daemon_tests::";
const DAEMON_LIVE_POSTGRES_MULTI_HOST_EXECUTION_BUNDLE_SELECTOR_ROWS: [(&str, &str); 6] = [
    (
        "b01_runtime_matrix_bundle",
        "integration_runtime_daemon_phase6_live_postgres_validation_slice_matrix_reasons_are_stable_across_repeated_runs",
    ),
    (
        "b02_parallel_lane_bundle",
        "integration_runtime_daemon_phase6_live_postgres_validation_slice_parallel_lane_fingerprint_schema_is_stable",
    ),
    (
        "b03_topology_mapping_bundle",
        "integration_runtime_daemon_phase6_live_postgres_validation_slice_parallel_lane_topology_scope_is_stable",
    ),
    (
        "b04_topology_coherence_bundle",
        "integration_runtime_daemon_phase6_live_postgres_validation_slice_parallel_lane_topology_host_mode_host_pair_lane_set_lane_id_bundle_coherence_is_stable",
    ),
    (
        "b05_fingerprint_stability_bundle",
        "integration_runtime_daemon_phase6_live_postgres_validation_slice_parallel_lane_topology_host_mode_host_pair_lane_set_lane_fingerprint_hash_order_normalization_digest_is_stable",
    ),
    (
        "b06_multi_host_execution_bundle",
        "integration_runtime_daemon_phase6_live_postgres_validation_slice_multi_host_execution_bundle_is_stable",
    ),
];

struct DaemonPhase6RuntimeProjection {
    reason_code: &'static str,
    total_cycles: u64,
    executed_cycles: u64,
    deferred_cycles: u64,
    fail_closed_cycles: u64,
}

struct DaemonConvergenceInput {
    schema_gate_passed: bool,
    error_path_gate_passed: bool,
    concurrency_gate_passed: bool,
    performance_budget_gate_passed: bool,
    cost_budget_gate_passed: bool,
}

struct DaemonConvergenceProjection {
    decision: &'static str,
    reason_code: &'static str,
    schema_gate_passed: bool,
    error_path_gate_passed: bool,
    concurrency_gate_passed: bool,
    performance_budget_gate_passed: bool,
    cost_budget_gate_passed: bool,
}

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

pub(super) fn daemon_shutdown_drain_status(completion_reason: &str) -> &'static str {
    if completion_reason.starts_with("graceful-shutdown:signal@") {
        "completed"
    } else if completion_reason.starts_with("graceful-shutdown-timeout:signal@") {
        "timeout"
    } else {
        "not-signaled"
    }
}

pub(super) fn daemon_shutdown_snapshot_flush_status(completion_reason: &str) -> &'static str {
    if completion_reason.starts_with("graceful-shutdown:signal@") {
        "snapshot-flushed"
    } else if completion_reason.starts_with("graceful-shutdown-timeout:signal@") {
        "snapshot-flush-timeout"
    } else {
        "snapshot-not-requested"
    }
}

pub(super) fn daemon_shutdown_signal_tick(completion_reason: &str) -> Option<&str> {
    completion_reason
        .strip_prefix("graceful-shutdown:signal@")
        .or_else(|| completion_reason.strip_prefix("graceful-shutdown-timeout:signal@"))
        .and_then(|value| value.split(';').next())
        .map(str::trim)
        .filter(|tick| !tick.is_empty() && tick.chars().all(|c| c.is_ascii_digit()))
}

pub(super) fn daemon_shutdown_reason_field<'a>(
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

fn execute_daemon_phase6_runtime_projection(
    _max_ticks: u64,
    tick_interval_ms: u64,
    has_shutdown_signal: bool,
    regressed_now_epoch_seconds: Option<u64>,
) -> Result<DaemonPhase6RuntimeProjection, ConfigError> {
    let owner_did = "kamn:did:owner:daemon-phase6";
    let mut m8_registry = kamn_core::DataLayerM8ComplianceRegistry::new();
    let mut m10_registry = kamn_core::DataLayerM10PartitionLifecycleRegistry::new();
    let mut partition_message_ids_by_month = BTreeMap::new();

    if !has_shutdown_signal {
        m10_registry
            .register_partition(kamn_core::DataLayerM10PartitionRecordInput {
                partition_month_id: 202401,
                all_messages_shredded: false,
            })
            .map_err(|error| ConfigError::RuntimeDaemonLifecycle(error.to_string()))?;

        for (message_id, created_at_epoch_seconds) in [
            ("daemon-phase6-message-a", 1_699_700_000_u64),
            ("daemon-phase6-message-b", 1_699_700_100_u64),
        ] {
            m8_registry
                .register_message(kamn_core::DataLayerM8MessageRecordInput {
                    owner_did: owner_did.to_owned(),
                    message_id: message_id.to_owned(),
                    created_at_epoch_seconds,
                    content_hash: format!("hash:{message_id}"),
                    hash_chain_prev: format!("prev:{message_id}"),
                    retention_class: kamn_core::DataLayerM8RetentionClass::Ephemeral,
                    retention_extension_seconds: 0,
                    wrapped_keys: vec![kamn_core::DataLayerM8WrappedCekInput {
                        recipient_did: "kamn:did:agent:daemon-phase6".to_owned(),
                        wrapped_cek: format!("cek:{message_id}"),
                    }],
                })
                .map_err(|error| ConfigError::RuntimeDaemonLifecycle(error.to_string()))?;
        }

        partition_message_ids_by_month.insert(
            202401,
            vec![
                "daemon-phase6-message-b".to_owned(),
                "daemon-phase6-message-a".to_owned(),
            ],
        );
    } else {
        m10_registry
            .register_partition(kamn_core::DataLayerM10PartitionRecordInput {
                partition_month_id: 202601,
                all_messages_shredded: false,
            })
            .map_err(|error| ConfigError::RuntimeDaemonLifecycle(error.to_string()))?;

        m8_registry
            .register_message(kamn_core::DataLayerM8MessageRecordInput {
                owner_did: owner_did.to_owned(),
                message_id: "daemon-phase6-deferred-message".to_owned(),
                created_at_epoch_seconds: 1_699_999_990,
                content_hash: "hash:daemon-phase6-deferred-message".to_owned(),
                hash_chain_prev: "prev:daemon-phase6-deferred-message".to_owned(),
                retention_class: kamn_core::DataLayerM8RetentionClass::Ephemeral,
                retention_extension_seconds: 0,
                wrapped_keys: vec![kamn_core::DataLayerM8WrappedCekInput {
                    recipient_did: "kamn:did:agent:daemon-phase6".to_owned(),
                    wrapped_cek: "cek:daemon-phase6-deferred-message".to_owned(),
                }],
            })
            .map_err(|error| ConfigError::RuntimeDaemonLifecycle(error.to_string()))?;

        partition_message_ids_by_month
            .insert(202601, vec!["daemon-phase6-deferred-message".to_owned()]);
    }

    let scheduler_policy = if has_shutdown_signal {
        kamn_core::DataLayerM10Phase6SchedulerPolicy {
            due_candidate_trigger_threshold: 2,
            max_tick_interval_seconds: 2_000_000_000,
        }
    } else {
        kamn_core::DataLayerM10Phase6SchedulerPolicy {
            due_candidate_trigger_threshold: 1,
            max_tick_interval_seconds: 60,
        }
    };
    let mut runtime = kamn_core::DataLayerM10Phase6SchedulerRuntime::new(
        scheduler_policy,
        kamn_core::DataLayerM10Phase6ExecutionTickBudget {
            max_due_candidates: 2,
            max_shredded_messages: 2,
            max_projection_reports: 1,
            max_archived_entries: 1,
        },
    )
    .map_err(|error| ConfigError::RuntimeDaemonLifecycle(error.to_string()))?;

    let now_epoch_seconds = if has_shutdown_signal {
        1_700_000_010_u64
    } else {
        1_700_000_000_u64 + tick_interval_ms.saturating_add(95)
    };
    let base_request = kamn_core::DataLayerM10Phase6ExecutionTickRequest {
        requester_owner_did: owner_did.to_owned(),
        owner_did: owner_did.to_owned(),
        now_epoch_seconds,
        shredded_at_epoch_seconds: 1_700_000_300,
        now_month_id: 202602,
        active_retention_months: 2,
        object_storage_prefix: "s3://kamn-archive/messages".to_owned(),
        partition_message_ids_by_month,
    };

    let cycle_report = runtime
        .run_cycle(&mut m8_registry, &mut m10_registry, base_request.clone())
        .map_err(|error| ConfigError::RuntimeDaemonLifecycle(error.to_string()))?;
    let mut reason_code = cycle_report.reason_code;
    if let Some(regressed_now_epoch_seconds) = regressed_now_epoch_seconds {
        let mut regressed_request = base_request;
        regressed_request.now_epoch_seconds = regressed_now_epoch_seconds;
        let _ = runtime.run_cycle(&mut m8_registry, &mut m10_registry, regressed_request);
        reason_code = runtime.state().last_reason_code;
    }

    let runtime_state = runtime.state();
    Ok(DaemonPhase6RuntimeProjection {
        reason_code,
        total_cycles: runtime_state.total_cycles,
        executed_cycles: runtime_state.executed_cycles,
        deferred_cycles: runtime_state.deferred_cycles,
        fail_closed_cycles: runtime_state.fail_closed_cycles,
    })
}

fn execute_daemon_convergence_projection(
    input: DaemonConvergenceInput,
) -> DaemonConvergenceProjection {
    let (decision, reason_code) = if !input.schema_gate_passed {
        (
            DAEMON_CONVERGENCE_DECISION_NO_GO,
            DAEMON_CONVERGENCE_REASON_SCHEMA_DRIFT,
        )
    } else if !input.error_path_gate_passed {
        (
            DAEMON_CONVERGENCE_DECISION_NO_GO,
            DAEMON_CONVERGENCE_REASON_ERROR_PATH_DRIFT,
        )
    } else if !input.concurrency_gate_passed {
        (
            DAEMON_CONVERGENCE_DECISION_NO_GO,
            DAEMON_CONVERGENCE_REASON_CONCURRENCY_DRIFT,
        )
    } else if !input.performance_budget_gate_passed {
        (
            DAEMON_CONVERGENCE_DECISION_NO_GO,
            DAEMON_CONVERGENCE_REASON_PERFORMANCE_BUDGET,
        )
    } else if !input.cost_budget_gate_passed {
        (
            DAEMON_CONVERGENCE_DECISION_NO_GO,
            DAEMON_CONVERGENCE_REASON_COST_BUDGET,
        )
    } else {
        (DAEMON_CONVERGENCE_DECISION_GO, DAEMON_CONVERGENCE_REASON_GO)
    };

    DaemonConvergenceProjection {
        decision,
        reason_code,
        schema_gate_passed: input.schema_gate_passed,
        error_path_gate_passed: input.error_path_gate_passed,
        concurrency_gate_passed: input.concurrency_gate_passed,
        performance_budget_gate_passed: input.performance_budget_gate_passed,
        cost_budget_gate_passed: input.cost_budget_gate_passed,
    }
}

#[cfg(test)]
fn project_live_postgres_multi_host_execution_bundle_selector_rows() -> Vec<String> {
    DAEMON_LIVE_POSTGRES_MULTI_HOST_EXECUTION_BUNDLE_SELECTOR_ROWS
        .iter()
        .map(|(row_id, row_suffix)| {
            format!(
                "{row_id}->{DAEMON_LIVE_POSTGRES_MULTI_HOST_EXECUTION_BUNDLE_SELECTOR_PREFIX}{row_suffix}"
            )
        })
        .collect()
}

fn daemon_live_postgres_multi_host_execution_bundle_row_count() -> usize {
    DAEMON_LIVE_POSTGRES_MULTI_HOST_EXECUTION_BUNDLE_SELECTOR_ROWS.len()
}

#[cfg(test)]
pub(crate) fn live_postgres_multi_host_execution_bundle_selector_rows_for_test() -> Vec<String> {
    project_live_postgres_multi_host_execution_bundle_selector_rows()
}

#[cfg(test)]
pub(crate) fn live_postgres_multi_host_execution_bundle_row_count_for_test() -> usize {
    daemon_live_postgres_multi_host_execution_bundle_row_count()
}

#[cfg(test)]
pub(crate) fn execute_daemon_phase6_runtime_projection_for_test(
    max_ticks: u64,
    tick_interval_ms: u64,
    has_shutdown_signal: bool,
    regressed_now_epoch_seconds: Option<u64>,
) -> Result<(&'static str, u64), ConfigError> {
    let projection = execute_daemon_phase6_runtime_projection(
        max_ticks,
        tick_interval_ms,
        has_shutdown_signal,
        regressed_now_epoch_seconds,
    )?;
    Ok((projection.reason_code, projection.fail_closed_cycles))
}

#[cfg(test)]
pub(crate) fn execute_daemon_convergence_projection_for_test(
    schema_gate_passed: bool,
    error_path_gate_passed: bool,
    concurrency_gate_passed: bool,
    performance_budget_gate_passed: bool,
    cost_budget_gate_passed: bool,
) -> (&'static str, &'static str) {
    let projection = execute_daemon_convergence_projection(DaemonConvergenceInput {
        schema_gate_passed,
        error_path_gate_passed,
        concurrency_gate_passed,
        performance_budget_gate_passed,
        cost_budget_gate_passed,
    });
    (projection.decision, projection.reason_code)
}

pub(super) fn execute_daemon_runtime(
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
    let daemon_completion = if should_use_os_signal_shutdown(
        runtime_mode,
        daemon_shutdown_os_signals,
        daemon_shutdown_signal_ticks.as_slice(),
    ) {
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
    validate_shutdown_checkpoint_reconciliation(
        daemon_completion.completion_reason.as_str(),
        daemon_observability.reason_code.as_str(),
        daemon_observability.transport_checkpoint_failures,
        daemon_observability.signer_checkpoint_failures,
        daemon_observability.commit_checkpoint_failures,
    )?;
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
    let phase6_projection = execute_daemon_phase6_runtime_projection(
        max_ticks,
        tick_interval_ms,
        !daemon_shutdown_signal_ticks.is_empty(),
        None,
    )?;
    let phase6_total_cycles_label = phase6_projection.total_cycles.to_string();
    let phase6_executed_cycles_label = phase6_projection.executed_cycles.to_string();
    let phase6_deferred_cycles_label = phase6_projection.deferred_cycles.to_string();
    let phase6_fail_closed_cycles_label = phase6_projection.fail_closed_cycles.to_string();
    let multi_host_execution_bundle_row_count =
        daemon_live_postgres_multi_host_execution_bundle_row_count();
    let multi_host_execution_bundle_row_count_label =
        multi_host_execution_bundle_row_count.to_string();
    let convergence_projection = execute_daemon_convergence_projection(DaemonConvergenceInput {
        schema_gate_passed: phase6_projection.total_cycles > 0
            && phase6_projection.reason_code != "m10_phase6_scheduler_signal_invalid",
        error_path_gate_passed: phase6_projection.fail_closed_cycles == 0,
        concurrency_gate_passed: phase6_projection.total_cycles
            == phase6_projection
                .executed_cycles
                .saturating_add(phase6_projection.deferred_cycles)
                .saturating_add(phase6_projection.fail_closed_cycles),
        performance_budget_gate_passed: daemon_observability.reason_code
            != "daemon_shutdown_timeout",
        cost_budget_gate_passed: max_ticks <= 10_000 && tick_interval_ms <= 5_000,
    });
    let convergence_schema_gate_passed = if convergence_projection.schema_gate_passed {
        "true"
    } else {
        "false"
    };
    let convergence_error_path_gate_passed = if convergence_projection.error_path_gate_passed {
        "true"
    } else {
        "false"
    };
    let convergence_concurrency_gate_passed = if convergence_projection.concurrency_gate_passed {
        "true"
    } else {
        "false"
    };
    let convergence_performance_budget_gate_passed =
        if convergence_projection.performance_budget_gate_passed {
            "true"
        } else {
            "false"
        };
    let convergence_cost_budget_gate_passed = if convergence_projection.cost_budget_gate_passed {
        "true"
    } else {
        "false"
    };
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
            (
                "phase6_reason_taxonomy_version",
                DAEMON_PHASE6_RUNTIME_REASON_TAXONOMY_VERSION,
            ),
            (
                "phase6_reason_codes_csv",
                DAEMON_PHASE6_RUNTIME_REASON_CODES_CSV,
            ),
            ("phase6_reason_code", phase6_projection.reason_code),
            ("phase6_total_cycles", phase6_total_cycles_label.as_str()),
            (
                "phase6_executed_cycles",
                phase6_executed_cycles_label.as_str(),
            ),
            (
                "phase6_deferred_cycles",
                phase6_deferred_cycles_label.as_str(),
            ),
            (
                "phase6_fail_closed_cycles",
                phase6_fail_closed_cycles_label.as_str(),
            ),
            (
                "convergence_reason_taxonomy_version",
                DAEMON_CONVERGENCE_REASON_TAXONOMY_VERSION,
            ),
            (
                "convergence_reason_codes_csv",
                DAEMON_CONVERGENCE_REASON_CODES_CSV,
            ),
            ("convergence_decision", convergence_projection.decision),
            (
                "convergence_reason_code",
                convergence_projection.reason_code,
            ),
            (
                "convergence_schema_gate_passed",
                convergence_schema_gate_passed,
            ),
            (
                "convergence_error_path_gate_passed",
                convergence_error_path_gate_passed,
            ),
            (
                "convergence_concurrency_gate_passed",
                convergence_concurrency_gate_passed,
            ),
            (
                "convergence_performance_budget_gate_passed",
                convergence_performance_budget_gate_passed,
            ),
            (
                "convergence_cost_budget_gate_passed",
                convergence_cost_budget_gate_passed,
            ),
            (
                "multi_host_execution_bundle_schema_version",
                DAEMON_LIVE_POSTGRES_MULTI_HOST_EXECUTION_BUNDLE_SCHEMA_VERSION,
            ),
            (
                "multi_host_execution_bundle_selector_prefix",
                DAEMON_LIVE_POSTGRES_MULTI_HOST_EXECUTION_BUNDLE_SELECTOR_PREFIX,
            ),
            (
                "multi_host_execution_bundle_row_count",
                multi_host_execution_bundle_row_count_label.as_str(),
            ),
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
        phase6_runtime_reason_taxonomy_version: DAEMON_PHASE6_RUNTIME_REASON_TAXONOMY_VERSION
            .to_owned(),
        phase6_runtime_reason_codes_csv: DAEMON_PHASE6_RUNTIME_REASON_CODES_CSV.to_owned(),
        phase6_runtime_reason_code: phase6_projection.reason_code.to_owned(),
        phase6_runtime_total_cycles: phase6_projection.total_cycles,
        phase6_runtime_executed_cycles: phase6_projection.executed_cycles,
        phase6_runtime_deferred_cycles: phase6_projection.deferred_cycles,
        phase6_runtime_fail_closed_cycles: phase6_projection.fail_closed_cycles,
        convergence_reason_taxonomy_version: DAEMON_CONVERGENCE_REASON_TAXONOMY_VERSION.to_owned(),
        convergence_reason_codes_csv: DAEMON_CONVERGENCE_REASON_CODES_CSV.to_owned(),
        convergence_decision: convergence_projection.decision.to_owned(),
        convergence_reason_code: convergence_projection.reason_code.to_owned(),
        convergence_schema_gate_passed: convergence_projection.schema_gate_passed,
        convergence_error_path_gate_passed: convergence_projection.error_path_gate_passed,
        convergence_concurrency_gate_passed: convergence_projection.concurrency_gate_passed,
        convergence_performance_budget_gate_passed: convergence_projection
            .performance_budget_gate_passed,
        convergence_cost_budget_gate_passed: convergence_projection.cost_budget_gate_passed,
        live_postgres_multi_host_execution_bundle_schema_version:
            DAEMON_LIVE_POSTGRES_MULTI_HOST_EXECUTION_BUNDLE_SCHEMA_VERSION.to_owned(),
        live_postgres_multi_host_execution_bundle_selector_prefix:
            DAEMON_LIVE_POSTGRES_MULTI_HOST_EXECUTION_BUNDLE_SELECTOR_PREFIX.to_owned(),
        live_postgres_multi_host_execution_bundle_row_count: multi_host_execution_bundle_row_count,
    })
}
