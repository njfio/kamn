mod live_postgres_bundle;
mod projections;
mod runtime_execution;
mod service_api_relay_p2p;
mod service_api_relay_tick_loop;
#[cfg(test)]
mod tests;

use super::*;

#[cfg(test)]
pub(crate) use live_postgres_bundle::{
    live_postgres_multi_host_execution_bundle_row_count_for_test,
    live_postgres_multi_host_execution_bundle_selector_rows_fingerprint_for_test,
    live_postgres_multi_host_execution_bundle_selector_rows_for_test,
    validate_live_postgres_selector_bundle_for_test,
};
#[cfg(test)]
pub(crate) use projections::{
    execute_daemon_convergence_projection_for_test,
    execute_daemon_phase6_runtime_projection_for_test,
};

pub(super) fn execute_daemon_runtime(
    runtime_mode: RuntimeMode,
    execution_id: &str,
    options: DaemonRuntimeOptions,
) -> Result<DaemonExecution, ConfigError> {
    runtime_execution::execute_daemon_runtime(runtime_mode, execution_id, options)
}

pub(super) fn daemon_shutdown_drain_status(completion_reason: &str) -> &'static str {
    runtime_execution::daemon_shutdown_drain_status(completion_reason)
}

pub(super) fn daemon_shutdown_snapshot_flush_status(completion_reason: &str) -> &'static str {
    runtime_execution::daemon_shutdown_snapshot_flush_status(completion_reason)
}

pub(super) fn daemon_shutdown_signal_tick(completion_reason: &str) -> Option<&str> {
    runtime_execution::daemon_shutdown_signal_tick(completion_reason)
}

pub(super) fn daemon_shutdown_reason_field<'a>(
    completion_reason: &'a str,
    key: &str,
) -> Option<&'a str> {
    runtime_execution::daemon_shutdown_reason_field(completion_reason, key)
}
