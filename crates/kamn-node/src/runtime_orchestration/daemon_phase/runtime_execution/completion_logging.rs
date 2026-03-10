use super::super::super::*;
use super::super::live_postgres_bundle::{
    live_postgres_schema_version, live_postgres_selector_prefix,
};
use super::super::projections::{
    convergence_reason_codes_csv, convergence_reason_taxonomy_version,
    phase6_runtime_reason_codes_csv, phase6_runtime_reason_taxonomy_version,
};
use super::reporting::DaemonReportSnapshot;
use crate::daemon_shutdown::DaemonCompletion;

pub(super) fn log_daemon_execution_complete(
    runtime_mode: RuntimeMode,
    execution_id: &str,
    daemon_completion: &DaemonCompletion,
    runtime_processing: &crate::daemon_observability::DaemonRuntimeProcessingTelemetry,
    report: &DaemonReportSnapshot,
) -> Result<(), ConfigError> {
    let metric_labels = completion_metric_labels(daemon_completion, runtime_processing, report);
    log_info(
        "node.runtime.daemon.execute.complete",
        &completion_log_fields(
            runtime_mode,
            execution_id,
            daemon_completion,
            report,
            &metric_labels,
        ),
    )
}

struct CompletionMetricLabels {
    executed_ticks: String,
    relay_drained_count: String,
    relay_projected_state_count: String,
    phase6: Phase6MetricLabels,
}

struct Phase6MetricLabels {
    total_cycles: String,
    executed_cycles: String,
    deferred_cycles: String,
    fail_closed_cycles: String,
}

fn completion_metric_labels(
    daemon_completion: &DaemonCompletion,
    runtime_processing: &crate::daemon_observability::DaemonRuntimeProcessingTelemetry,
    report: &DaemonReportSnapshot,
) -> CompletionMetricLabels {
    CompletionMetricLabels {
        executed_ticks: daemon_completion.executed_ticks.to_string(),
        relay_drained_count: runtime_processing.relay_drained_count.to_string(),
        relay_projected_state_count: runtime_processing.relay_projected_state_count.to_string(),
        phase6: phase6_metric_labels(report),
    }
}

fn phase6_metric_labels(report: &DaemonReportSnapshot) -> Phase6MetricLabels {
    Phase6MetricLabels {
        total_cycles: report.phase6.total_cycles.to_string(),
        executed_cycles: report.phase6.executed_cycles.to_string(),
        deferred_cycles: report.phase6.deferred_cycles.to_string(),
        fail_closed_cycles: report.phase6.fail_closed_cycles.to_string(),
    }
}

fn completion_log_fields<'a>(
    runtime_mode: RuntimeMode,
    execution_id: &'a str,
    daemon_completion: &'a DaemonCompletion,
    report: &'a DaemonReportSnapshot,
    metric_labels: &'a CompletionMetricLabels,
) -> [(&'a str, &'a str); 33] {
    [
        ("runtime_mode", runtime_mode.as_str()),
        ("executed_ticks", metric_labels.executed_ticks.as_str()),
        (
            "completion_reason",
            daemon_completion.completion_reason.as_str(),
        ),
        ("shutdown_drain_status", report.shutdown_drain_status),
        (
            "shutdown_snapshot_flush_status",
            report.shutdown_snapshot_flush_status,
        ),
        ("shutdown_signal_tick", report.shutdown_signal_tick.as_str()),
        ("shutdown_drain_ticks", report.shutdown_drain_ticks.as_str()),
        (
            "shutdown_timeout_ticks",
            report.shutdown_timeout_ticks.as_str(),
        ),
        (
            "shutdown_ignored_signals",
            report.shutdown_ignored_signals.as_str(),
        ),
        (
            "phase6_reason_taxonomy_version",
            phase6_runtime_reason_taxonomy_version(),
        ),
        ("phase6_reason_codes_csv", phase6_runtime_reason_codes_csv()),
        ("phase6_reason_code", report.phase6.reason_code),
        (
            "phase6_total_cycles",
            metric_labels.phase6.total_cycles.as_str(),
        ),
        (
            "phase6_executed_cycles",
            metric_labels.phase6.executed_cycles.as_str(),
        ),
        (
            "phase6_deferred_cycles",
            metric_labels.phase6.deferred_cycles.as_str(),
        ),
        (
            "phase6_fail_closed_cycles",
            metric_labels.phase6.fail_closed_cycles.as_str(),
        ),
        (
            "convergence_reason_taxonomy_version",
            convergence_reason_taxonomy_version(),
        ),
        (
            "convergence_reason_codes_csv",
            convergence_reason_codes_csv(),
        ),
        ("convergence_decision", report.convergence.decision),
        ("convergence_reason_code", report.convergence.reason_code),
        (
            "convergence_schema_gate_passed",
            bool_label(report.convergence.schema_gate_passed),
        ),
        (
            "convergence_error_path_gate_passed",
            bool_label(report.convergence.error_path_gate_passed),
        ),
        (
            "convergence_concurrency_gate_passed",
            bool_label(report.convergence.concurrency_gate_passed),
        ),
        (
            "convergence_performance_budget_gate_passed",
            bool_label(report.convergence.performance_budget_gate_passed),
        ),
        (
            "convergence_cost_budget_gate_passed",
            bool_label(report.convergence.cost_budget_gate_passed),
        ),
        (
            "multi_host_execution_bundle_schema_version",
            live_postgres_schema_version(),
        ),
        (
            "multi_host_execution_bundle_selector_prefix",
            live_postgres_selector_prefix(),
        ),
        (
            "multi_host_execution_bundle_row_count",
            report.row_count_label.as_str(),
        ),
        (
            "multi_host_execution_bundle_selector_rows_csv",
            report.selector_rows_csv.as_str(),
        ),
        (
            "multi_host_execution_bundle_selector_rows_fingerprint",
            report.selector_rows_fingerprint.as_str(),
        ),
        (
            "service_api_relay_drained_count",
            metric_labels.relay_drained_count.as_str(),
        ),
        (
            "service_api_relay_projected_state_count",
            metric_labels.relay_projected_state_count.as_str(),
        ),
        ("execution_id", execution_id),
    ]
}

fn bool_label(value: bool) -> &'static str {
    if value {
        "true"
    } else {
        "false"
    }
}
