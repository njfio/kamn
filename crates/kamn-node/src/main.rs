use kamn_core::{
    bootstrap, bootstrap_with_transport_profile, ConfigError, DeterministicProposalPlanner,
    NodeConfig, NodeRole, PeerLifecycle, PeerLifecycleEvent, PeerLifecycleState, ProposalCandidate,
    RecoveryRejoinGuard, RecoveryStatus, RejoinAttempt, RuntimeTransportProfile, SyncMode,
};
use std::env;
use std::process::ExitCode;

mod cli;
mod daemon_observability;
mod daemon_shutdown;
mod kolme_live_observability;
mod logging;
mod observability_endpoint;
mod output_io;
mod report_builder;
mod report_render;
mod runtime_constants;
mod runtime_entrypoint;
mod runtime_kolme_live;
mod runtime_models;
mod runtime_modes;
mod runtime_orchestration;
mod service_api_endpoint;
mod signer;
mod wire_payload;

use cli::parse_args;
use daemon_observability::build_daemon_observability_telemetry;
#[cfg(test)]
pub(crate) use daemon_shutdown::{
    configure_os_signal_test_triggers, OsSignalTestKind, OsSignalTestTrigger,
};
use daemon_shutdown::{evaluate_daemon_completion, evaluate_daemon_completion_with_os_signals};
#[cfg(test)]
use logging::{
    capture_test_logs, render_log_event_line, reset_cached_log_config_for_tests,
    resolve_log_config_from_inputs, NodeLogConfig, NodeLogFormat, NodeLogLevel,
    KAMN_NODE_LOG_FORMAT_ENV, KAMN_NODE_LOG_LEVEL_ENV,
};
use logging::{initialize_log_config_from_env, log_error, log_info};
#[cfg(test)]
pub(crate) use observability_endpoint::render_observability_endpoint_response;
pub(crate) use observability_endpoint::{
    build_runtime_observability_snapshot, serve_observability_endpoint,
    ObservabilityEndpointConfig, DEFAULT_OBSERVABILITY_ENDPOINT_HEALTH_PATH,
    DEFAULT_OBSERVABILITY_ENDPOINT_IDLE_TIMEOUT_MS, DEFAULT_OBSERVABILITY_ENDPOINT_MAX_REQUESTS,
    DEFAULT_OBSERVABILITY_ENDPOINT_METRICS_PATH,
};
use output_io::{emit_bootstrap_report_output, write_stderr_line};
use report_builder::build_bootstrap_report;
#[cfg(test)]
use report_render::render_bootstrap_report;
pub(crate) use runtime_constants::*;
use runtime_entrypoint::serve_runtime_endpoints;
#[cfg(test)]
pub(crate) use runtime_entrypoint::{
    classify_service_api_endpoint_runtime_path,
    should_skip_observability_endpoint_for_full_supervisor, ServiceApiEndpointRuntimePath,
};
#[cfg(test)]
use runtime_kolme_live::build_kolme_live_request;
use runtime_kolme_live::{
    execute_kolme_live_runtime, execute_kolme_live_runtime_continuous, KolmeLiveContinuousMode,
};
pub(crate) use runtime_models::{
    DaemonExecution, DaemonRuntimeOptions, KolmeLiveExecution, NodeBootstrapReport, NodeCli,
    PlanningExecution, RecoveryExecution, RuntimeExecutionBundle,
};
pub(crate) use runtime_modes::{
    DiagnosticsMode, LocalProfile, OutputMode, OutputModeKind, RuntimeMode, RuntimeModeKind,
};
use runtime_orchestration::{build_runtime_execution_id, execute};
#[cfg(test)]
pub(crate) use runtime_orchestration::{
    classify_full_bootstrap_component_contract_violation,
    classify_full_supervisor_stop_contract_violation,
    classify_kolme_live_signer_key_source_policy_violation,
    classify_production_transport_profile_violation,
    classify_shutdown_checkpoint_reconciliation_violation,
    enforce_kolme_live_signer_contract_policy, enforce_kolme_live_signer_key_source_policy,
    execute_daemon_convergence_projection_for_test,
    execute_daemon_phase6_runtime_projection_for_test,
    live_postgres_multi_host_execution_bundle_row_count_for_test,
    live_postgres_multi_host_execution_bundle_selector_rows_fingerprint_for_test,
    live_postgres_multi_host_execution_bundle_selector_rows_for_test,
    resolve_kolme_live_allow_local_signer_testing_override,
    select_runtime_transport_profile_for_runtime_mode, should_use_os_signal_shutdown,
    validate_full_supervisor_stop_contract, validate_live_postgres_selector_bundle_for_test,
    validate_shutdown_checkpoint_reconciliation,
};
#[cfg(test)]
pub(crate) use service_api_endpoint::render_service_api_endpoint_response;
pub(crate) use service_api_endpoint::{
    build_service_api_snapshot, serve_service_api_endpoint, ServiceApiEndpointConfig,
    DEFAULT_SERVICE_API_BODY_LIMIT_BYTES, DEFAULT_SERVICE_API_CONCURRENCY_LIMIT,
    DEFAULT_SERVICE_API_IDLE_TIMEOUT_MS, DEFAULT_SERVICE_API_MAX_REQUESTS,
    DEFAULT_SERVICE_API_RATE_LIMIT_PER_SECOND,
};
#[cfg(test)]
use signer::{
    build_kolme_live_direct_signed_wire_payload, build_kolme_live_managed_signing_key,
    build_kolme_live_signer_adapter, encode_kolme_hex_lower,
    resolve_kolme_live_managed_signer_required_marker, resolve_kolme_live_nonce,
    resolve_kolme_live_signer_private_key_env_name, sign_kolme_live_managed_external_message,
    KolmeForkSecp256k1SignerAdapter,
};
use signer::{
    enforce_kolme_live_signer_preflight, normalize_kolme_live_signer_key_source,
    normalize_kolme_live_signer_profile_selector,
};
#[cfg(test)]
use wire_payload::render_kolme_live_native_direct_message;

