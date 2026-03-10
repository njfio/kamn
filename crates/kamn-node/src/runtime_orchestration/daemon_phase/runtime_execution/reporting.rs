use super::super::super::*;
use super::super::live_postgres_bundle::{
    daemon_live_postgres_multi_host_execution_bundle_row_count,
    project_live_postgres_multi_host_execution_bundle_selector_rows,
    project_live_postgres_multi_host_execution_bundle_selector_rows_fingerprint,
    validate_live_postgres_selector_bundle,
};
use super::super::projections::{
    execute_daemon_convergence_projection, execute_daemon_phase6_runtime_projection,
    DaemonConvergenceInput, DaemonConvergenceProjection, DaemonPhase6RuntimeProjection,
};
use super::shutdown_fields::{
    daemon_shutdown_drain_status, daemon_shutdown_reason_field, daemon_shutdown_signal_tick,
    daemon_shutdown_snapshot_flush_status,
};
use crate::daemon_shutdown::DaemonCompletion;

pub(super) struct DaemonReportSnapshot {
    pub(super) phase6: DaemonPhase6RuntimeProjection,
    pub(super) convergence: DaemonConvergenceProjection,
    pub(super) shutdown_signal_tick: String,
    pub(super) shutdown_drain_ticks: String,
    pub(super) shutdown_timeout_ticks: String,
    pub(super) shutdown_ignored_signals: String,
    pub(super) shutdown_drain_status: &'static str,
    pub(super) shutdown_snapshot_flush_status: &'static str,
    pub(super) selector_rows_fingerprint: String,
    pub(super) row_count: usize,
    pub(super) row_count_label: String,
    pub(super) selector_rows_csv: String,
}

pub(super) fn build_daemon_report_snapshot(
    max_ticks: u64,
    tick_interval_ms: u64,
    daemon_shutdown_signal_ticks: &[u64],
    daemon_completion: &DaemonCompletion,
    daemon_reason_code: &str,
) -> Result<DaemonReportSnapshot, ConfigError> {
    let phase6 =
        build_phase6_runtime_snapshot(max_ticks, tick_interval_ms, daemon_shutdown_signal_ticks)?;
    let (selector_rows, row_count) = validated_selector_rows()?;
    let convergence = execute_daemon_convergence_projection(build_convergence_input(
        &phase6,
        daemon_reason_code,
        max_ticks,
        tick_interval_ms,
    ));
    Ok(DaemonReportSnapshot {
        phase6,
        convergence,
        shutdown_signal_tick: shutdown_signal_tick_value(daemon_completion),
        shutdown_drain_ticks: shutdown_reason_value(daemon_completion, "drain_ticks", "0"),
        shutdown_timeout_ticks: shutdown_reason_value(daemon_completion, "timeout_ticks", "0"),
        shutdown_ignored_signals: shutdown_reason_value(daemon_completion, "ignored_signals", "0"),
        shutdown_drain_status: daemon_shutdown_drain_status(
            daemon_completion.completion_reason.as_str(),
        ),
        shutdown_snapshot_flush_status: daemon_shutdown_snapshot_flush_status(
            daemon_completion.completion_reason.as_str(),
        ),
        selector_rows_fingerprint:
            project_live_postgres_multi_host_execution_bundle_selector_rows_fingerprint(
                selector_rows.as_slice(),
            ),
        row_count,
        row_count_label: row_count.to_string(),
        selector_rows_csv: selector_rows.join(","),
    })
}

fn build_phase6_runtime_snapshot(
    max_ticks: u64,
    tick_interval_ms: u64,
    daemon_shutdown_signal_ticks: &[u64],
) -> Result<DaemonPhase6RuntimeProjection, ConfigError> {
    execute_daemon_phase6_runtime_projection(
        max_ticks,
        tick_interval_ms,
        !daemon_shutdown_signal_ticks.is_empty(),
        None,
    )
}

fn validated_selector_rows() -> Result<(Vec<String>, usize), ConfigError> {
    let selector_rows = project_live_postgres_multi_host_execution_bundle_selector_rows();
    let row_count = daemon_live_postgres_multi_host_execution_bundle_row_count();
    validate_live_postgres_selector_bundle(selector_rows.as_slice(), row_count)
        .map_err(selector_bundle_validation_error)?;
    Ok((selector_rows, row_count))
}

fn selector_bundle_validation_error(reason_code: &str) -> ConfigError {
    ConfigError::RuntimeDaemonLifecycle(format!(
        "live_postgres_selector_bundle_validation_failed:{reason_code}"
    ))
}

fn build_convergence_input(
    phase6: &DaemonPhase6RuntimeProjection,
    daemon_reason_code: &str,
    max_ticks: u64,
    tick_interval_ms: u64,
) -> DaemonConvergenceInput {
    DaemonConvergenceInput {
        schema_gate_passed: phase6_schema_gate_passed(phase6),
        error_path_gate_passed: phase6.fail_closed_cycles == 0,
        concurrency_gate_passed: phase6_counts_reconcile(phase6),
        performance_budget_gate_passed: daemon_reason_code != "daemon_shutdown_timeout",
        cost_budget_gate_passed: max_ticks <= 10_000 && tick_interval_ms <= 5_000,
    }
}

fn phase6_schema_gate_passed(phase6: &DaemonPhase6RuntimeProjection) -> bool {
    phase6.total_cycles > 0 && phase6.reason_code != "m10_phase6_scheduler_signal_invalid"
}

fn phase6_counts_reconcile(phase6: &DaemonPhase6RuntimeProjection) -> bool {
    phase6.total_cycles
        == phase6
            .executed_cycles
            .saturating_add(phase6.deferred_cycles)
            .saturating_add(phase6.fail_closed_cycles)
}

fn shutdown_signal_tick_value(daemon_completion: &DaemonCompletion) -> String {
    daemon_shutdown_signal_tick(daemon_completion.completion_reason.as_str())
        .unwrap_or("none")
        .to_owned()
}

fn shutdown_reason_value(
    daemon_completion: &DaemonCompletion,
    key: &str,
    default_value: &str,
) -> String {
    daemon_shutdown_reason_field(daemon_completion.completion_reason.as_str(), key)
        .unwrap_or(default_value)
        .to_owned()
}
