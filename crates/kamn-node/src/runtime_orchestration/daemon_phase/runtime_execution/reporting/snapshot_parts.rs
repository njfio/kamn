use super::{DaemonConvergenceProjection, DaemonPhase6RuntimeProjection, DaemonReportSnapshot};
use crate::daemon_shutdown::DaemonCompletion;

pub(super) struct SelectorSnapshot {
    pub(super) fingerprint: String,
    pub(super) row_count: usize,
    pub(super) row_count_label: String,
    pub(super) rows_csv: String,
}

pub(super) struct ShutdownSnapshot {
    pub(super) signal_tick: String,
    pub(super) drain_ticks: String,
    pub(super) timeout_ticks: String,
    pub(super) ignored_signals: String,
    pub(super) drain_status: &'static str,
    pub(super) snapshot_flush_status: &'static str,
}

pub(super) fn build_selector_snapshot() -> Result<SelectorSnapshot, super::ConfigError> {
    let (selector_rows, row_count) = super::validated_selector_rows()?;
    Ok(SelectorSnapshot {
        fingerprint:
            super::project_live_postgres_multi_host_execution_bundle_selector_rows_fingerprint(
                selector_rows.as_slice(),
            ),
        row_count,
        row_count_label: row_count.to_string(),
        rows_csv: selector_rows.join(","),
    })
}

pub(super) fn build_shutdown_snapshot(daemon_completion: &DaemonCompletion) -> ShutdownSnapshot {
    ShutdownSnapshot {
        signal_tick: super::shutdown_signal_tick_value(daemon_completion),
        drain_ticks: super::shutdown_reason_value(daemon_completion, "drain_ticks", "0"),
        timeout_ticks: super::shutdown_reason_value(daemon_completion, "timeout_ticks", "0"),
        ignored_signals: super::shutdown_reason_value(daemon_completion, "ignored_signals", "0"),
        drain_status: super::daemon_shutdown_drain_status(
            daemon_completion.completion_reason.as_str(),
        ),
        snapshot_flush_status: super::daemon_shutdown_snapshot_flush_status(
            daemon_completion.completion_reason.as_str(),
        ),
    }
}

pub(super) fn report_snapshot(
    phase6: DaemonPhase6RuntimeProjection,
    selector_snapshot: SelectorSnapshot,
    shutdown_snapshot: ShutdownSnapshot,
    daemon_reason_code: &str,
    max_ticks: u64,
    tick_interval_ms: u64,
) -> DaemonReportSnapshot {
    let convergence =
        convergence_projection(&phase6, daemon_reason_code, max_ticks, tick_interval_ms);
    DaemonReportSnapshot {
        phase6,
        convergence,
        shutdown_signal_tick: shutdown_snapshot.signal_tick,
        shutdown_drain_ticks: shutdown_snapshot.drain_ticks,
        shutdown_timeout_ticks: shutdown_snapshot.timeout_ticks,
        shutdown_ignored_signals: shutdown_snapshot.ignored_signals,
        shutdown_drain_status: shutdown_snapshot.drain_status,
        shutdown_snapshot_flush_status: shutdown_snapshot.snapshot_flush_status,
        selector_rows_fingerprint: selector_snapshot.fingerprint,
        row_count: selector_snapshot.row_count,
        row_count_label: selector_snapshot.row_count_label,
        selector_rows_csv: selector_snapshot.rows_csv,
    }
}

fn convergence_projection(
    phase6: &DaemonPhase6RuntimeProjection,
    daemon_reason_code: &str,
    max_ticks: u64,
    tick_interval_ms: u64,
) -> DaemonConvergenceProjection {
    super::execute_daemon_convergence_projection(super::build_convergence_input(
        phase6,
        daemon_reason_code,
        max_ticks,
        tick_interval_ms,
    ))
}
