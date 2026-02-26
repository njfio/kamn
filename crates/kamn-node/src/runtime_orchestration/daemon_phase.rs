use super::*;
use kamn_core::{service_auth_sign_with_private_key_hex, SERVICE_AUTH_SIGNATURE_PRIVATE_KEY_ENV};
use std::collections::{BTreeMap, BTreeSet};
use std::env;
use std::io::{Read, Write};
use std::net::{SocketAddr, TcpStream};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

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
const DAEMON_LIVE_POSTGRES_SELECTOR_BUNDLE_DUPLICATE_ROWS_REASON_CODE: &str =
    "live_postgres_selector_bundle_duplicate_rows";
const DAEMON_LIVE_POSTGRES_SELECTOR_BUNDLE_PREFIX_VIOLATION_REASON_CODE: &str =
    "live_postgres_selector_bundle_prefix_violation";
const DAEMON_LIVE_POSTGRES_SELECTOR_BUNDLE_ROW_FORMAT_VIOLATION_REASON_CODE: &str =
    "live_postgres_selector_bundle_row_format_violation";
const DAEMON_LIVE_POSTGRES_SELECTOR_BUNDLE_ROW_ID_VIOLATION_REASON_CODE: &str =
    "live_postgres_selector_bundle_row_id_violation";
const DAEMON_LIVE_POSTGRES_SELECTOR_BUNDLE_ROW_COUNT_MISMATCH_REASON_CODE: &str =
    "live_postgres_selector_bundle_row_count_mismatch";
const SERVICE_API_RELAY_RECIPIENT_ROUTE_MAP_ENV: &str =
    "KAMN_SERVICE_API_RELAY_RECIPIENT_ROUTE_MAP_JSON";
const SERVICE_API_RELAY_FORWARD_PATH: &str = "/v1/messages/relay";
const SERVICE_API_RELAY_FORWARD_SCOPE: &str = "messages:write";
const SERVICE_API_RELAY_FORWARD_DEFAULT_SENDER_DID: &str = "kamn:did:agent:relay-daemon";
const SERVICE_API_RELAY_FORWARD_CONNECT_TIMEOUT_MS: u64 = 500;
const SERVICE_API_RELAY_FORWARD_IO_TIMEOUT_MS: u64 = 500;
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

fn daemon_live_postgres_multi_host_execution_bundle_row_ids() -> BTreeSet<&'static str> {
    DAEMON_LIVE_POSTGRES_MULTI_HOST_EXECUTION_BUNDLE_SELECTOR_ROWS
        .iter()
        .map(|(row_id, _)| *row_id)
        .collect()
}

fn deterministic_fnv1a64_hex(input: &str) -> String {
    const FNV_OFFSET_BASIS: u64 = 0xcbf29ce484222325;
    const FNV_PRIME: u64 = 0x00000100000001b3;
    let mut hash = FNV_OFFSET_BASIS;
    for byte in input.as_bytes() {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(FNV_PRIME);
    }
    format!("{hash:016x}")
}

fn project_live_postgres_multi_host_execution_bundle_selector_rows_fingerprint(
    rows: &[String],
) -> String {
    deterministic_fnv1a64_hex(&rows.join(","))
}

fn validate_live_postgres_selector_bundle(
    rows: &[String],
    expected_row_count: usize,
) -> Result<(), &'static str> {
    if rows.len() != expected_row_count {
        return Err(DAEMON_LIVE_POSTGRES_SELECTOR_BUNDLE_ROW_COUNT_MISMATCH_REASON_CODE);
    }

    let canonical_row_ids = daemon_live_postgres_multi_host_execution_bundle_row_ids();
    let mut dedupe = BTreeSet::new();
    for row in rows {
        if !dedupe.insert(row.as_str()) {
            return Err(DAEMON_LIVE_POSTGRES_SELECTOR_BUNDLE_DUPLICATE_ROWS_REASON_CODE);
        }

        let Some((row_id, selector)) = row.split_once("->") else {
            return Err(DAEMON_LIVE_POSTGRES_SELECTOR_BUNDLE_ROW_FORMAT_VIOLATION_REASON_CODE);
        };
        if row_id.is_empty() || selector.is_empty() || selector.contains("->") {
            return Err(DAEMON_LIVE_POSTGRES_SELECTOR_BUNDLE_ROW_FORMAT_VIOLATION_REASON_CODE);
        }
        if !canonical_row_ids.contains(row_id) {
            return Err(DAEMON_LIVE_POSTGRES_SELECTOR_BUNDLE_ROW_ID_VIOLATION_REASON_CODE);
        }
        if !selector.starts_with(DAEMON_LIVE_POSTGRES_MULTI_HOST_EXECUTION_BUNDLE_SELECTOR_PREFIX) {
            return Err(DAEMON_LIVE_POSTGRES_SELECTOR_BUNDLE_PREFIX_VIOLATION_REASON_CODE);
        }
    }

    Ok(())
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
pub(crate) fn live_postgres_multi_host_execution_bundle_selector_rows_fingerprint_for_test(
) -> String {
    let rows = project_live_postgres_multi_host_execution_bundle_selector_rows();
    project_live_postgres_multi_host_execution_bundle_selector_rows_fingerprint(rows.as_slice())
}

