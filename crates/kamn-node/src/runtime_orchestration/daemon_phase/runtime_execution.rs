mod completion_logging;
mod core;
mod execution_builder;
mod lifecycle;
mod options;
mod reporting;
mod shutdown_fields;

use super::super::*;

pub(super) fn execute_daemon_runtime(
    runtime_mode: RuntimeMode,
    execution_id: &str,
    options: DaemonRuntimeOptions,
) -> Result<DaemonExecution, ConfigError> {
    core::execute_daemon_runtime(runtime_mode, execution_id, options)
}

pub(super) fn daemon_shutdown_drain_status(completion_reason: &str) -> &'static str {
    shutdown_fields::daemon_shutdown_drain_status(completion_reason)
}

pub(super) fn daemon_shutdown_snapshot_flush_status(completion_reason: &str) -> &'static str {
    shutdown_fields::daemon_shutdown_snapshot_flush_status(completion_reason)
}

pub(super) fn daemon_shutdown_signal_tick(completion_reason: &str) -> Option<&str> {
    shutdown_fields::daemon_shutdown_signal_tick(completion_reason)
}

pub(super) fn daemon_shutdown_reason_field<'a>(
    completion_reason: &'a str,
    key: &str,
) -> Option<&'a str> {
    shutdown_fields::daemon_shutdown_reason_field(completion_reason, key)
}
