mod convergence;
mod phase6_runtime;

pub(super) use convergence::{
    convergence_reason_codes_csv, convergence_reason_taxonomy_version,
    execute_daemon_convergence_projection, DaemonConvergenceInput, DaemonConvergenceProjection,
};
pub(super) use phase6_runtime::{
    execute_daemon_phase6_runtime_projection, phase6_runtime_reason_codes_csv,
    phase6_runtime_reason_taxonomy_version, DaemonPhase6RuntimeProjection,
};

#[cfg(test)]
pub(crate) fn execute_daemon_phase6_runtime_projection_for_test(
    max_ticks: u64,
    tick_interval_ms: u64,
    has_shutdown_signal: bool,
    regressed_now_epoch_seconds: Option<u64>,
) -> Result<(&'static str, u64), super::super::super::ConfigError> {
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