#[cfg(test)]
pub(crate) fn validate_live_postgres_selector_bundle_for_test(
    rows: &[String],
    expected_row_count: usize,
) -> Result<(), &'static str> {
    validate_live_postgres_selector_bundle(rows, expected_row_count)
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
        service_api_state_file,
        service_api_relay_spool_file,
        service_api_signature_state_hash,
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
    let runtime_processing = execute_daemon_service_api_relay_tick_loop(
        daemon_completion.executed_ticks,
        tick_interval_ms,
        service_api_state_file.as_deref(),
        service_api_relay_spool_file.as_deref(),
        service_api_signature_state_hash.as_str(),
    )?;
    let daemon_observability = build_daemon_observability_telemetry(
        tick_interval_ms,
        daemon_completion.completion_reason.as_str(),
        &runtime_processing,
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
    let multi_host_execution_bundle_selector_rows =
        project_live_postgres_multi_host_execution_bundle_selector_rows();
    if let Err(reason_code) = validate_live_postgres_selector_bundle(
        multi_host_execution_bundle_selector_rows.as_slice(),
        multi_host_execution_bundle_row_count,
    ) {
        return Err(ConfigError::RuntimeDaemonLifecycle(format!(
            "live_postgres_selector_bundle_validation_failed:{reason_code}"
        )));
    }
    let multi_host_execution_bundle_selector_rows_csv =
        multi_host_execution_bundle_selector_rows.join(",");
    let multi_host_execution_bundle_selector_rows_fingerprint =
        project_live_postgres_multi_host_execution_bundle_selector_rows_fingerprint(
            multi_host_execution_bundle_selector_rows.as_slice(),
        );
    let relay_drained_count_label = runtime_processing.relay_drained_count.to_string();
    let relay_projected_state_count_label =
        runtime_processing.relay_projected_state_count.to_string();
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
            (
                "multi_host_execution_bundle_selector_rows_csv",
                multi_host_execution_bundle_selector_rows_csv.as_str(),
            ),
            (
                "multi_host_execution_bundle_selector_rows_fingerprint",
                multi_host_execution_bundle_selector_rows_fingerprint.as_str(),
            ),
            (
                "service_api_relay_drained_count",
                relay_drained_count_label.as_str(),
            ),
            (
                "service_api_relay_projected_state_count",
                relay_projected_state_count_label.as_str(),
            ),
            ("execution_id", execution_id),
        ],
    )?;
    Ok(DaemonExecution {
        max_ticks,
        tick_interval_ms,
        executed_ticks: daemon_completion.executed_ticks,
        completion_reason: daemon_completion.completion_reason,
        service_api_relay_drained_count: runtime_processing.relay_drained_count,
        service_api_relay_projected_state_count: runtime_processing.relay_projected_state_count,
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
        live_postgres_multi_host_execution_bundle_selector_rows_fingerprint:
            multi_host_execution_bundle_selector_rows_fingerprint,
    })
}

