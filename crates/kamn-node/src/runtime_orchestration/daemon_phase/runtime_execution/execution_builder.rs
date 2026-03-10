mod defaults;
mod seed_fields;

use super::super::super::*;
use super::reporting::DaemonReportSnapshot;
use super::shutdown_fields::daemon_shutdown_drain_status;
use crate::daemon_shutdown::DaemonCompletion;
use defaults::{empty_observability_fields, empty_projection_fields};
use seed_fields::{populate_base_fields, populate_empty_fields, populate_peer_lifecycle_fields};

pub(super) struct DaemonPeerLifecycle {
    pub(super) peer_id: Option<String>,
    pub(super) final_state: Option<String>,
    pub(super) applied_events: Option<Vec<String>>,
}

pub(super) struct ExecutedDaemonRun {
    pub(super) daemon_completion: DaemonCompletion,
    pub(super) runtime_processing: crate::daemon_observability::DaemonRuntimeProcessingTelemetry,
    pub(super) daemon_observability: crate::daemon_observability::DaemonObservabilityTelemetry,
    pub(super) report: DaemonReportSnapshot,
}

pub(super) fn build_daemon_execution(
    max_ticks: u64,
    tick_interval_ms: u64,
    peer_lifecycle: DaemonPeerLifecycle,
    daemon_run: ExecutedDaemonRun,
) -> DaemonExecution {
    let ExecutedDaemonRun {
        daemon_completion,
        runtime_processing,
        daemon_observability,
        report,
    } = daemon_run;
    let _ = daemon_shutdown_drain_status(daemon_completion.completion_reason.as_str());
    let mut execution = base_daemon_execution(
        max_ticks,
        tick_interval_ms,
        &peer_lifecycle,
        daemon_completion,
        runtime_processing,
    );
    populate_observability_fields(&mut execution, daemon_observability);
    populate_projection_fields(&mut execution, &report);
    execution
}

fn base_daemon_execution(
    max_ticks: u64,
    tick_interval_ms: u64,
    peer_lifecycle: &DaemonPeerLifecycle,
    daemon_completion: DaemonCompletion,
    runtime_processing: crate::daemon_observability::DaemonRuntimeProcessingTelemetry,
) -> DaemonExecution {
    let observability = empty_observability_fields();
    let projections = empty_projection_fields();
    let mut execution = DaemonExecution::default();
    populate_base_fields(
        &mut execution,
        max_ticks,
        tick_interval_ms,
        daemon_completion,
        runtime_processing,
    );
    populate_empty_fields(&mut execution, observability, projections);
    populate_peer_lifecycle_fields(&mut execution, peer_lifecycle);
    execution
}

fn populate_observability_fields(
    execution: &mut DaemonExecution,
    daemon_observability: crate::daemon_observability::DaemonObservabilityTelemetry,
) {
    execution.observability_latency_p50_ms = daemon_observability.latency_p50_ms;
    execution.observability_latency_p99_ms = daemon_observability.latency_p99_ms;
    execution.observability_throughput_tps = daemon_observability.throughput_tps;
    execution.observability_error_rate_bps = daemon_observability.error_rate_bps;
    execution.observability_availability_bps = daemon_observability.availability_bps;
    execution.observability_health = daemon_observability.health;
    execution.observability_alert_count = daemon_observability.alert_count;
    execution.observability_reason_code = daemon_observability.reason_code;
    execution.observability_transport_checkpoint_failures =
        daemon_observability.transport_checkpoint_failures;
    execution.observability_signer_checkpoint_failures =
        daemon_observability.signer_checkpoint_failures;
    execution.observability_commit_checkpoint_failures =
        daemon_observability.commit_checkpoint_failures;
}

fn populate_projection_fields(execution: &mut DaemonExecution, report: &DaemonReportSnapshot) {
    populate_phase6_fields(execution, report);
    populate_convergence_fields(execution, report);
    populate_live_postgres_fields(execution, report);
}

fn populate_phase6_fields(execution: &mut DaemonExecution, report: &DaemonReportSnapshot) {
    execution.phase6_runtime_reason_taxonomy_version =
        super::super::projections::phase6_runtime_reason_taxonomy_version().to_owned();
    execution.phase6_runtime_reason_codes_csv =
        super::super::projections::phase6_runtime_reason_codes_csv().to_owned();
    execution.phase6_runtime_reason_code = report.phase6.reason_code.to_owned();
    execution.phase6_runtime_total_cycles = report.phase6.total_cycles;
    execution.phase6_runtime_executed_cycles = report.phase6.executed_cycles;
    execution.phase6_runtime_deferred_cycles = report.phase6.deferred_cycles;
    execution.phase6_runtime_fail_closed_cycles = report.phase6.fail_closed_cycles;
}

fn populate_convergence_fields(execution: &mut DaemonExecution, report: &DaemonReportSnapshot) {
    execution.convergence_reason_taxonomy_version =
        super::super::projections::convergence_reason_taxonomy_version().to_owned();
    execution.convergence_reason_codes_csv =
        super::super::projections::convergence_reason_codes_csv().to_owned();
    execution.convergence_decision = report.convergence.decision.to_owned();
    execution.convergence_reason_code = report.convergence.reason_code.to_owned();
    execution.convergence_schema_gate_passed = report.convergence.schema_gate_passed;
    execution.convergence_error_path_gate_passed = report.convergence.error_path_gate_passed;
    execution.convergence_concurrency_gate_passed = report.convergence.concurrency_gate_passed;
    execution.convergence_performance_budget_gate_passed =
        report.convergence.performance_budget_gate_passed;
    execution.convergence_cost_budget_gate_passed = report.convergence.cost_budget_gate_passed;
}

fn populate_live_postgres_fields(execution: &mut DaemonExecution, report: &DaemonReportSnapshot) {
    execution.live_postgres_multi_host_execution_bundle_schema_version =
        super::super::live_postgres_bundle::live_postgres_schema_version().to_owned();
    execution.live_postgres_multi_host_execution_bundle_selector_prefix =
        super::super::live_postgres_bundle::live_postgres_selector_prefix().to_owned();
    execution.live_postgres_multi_host_execution_bundle_row_count = report.row_count;
    execution.live_postgres_multi_host_execution_bundle_selector_rows_fingerprint =
        report.selector_rows_fingerprint.clone();
}
