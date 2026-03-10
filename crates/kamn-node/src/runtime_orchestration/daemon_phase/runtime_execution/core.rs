use super::super::super::*;
use super::super::service_api_relay_tick_loop::execute_daemon_service_api_relay_tick_loop;
use super::completion_logging::log_daemon_execution_complete;
use super::execution_builder::{build_daemon_execution, DaemonPeerLifecycle, ExecutedDaemonRun};
use super::lifecycle::build_peer_lifecycle_summary;
use super::options::{parse_daemon_runtime_options, ParsedDaemonRuntimeOptions};
use super::reporting::{build_daemon_report_snapshot, DaemonReportSnapshot};
use super::shutdown_fields::daemon_shutdown_drain_status;
use crate::daemon_shutdown::DaemonCompletion;

pub(super) fn execute_daemon_runtime(
    runtime_mode: RuntimeMode,
    execution_id: &str,
    options: DaemonRuntimeOptions,
) -> Result<DaemonExecution, ConfigError> {
    let options = parse_daemon_runtime_options(options)?;
    log_daemon_execution_start(
        runtime_mode,
        execution_id,
        options.max_ticks,
        options.tick_interval_ms,
    )?;
    let peer_lifecycle = build_daemon_peer_lifecycle(&options)?;
    let daemon_run = execute_daemon_run(runtime_mode, &options)?;
    log_daemon_execution_complete(
        runtime_mode,
        execution_id,
        &daemon_run.daemon_completion,
        &daemon_run.runtime_processing,
        &daemon_run.report,
    )?;
    Ok(build_daemon_execution(
        options.max_ticks,
        options.tick_interval_ms,
        peer_lifecycle,
        daemon_run,
    ))
}

fn build_daemon_peer_lifecycle(
    options: &ParsedDaemonRuntimeOptions,
) -> Result<DaemonPeerLifecycle, ConfigError> {
    let (peer_id, final_state, applied_events) = build_peer_lifecycle_summary(
        options.daemon_peer_id.clone(),
        options.daemon_lifecycle_events.clone(),
    )?;
    Ok(DaemonPeerLifecycle {
        peer_id,
        final_state,
        applied_events,
    })
}

fn execute_daemon_run(
    runtime_mode: RuntimeMode,
    options: &ParsedDaemonRuntimeOptions,
) -> Result<ExecutedDaemonRun, ConfigError> {
    let daemon_completion = evaluate_daemon_completion_for_options(runtime_mode, options)?;
    let runtime_processing = execute_relay_tick_loop(&daemon_completion, options)?;
    let daemon_observability = build_daemon_observability(
        &daemon_completion,
        options.tick_interval_ms,
        &runtime_processing,
    )?;
    validate_shutdown_observability(&daemon_completion, &daemon_observability)?;
    let report = build_daemon_report_snapshot(
        options.max_ticks,
        options.tick_interval_ms,
        options.daemon_shutdown_signal_ticks.as_slice(),
        &daemon_completion,
        daemon_observability.reason_code.as_str(),
    )?;
    Ok(ExecutedDaemonRun {
        daemon_completion,
        runtime_processing,
        daemon_observability,
        report,
    })
}

fn execute_relay_tick_loop(
    daemon_completion: &DaemonCompletion,
    options: &ParsedDaemonRuntimeOptions,
) -> Result<crate::daemon_observability::DaemonRuntimeProcessingTelemetry, ConfigError> {
    execute_daemon_service_api_relay_tick_loop(
        daemon_completion.executed_ticks,
        options.tick_interval_ms,
        options.service_api_state_file.as_deref(),
        options.service_api_relay_spool_file.as_deref(),
        options.service_api_signature_state_hash.as_str(),
    )
}

fn validate_shutdown_observability(
    daemon_completion: &DaemonCompletion,
    daemon_observability: &crate::daemon_observability::DaemonObservabilityTelemetry,
) -> Result<(), ConfigError> {
    validate_shutdown_checkpoint_reconciliation(
        daemon_completion.completion_reason.as_str(),
        daemon_observability.reason_code.as_str(),
        daemon_observability.transport_checkpoint_failures,
        daemon_observability.signer_checkpoint_failures,
        daemon_observability.commit_checkpoint_failures,
    )
}

fn log_daemon_execution_start(
    runtime_mode: RuntimeMode,
    execution_id: &str,
    max_ticks: u64,
    tick_interval_ms: u64,
) -> Result<(), ConfigError> {
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
    )
}

fn evaluate_daemon_completion_for_options(
    runtime_mode: RuntimeMode,
    options: &ParsedDaemonRuntimeOptions,
) -> Result<DaemonCompletion, ConfigError> {
    if should_use_os_signal_shutdown(
        runtime_mode,
        options.daemon_shutdown_os_signals,
        options.daemon_shutdown_signal_ticks.as_slice(),
    ) {
        return evaluate_daemon_completion_with_os_signals(
            options.max_ticks,
            options.tick_interval_ms,
            options.daemon_shutdown_drain_ticks,
            options.daemon_shutdown_timeout_ticks,
        )
        .map_err(|error| ConfigError::RuntimeDaemonLifecycle(error.to_string()));
    }
    Ok(evaluate_daemon_completion(
        options.max_ticks,
        options.daemon_shutdown_signal_ticks.as_slice(),
        options.daemon_shutdown_drain_ticks,
        options.daemon_shutdown_timeout_ticks,
    ))
}

fn build_daemon_observability(
    daemon_completion: &DaemonCompletion,
    tick_interval_ms: u64,
    runtime_processing: &crate::daemon_observability::DaemonRuntimeProcessingTelemetry,
) -> Result<crate::daemon_observability::DaemonObservabilityTelemetry, ConfigError> {
    build_daemon_observability_telemetry(
        tick_interval_ms,
        daemon_completion.completion_reason.as_str(),
        runtime_processing,
    )
    .map_err(|error| ConfigError::RuntimeDaemonLifecycle(error.to_string()))
}