fn execute_daemon_service_api_relay_tick_loop(
    executed_ticks: u64,
    tick_interval_ms: u64,
    service_api_state_file: Option<&str>,
    service_api_relay_spool_file: Option<&str>,
    service_api_signature_state_hash: &str,
) -> Result<crate::daemon_observability::DaemonRuntimeProcessingTelemetry, ConfigError> {
    let mut runtime_processing = crate::daemon_observability::DaemonRuntimeProcessingTelemetry {
        executed_ticks,
        ..crate::daemon_observability::DaemonRuntimeProcessingTelemetry::default()
    };
    let relay_enabled = service_api_relay_spool_file.is_some();
    if executed_ticks == 0 {
        return Ok(runtime_processing);
    }
    let relay_route_map = resolve_daemon_service_api_relay_recipient_route_map()?;
    let relay_forwarding_enabled = !relay_route_map.is_empty();
    let relay_signing_private_key_hex = if relay_forwarding_enabled {
        Some(resolve_daemon_service_api_auth_private_key_hex()?)
    } else {
        None
    };
    let mut relay_nonce_counter = initial_daemon_relay_nonce_counter();

    let tick_duration = Duration::from_millis(tick_interval_ms.max(1));
    for tick in 0..executed_ticks {
        let tick_started_at = Instant::now();
        if relay_enabled {
            let relay_entries = crate::service_api_endpoint::drain_service_api_relay_spool_entries(
                service_api_relay_spool_file,
            )
            .map_err(|error| ConfigError::RuntimeDaemonLifecycle(error.to_string()))?;
            runtime_processing.relay_drained_count = runtime_processing
                .relay_drained_count
                .saturating_add(relay_entries.len() as u64);
            let mut relay_message_ids = Vec::new();
            let mut failed_entries = Vec::new();
            for relay_entry in relay_entries {
                if !relay_forwarding_enabled {
                    relay_message_ids.push(relay_entry.message_id.clone());
                    continue;
                }
                let Some(signing_key_hex) = relay_signing_private_key_hex.as_deref() else {
                    return Err(ConfigError::RuntimeDaemonLifecycle(
                        "service api relay forwarding signer key was missing".to_owned(),
                    ));
                };
                match forward_service_api_relay_entry(
                    &relay_route_map,
                    &relay_entry,
                    service_api_signature_state_hash,
                    signing_key_hex,
                    &mut relay_nonce_counter,
                ) {
                    Ok(()) => relay_message_ids.push(relay_entry.message_id.clone()),
                    Err(error) => {
                        runtime_processing.processing_error_count =
                            runtime_processing.processing_error_count.saturating_add(1);
                        let error_message = error;
                        let queued_at_label = relay_entry.queued_at_unix.to_string();
                        log_info(
                            "node.runtime.daemon.relay.forward.failed",
                            &[
                                ("message_id", relay_entry.message_id.as_str()),
                                ("recipient_did", relay_entry.recipient_did.as_str()),
                                ("queued_at_unix", queued_at_label.as_str()),
                                ("error", error_message.as_str()),
                            ],
                        )
                        .map_err(|logging_error| {
                            ConfigError::RuntimeDaemonLifecycle(logging_error.to_string())
                        })?;
                        failed_entries.push(relay_entry);
                    }
                }
            }
            for relay_entry in failed_entries {
                crate::service_api_endpoint::append_service_api_relay_spool_entry(
                    service_api_relay_spool_file,
                    &relay_entry,
                )
                .map_err(|error| ConfigError::RuntimeDaemonLifecycle(error.to_string()))?;
            }
            let relay_projected_state_count =
                crate::service_api_endpoint::project_service_api_relayed_message_statuses(
                    service_api_state_file,
                    relay_message_ids.as_slice(),
                )
                .map_err(|error| ConfigError::RuntimeDaemonLifecycle(error.to_string()))?;
            runtime_processing.relay_projected_state_count = runtime_processing
                .relay_projected_state_count
                .saturating_add(relay_projected_state_count as u64);
        }
        let elapsed_ms = tick_started_at.elapsed().as_millis();
        runtime_processing
            .tick_processing_samples_ms
            .push((elapsed_ms.min(u128::from(u64::MAX)) as u64).max(1));

        if let Some(remaining_sleep) = daemon_tick_remaining_sleep_duration(
            tick,
            executed_ticks,
            tick_duration,
            tick_started_at.elapsed(),
        ) {
            std::thread::sleep(remaining_sleep);
            runtime_processing.tick_sleep_count =
                runtime_processing.tick_sleep_count.saturating_add(1);
        }
    }

    Ok(runtime_processing)
}