fn run() -> Result<(), ConfigError> {
    let cli = parse_args(env::args())?;
    initialize_log_config_from_env()?;
    let runtime_mode = cli.runtime_mode.as_str();
    let execution_id =
        build_runtime_execution_id(cli.runtime_mode, cli.chain_id.as_str(), cli.role.as_str());
    log_info(
        "node.runtime.execute.start",
        &[
            ("runtime_mode", runtime_mode),
            ("execution_id", execution_id.as_str()),
        ],
    )?;
    let output_mode = cli.output_mode;
    let service_api_endpoint_config =
        cli.api_bind_addr
            .as_ref()
            .map(|bind_addr| ServiceApiEndpointConfig {
                bind_addr: bind_addr.clone(),
                max_requests: cli.api_max_requests,
                idle_timeout_ms: cli.api_idle_timeout_ms,
                body_limit_bytes: cli.api_body_limit_bytes,
                concurrency_limit: cli.api_concurrency_limit,
                rate_limit_per_second: cli.api_rate_limit_per_second,
            });
    let observability_endpoint_config =
        cli.observability_endpoint_bind_addr
            .as_ref()
            .map(|bind_addr| ObservabilityEndpointConfig {
                bind_addr: bind_addr.clone(),
                metrics_path: cli.observability_endpoint_metrics_path.clone(),
                health_path: cli.observability_endpoint_health_path.clone(),
                max_requests: cli.observability_endpoint_max_requests,
                idle_timeout_ms: cli.observability_endpoint_idle_timeout_ms,
            });
    let report = execute(cli)?;
    log_info(
        "node.runtime.execute.complete",
        &[
            ("runtime_mode", report.runtime_mode.as_str()),
            ("role", report.role.as_str()),
            ("execution_id", execution_id.as_str()),
        ],
    )?;
    emit_bootstrap_report_output(&report, output_mode)?;
    serve_runtime_endpoints(
        &report,
        service_api_endpoint_config.as_ref(),
        observability_endpoint_config.as_ref(),
        execution_id.as_str(),
    )?;

    Ok(())
}

async fn run_async() -> Result<(), ConfigError> {
    tokio::task::spawn_blocking(run)
        .await
        .map_err(|error| ConfigError::RuntimeDaemonLifecycle(error.to_string()))?
}

#[tokio::main(flavor = "multi_thread")]
async fn main() -> ExitCode {
    match run_async().await {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            let error_message = error.to_string();
            let _ = log_error(
                "node.runtime.execute.failed",
                &[("error", error_message.as_str())],
            );
            let _ = write_stderr_line(error_message.as_str());
            ExitCode::FAILURE
        }
    }
}

#[cfg(test)]
mod cli_tests;
#[cfg(test)]
mod main_tests;
