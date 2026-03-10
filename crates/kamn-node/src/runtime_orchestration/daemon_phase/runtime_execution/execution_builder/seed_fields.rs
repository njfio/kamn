use super::defaults;
use super::DaemonPeerLifecycle;
use crate::daemon_observability::DaemonRuntimeProcessingTelemetry;
use crate::daemon_shutdown::DaemonCompletion;
use crate::runtime_models::DaemonExecution;

pub(super) fn populate_base_fields(
    execution: &mut DaemonExecution,
    max_ticks: u64,
    tick_interval_ms: u64,
    daemon_completion: DaemonCompletion,
    runtime_processing: DaemonRuntimeProcessingTelemetry,
) {
    execution.max_ticks = max_ticks;
    execution.tick_interval_ms = tick_interval_ms;
    execution.executed_ticks = daemon_completion.executed_ticks;
    execution.completion_reason = daemon_completion.completion_reason;
    execution.service_api_relay_drained_count = runtime_processing.relay_drained_count;
    execution.service_api_relay_projected_state_count =
        runtime_processing.relay_projected_state_count;
}

pub(super) fn populate_empty_fields(
    execution: &mut DaemonExecution,
    observability: defaults::EmptyObservabilityFields,
    projections: defaults::EmptyProjectionFields,
) {
    populate_empty_observability_fields(execution, observability);
    populate_empty_projection_fields(execution, projections);
}

pub(super) fn populate_peer_lifecycle_fields(
    execution: &mut DaemonExecution,
    peer_lifecycle: &DaemonPeerLifecycle,
) {
    execution.peer_id = peer_lifecycle.peer_id.clone();
    execution.peer_lifecycle_final_state = peer_lifecycle.final_state.clone();
    execution.peer_lifecycle_applied_events = peer_lifecycle.applied_events.clone();
}

fn populate_empty_observability_fields(
    execution: &mut DaemonExecution,
    observability: defaults::EmptyObservabilityFields,
) {
    execution.observability_latency_p50_ms = observability.latency_p50_ms;
    execution.observability_latency_p99_ms = observability.latency_p99_ms;
    execution.observability_throughput_tps = observability.throughput_tps;
    execution.observability_error_rate_bps = observability.error_rate_bps;
    execution.observability_availability_bps = observability.availability_bps;
    execution.observability_health = observability.health;
    execution.observability_alert_count = observability.alert_count;
    execution.observability_reason_code = observability.reason_code;
    execution.observability_transport_checkpoint_failures =
        observability.transport_checkpoint_failures;
    execution.observability_signer_checkpoint_failures = observability.signer_checkpoint_failures;
    execution.observability_commit_checkpoint_failures = observability.commit_checkpoint_failures;
}

fn populate_empty_projection_fields(
    execution: &mut DaemonExecution,
    projections: defaults::EmptyProjectionFields,
) {
    populate_empty_phase6_fields(execution, &projections);
    populate_empty_convergence_fields(execution, &projections);
    populate_empty_live_postgres_fields(execution, projections);
}

fn populate_empty_phase6_fields(
    execution: &mut DaemonExecution,
    projections: &defaults::EmptyProjectionFields,
) {
    execution.phase6_runtime_reason_taxonomy_version =
        projections.phase6_runtime_reason_taxonomy_version.clone();
    execution.phase6_runtime_reason_codes_csv = projections.phase6_runtime_reason_codes_csv.clone();
    execution.phase6_runtime_reason_code = projections.phase6_runtime_reason_code.clone();
    execution.phase6_runtime_total_cycles = projections.phase6_runtime_total_cycles;
    execution.phase6_runtime_executed_cycles = projections.phase6_runtime_executed_cycles;
    execution.phase6_runtime_deferred_cycles = projections.phase6_runtime_deferred_cycles;
    execution.phase6_runtime_fail_closed_cycles = projections.phase6_runtime_fail_closed_cycles;
}

fn populate_empty_convergence_fields(
    execution: &mut DaemonExecution,
    projections: &defaults::EmptyProjectionFields,
) {
    execution.convergence_reason_taxonomy_version =
        projections.convergence_reason_taxonomy_version.clone();
    execution.convergence_reason_codes_csv = projections.convergence_reason_codes_csv.clone();
    execution.convergence_decision = projections.convergence_decision.clone();
    execution.convergence_reason_code = projections.convergence_reason_code.clone();
    execution.convergence_schema_gate_passed = projections.convergence_schema_gate_passed;
    execution.convergence_error_path_gate_passed = projections.convergence_error_path_gate_passed;
    execution.convergence_concurrency_gate_passed = projections.convergence_concurrency_gate_passed;
    execution.convergence_performance_budget_gate_passed =
        projections.convergence_performance_budget_gate_passed;
    execution.convergence_cost_budget_gate_passed = projections.convergence_cost_budget_gate_passed;
}

fn populate_empty_live_postgres_fields(
    execution: &mut DaemonExecution,
    projections: defaults::EmptyProjectionFields,
) {
    execution.live_postgres_multi_host_execution_bundle_schema_version =
        projections.live_postgres_multi_host_execution_bundle_schema_version;
    execution.live_postgres_multi_host_execution_bundle_selector_prefix =
        projections.live_postgres_multi_host_execution_bundle_selector_prefix;
    execution.live_postgres_multi_host_execution_bundle_row_count =
        projections.live_postgres_multi_host_execution_bundle_row_count;
    execution.live_postgres_multi_host_execution_bundle_selector_rows_fingerprint =
        projections.live_postgres_multi_host_execution_bundle_selector_rows_fingerprint;
}