fn resolve_daemon_service_api_relay_recipient_route_map(
) -> Result<BTreeMap<String, String>, ConfigError> {
    let raw = match env::var(SERVICE_API_RELAY_RECIPIENT_ROUTE_MAP_ENV) {
        Ok(value) => value,
        Err(env::VarError::NotPresent) => return Ok(BTreeMap::new()),
        Err(env::VarError::NotUnicode(_)) => {
            return Err(ConfigError::RuntimeDaemonLifecycle(format!(
                "{SERVICE_API_RELAY_RECIPIENT_ROUTE_MAP_ENV} must be valid utf-8 when present"
            )));
        }
    };
    let normalized = raw.trim();
    if normalized.is_empty() {
        return Err(ConfigError::RuntimeDaemonLifecycle(format!(
            "{SERVICE_API_RELAY_RECIPIENT_ROUTE_MAP_ENV} must not be empty when present"
        )));
    }
    let parsed = serde_json::from_str::<BTreeMap<String, String>>(normalized).map_err(|error| {
        ConfigError::RuntimeDaemonLifecycle(format!(
            "{SERVICE_API_RELAY_RECIPIENT_ROUTE_MAP_ENV} must be a JSON object mapping recipient DID to relay address: {error}"
        ))
    })?;
    let mut routes = BTreeMap::new();
    for (recipient_did, relay_addr) in parsed {
        let normalized_recipient_did = recipient_did.trim();
        if normalized_recipient_did.is_empty() {
            return Err(ConfigError::RuntimeDaemonLifecycle(format!(
                "{SERVICE_API_RELAY_RECIPIENT_ROUTE_MAP_ENV} contains an empty recipient DID key"
            )));
        }
        let normalized_relay_addr = relay_addr.trim();
        if normalized_relay_addr.is_empty() {
            return Err(ConfigError::RuntimeDaemonLifecycle(format!(
                "{SERVICE_API_RELAY_RECIPIENT_ROUTE_MAP_ENV} contains an empty relay address value for recipient={normalized_recipient_did}"
            )));
        }
        routes.insert(
            normalized_recipient_did.to_owned(),
            normalized_relay_addr.to_owned(),
        );
    }
    Ok(routes)
}

fn resolve_daemon_service_api_auth_private_key_hex() -> Result<String, ConfigError> {
    match env::var(SERVICE_AUTH_SIGNATURE_PRIVATE_KEY_ENV) {
        Ok(value) => {
            let normalized = value.trim();
            if normalized.is_empty() {
                return Err(ConfigError::RuntimeDaemonLifecycle(format!(
                    "{SERVICE_AUTH_SIGNATURE_PRIVATE_KEY_ENV} must not be empty when relay forwarding is enabled"
                )));
            }
            Ok(normalized.to_owned())
        }
        Err(env::VarError::NotPresent) => Err(ConfigError::RuntimeDaemonLifecycle(format!(
            "{SERVICE_AUTH_SIGNATURE_PRIVATE_KEY_ENV} is required when {SERVICE_API_RELAY_RECIPIENT_ROUTE_MAP_ENV} is configured"
        ))),
        Err(env::VarError::NotUnicode(_)) => Err(ConfigError::RuntimeDaemonLifecycle(format!(
            "{SERVICE_AUTH_SIGNATURE_PRIVATE_KEY_ENV} must be valid utf-8 when present"
        ))),
    }
}

fn initial_daemon_relay_nonce_counter() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| {
            let micros = duration.as_micros().min(u128::from(u64::MAX));
            micros as u64
        })
        .unwrap_or(1)
}

fn forward_service_api_relay_entry(
    relay_route_map: &BTreeMap<String, String>,
    relay_entry: &crate::service_api_endpoint::ServiceApiRelaySpoolEntry,
    service_api_signature_state_hash: &str,
    signing_private_key_hex: &str,
    relay_nonce_counter: &mut u64,
) -> Result<(), String> {
    let relay_addr = relay_route_map
        .get(relay_entry.recipient_did.as_str())
        .ok_or_else(|| {
            format!(
                "relay recipient route missing for recipient_did={}",
                relay_entry.recipient_did
            )
        })?;
    let relay_payload = serde_json::json!({
        "message_id": relay_entry.message_id.as_str(),
        "sender_did": relay_entry.sender_did.as_deref(),
        "recipient_did": relay_entry.recipient_did.as_str(),
        "body": relay_entry.body.as_str(),
        "queued_at_unix": relay_entry.queued_at_unix,
    });
    let relay_payload_body = serde_json::to_string(&relay_payload)
        .map_err(|error| format!("relay payload serialization failed: {error}"))?;

    let sender_did = relay_payload
        .get("sender_did")
        .and_then(serde_json::Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or(SERVICE_API_RELAY_FORWARD_DEFAULT_SENDER_DID);
    *relay_nonce_counter = relay_nonce_counter.saturating_add(1);
    let relay_nonce = (*relay_nonce_counter).max(1);
    let signature = service_auth_sign_with_private_key_hex(
        sender_did,
        relay_nonce,
        service_api_signature_state_hash,
        relay_payload_body.as_str(),
        signing_private_key_hex,
    )
    .map_err(|error| format!("relay request signature generation failed: {error}"))?;

    let request = format!(
        "POST {SERVICE_API_RELAY_FORWARD_PATH} HTTP/1.1\r\nHost: {relay_addr}\r\nConnection: close\r\nContent-Type: application/json\r\nX-KAMN-Sender-DID: {sender_did}\r\nX-KAMN-Request-Nonce: {relay_nonce}\r\nX-KAMN-Request-Signature: {signature}\r\nX-KAMN-Authz-Scope: {SERVICE_API_RELAY_FORWARD_SCOPE}\r\nContent-Length: {}\r\n\r\n{}",
        relay_payload_body.len(),
        relay_payload_body
    );
    let relay_socket_addr = relay_addr.parse::<SocketAddr>().map_err(|error| {
        format!("relay recipient address parse failed: addr={relay_addr}: {error}")
    })?;
    let mut stream = TcpStream::connect_timeout(
        &relay_socket_addr,
        Duration::from_millis(SERVICE_API_RELAY_FORWARD_CONNECT_TIMEOUT_MS),
    )
    .map_err(|error| format!("relay recipient connect failed: addr={relay_addr}: {error}"))?;
    let timeout = Duration::from_millis(SERVICE_API_RELAY_FORWARD_IO_TIMEOUT_MS);
    stream
        .set_write_timeout(Some(timeout))
        .map_err(|error| format!("relay recipient write-timeout set failed: {error}"))?;
    stream
        .set_read_timeout(Some(timeout))
        .map_err(|error| format!("relay recipient read-timeout set failed: {error}"))?;
    stream
        .write_all(request.as_bytes())
        .map_err(|error| format!("relay request write failed: {error}"))?;
    stream
        .flush()
        .map_err(|error| format!("relay request flush failed: {error}"))?;
    let mut response_bytes = Vec::new();
    let mut buffer = [0_u8; 512];
    loop {
        let read_count = stream
            .read(&mut buffer)
            .map_err(|error| format!("relay response read failed: {error}"))?;
        if read_count == 0 {
            break;
        }
        response_bytes.extend_from_slice(&buffer[..read_count]);
        if response_bytes
            .windows(4)
            .any(|window| window == b"\r\n\r\n")
        {
            break;
        }
    }
    let response = String::from_utf8(response_bytes)
        .map_err(|error| format!("relay response utf-8 parse failed: {error}"))?;
    let status_line = response.lines().next().unwrap_or("");
    if status_line.starts_with("HTTP/1.1 200")
        || status_line.starts_with("HTTP/1.1 201")
        || status_line.starts_with("HTTP/1.1 202")
    {
        return Ok(());
    }
    Err(format!(
        "relay request returned non-success status: addr={relay_addr};status={status_line}"
    ))
}

fn daemon_tick_remaining_sleep_duration(
    tick: u64,
    executed_ticks: u64,
    tick_duration: Duration,
    elapsed: Duration,
) -> Option<Duration> {
    if tick + 1 >= executed_ticks {
        return None;
    }
    let remaining = tick_duration.checked_sub(elapsed)?;
    if remaining.is_zero() {
        return None;
    }
    Some(remaining)
}

#[cfg(test)]
mod tests {
    use super::{
        daemon_tick_remaining_sleep_duration, execute_daemon_service_api_relay_tick_loop,
        SERVICE_API_RELAY_RECIPIENT_ROUTE_MAP_ENV,
    };
    use std::env;
    use std::fs;
    use std::time::Duration;

    struct TestEnvGuard {
        key: &'static str,
        previous: Option<String>,
    }

    impl TestEnvGuard {
        fn set(key: &'static str, value: Option<&str>) -> Self {
            let previous = env::var(key).ok();
            match value {
                Some(value) => env::set_var(key, value),
                None => env::remove_var(key),
            }
            Self { key, previous }
        }
    }

    impl Drop for TestEnvGuard {
        fn drop(&mut self) {
            if let Some(previous) = self.previous.as_deref() {
                env::set_var(self.key, previous);
            } else {
                env::remove_var(self.key);
            }
        }
    }

    #[test]
    fn unit_daemon_relay_tick_loop_sleeps_between_ticks_when_interval_budget_remains() {
        let runtime_processing =
            execute_daemon_service_api_relay_tick_loop(3, 50, None, None, "service-api:test:v1")
                .expect("tick loop");
        assert_eq!(runtime_processing.executed_ticks, 3);
        assert_eq!(runtime_processing.tick_processing_samples_ms.len(), 3);
        assert_eq!(
            runtime_processing.tick_sleep_count, 2,
            "daemon tick loop must sleep exactly between ticks and never after last tick"
        );
    }

    #[test]
    fn regression_daemon_relay_tick_loop_single_tick_never_sleeps() {
        // Regression: #5895
        let runtime_processing =
            execute_daemon_service_api_relay_tick_loop(1, 50, None, None, "service-api:test:v1")
                .expect("tick loop");
        assert_eq!(
            runtime_processing.tick_sleep_count, 0,
            "single-tick daemon loop must not execute sleep branch"
        );
        assert_eq!(runtime_processing.tick_processing_samples_ms.len(), 1);
    }

    #[test]
    fn unit_daemon_tick_remaining_sleep_duration_contract_is_deterministic() {
        assert_eq!(
            daemon_tick_remaining_sleep_duration(
                0,
                3,
                Duration::from_millis(50),
                Duration::from_millis(20),
            ),
            Some(Duration::from_millis(30))
        );
        assert_eq!(
            daemon_tick_remaining_sleep_duration(
                1,
                3,
                Duration::from_millis(50),
                Duration::from_millis(50),
            ),
            None,
            "equal elapsed and tick duration must not emit zero-duration sleeps"
        );
        assert_eq!(
            daemon_tick_remaining_sleep_duration(
                1,
                3,
                Duration::from_millis(50),
                Duration::from_millis(60),
            ),
            None,
            "elapsed values over tick duration must not underflow remaining sleep"
        );
        assert_eq!(
            daemon_tick_remaining_sleep_duration(
                2,
                3,
                Duration::from_millis(50),
                Duration::from_millis(1),
            ),
            None,
            "last tick must not sleep"
        );
    }

    #[test]
    fn unit_daemon_relay_tick_loop_reports_deterministic_projection_counters() {
        let unique_suffix = format!(
            "{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("clock should be monotonic")
                .as_nanos()
        );
        let state_file = std::env::temp_dir().join(format!(
            "kamn-node-daemon-phase-projection-state-{unique_suffix}.json"
        ));
        let relay_spool_file = std::env::temp_dir().join(format!(
            "kamn-node-daemon-phase-projection-spool-{unique_suffix}.ndjson"
        ));
        fs::write(
            state_file.as_path(),
            r#"{
  "schema_version":"kamn.runtime.service-api-message-store.v2",
  "messages":{
    "msg-daemon-projection-unit-1":{
      "message_id":"msg-daemon-projection-unit-1",
      "status":"created",
      "channel_id":null,
      "sender_did":"kamn:did:agent:sender",
      "recipient_did":"kamn:did:agent:recipient",
      "body":"{\"message\":\"project\"}"
    }
  },
  "channel_messages":{},
  "tasks":{},
  "escrows":{}
}"#,
        )
        .expect("state file fixture should write");
        fs::write(
            relay_spool_file.as_path(),
            r#"{"message_id":"msg-daemon-projection-unit-1","sender_did":"kamn:did:agent:sender","recipient_did":"kamn:did:agent:recipient","body":"{\"message\":\"project\"}","queued_at_unix":1700000888}
"#,
        )
        .expect("relay spool fixture should write");
        let state_file_str = state_file.to_string_lossy().to_string();
        let relay_spool_file_str = relay_spool_file.to_string_lossy().to_string();
        let _route_guard = TestEnvGuard::set(SERVICE_API_RELAY_RECIPIENT_ROUTE_MAP_ENV, None);

        let runtime_processing = execute_daemon_service_api_relay_tick_loop(
            1,
            1,
            Some(state_file_str.as_str()),
            Some(relay_spool_file_str.as_str()),
            "service-api:kamn-devnet:v0.1.0",
        )
        .expect("daemon relay tick loop should project local relayed state");

        assert_eq!(runtime_processing.relay_drained_count, 1);
        assert_eq!(runtime_processing.relay_projected_state_count, 1);
        assert_eq!(runtime_processing.processing_error_count, 0);

        let state_payload =
            fs::read_to_string(state_file.as_path()).expect("state file should remain readable");
        let state_json: serde_json::Value =
            serde_json::from_str(state_payload.as_str()).expect("state payload should parse");
        assert_eq!(
            state_json["messages"]["msg-daemon-projection-unit-1"]["status"],
            "relayed"
        );

        let relay_payload = fs::read_to_string(relay_spool_file.as_path())
            .expect("relay spool file should remain readable");
        assert!(
            relay_payload.trim().is_empty(),
            "relay spool should be drained after deterministic projection"
        );

        let _ = fs::remove_file(state_file);
        let _ = fs::remove_file(relay_spool_file);
    }

    #[test]
    fn regression_daemon_relay_tick_loop_requeues_failed_cross_node_forward_entries() {
        // Regression: #5983
        const TEST_SERVICE_API_AUTH_PRIVATE_KEY_HEX: &str =
            "658c3528422eb527b4c108b8f6d1e5f629543c304ea49cf608c67794424291c4";
        let unique_suffix = format!(
            "{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("clock should be monotonic")
                .as_nanos()
        );
        let state_file = std::env::temp_dir().join(format!(
            "kamn-node-daemon-phase-failed-forward-state-{unique_suffix}.json"
        ));
        let relay_spool_file = std::env::temp_dir().join(format!(
            "kamn-node-daemon-phase-failed-forward-spool-{unique_suffix}.ndjson"
        ));
        fs::write(
            state_file.as_path(),
            r#"{
  "schema_version":"kamn.runtime.service-api-message-store.v2",
  "messages":{
    "msg-forward-failure-unit-1":{
      "message_id":"msg-forward-failure-unit-1",
      "status":"created",
      "channel_id":null,
      "sender_did":"kamn:did:agent:sender",
      "recipient_did":"kamn:did:agent:recipient",
      "body":"{\"message\":\"unreachable\"}"
    }
  },
  "channel_messages":{},
  "tasks":{},
  "escrows":{}
}"#,
        )
        .expect("state file fixture should write");
        fs::write(
            relay_spool_file.as_path(),
            r#"{"message_id":"msg-forward-failure-unit-1","sender_did":"kamn:did:agent:sender","recipient_did":"kamn:did:agent:recipient","body":"{\"message\":\"unreachable\"}","queued_at_unix":1700000999}
"#,
        )
        .expect("relay spool fixture should write");
        let state_file_str = state_file.to_string_lossy().to_string();
        let relay_spool_file_str = relay_spool_file.to_string_lossy().to_string();
        let _route_guard = TestEnvGuard::set(
            SERVICE_API_RELAY_RECIPIENT_ROUTE_MAP_ENV,
            Some(r#"{"kamn:did:agent:recipient":"127.0.0.1:9"}"#),
        );
        let _signer_guard = TestEnvGuard::set(
            "KAMN_SERVICE_API_AUTH_PRIVATE_KEY_HEX",
            Some(TEST_SERVICE_API_AUTH_PRIVATE_KEY_HEX),
        );

        let runtime_processing = execute_daemon_service_api_relay_tick_loop(
            1,
            1,
            Some(state_file_str.as_str()),
            Some(relay_spool_file_str.as_str()),
            "service-api:kamn-devnet:v0.1.0",
        )
        .expect("daemon relay tick loop should complete on forward failure");

        assert_eq!(runtime_processing.relay_drained_count, 1);
        assert_eq!(runtime_processing.relay_projected_state_count, 0);
        assert_eq!(runtime_processing.processing_error_count, 1);

        let state_payload =
            fs::read_to_string(state_file.as_path()).expect("state file should remain readable");
        let state_json: serde_json::Value =
            serde_json::from_str(state_payload.as_str()).expect("state payload should parse");
        assert_eq!(
            state_json["messages"]["msg-forward-failure-unit-1"]["status"],
            "created"
        );

        let relay_payload = fs::read_to_string(relay_spool_file.as_path())
            .expect("relay spool file should remain readable");
        assert!(relay_payload.contains("msg-forward-failure-unit-1"));

        let _ = fs::remove_file(state_file);
        let _ = fs::remove_file(relay_spool_file);
    }
}
