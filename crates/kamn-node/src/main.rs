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
mod report_builder;
mod report_render;
mod runtime_kolme_live;
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
    capture_test_logs, render_log_event_line, resolve_log_config_from_inputs, NodeLogConfig,
    NodeLogFormat, NodeLogLevel,
};
use logging::{log_error, log_info};
#[cfg(test)]
pub(crate) use observability_endpoint::render_observability_endpoint_response;
pub(crate) use observability_endpoint::{
    build_runtime_observability_snapshot, serve_observability_endpoint,
    ObservabilityEndpointConfig, DEFAULT_OBSERVABILITY_ENDPOINT_HEALTH_PATH,
    DEFAULT_OBSERVABILITY_ENDPOINT_IDLE_TIMEOUT_MS, DEFAULT_OBSERVABILITY_ENDPOINT_MAX_REQUESTS,
    DEFAULT_OBSERVABILITY_ENDPOINT_METRICS_PATH,
};
use report_builder::build_bootstrap_report;
use report_render::render_bootstrap_report;
#[cfg(test)]
use runtime_kolme_live::build_kolme_live_request;
use runtime_kolme_live::{
    execute_kolme_live_runtime, execute_kolme_live_runtime_continuous, KolmeLiveContinuousMode,
};
use runtime_orchestration::{build_runtime_execution_id, execute};
#[cfg(test)]
pub(crate) use runtime_orchestration::{
    classify_full_bootstrap_component_contract_violation,
    classify_full_supervisor_stop_contract_violation,
    classify_kolme_live_signer_key_source_policy_violation,
    classify_production_transport_profile_violation, enforce_kolme_live_signer_contract_policy,
    enforce_kolme_live_signer_key_source_policy,
    resolve_kolme_live_allow_local_signer_testing_override,
    select_runtime_transport_profile_for_runtime_mode, should_use_os_signal_shutdown,
    validate_full_supervisor_stop_contract,
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

const KOLME_LIVE_PROVIDER_CONTRACT: &str = "KolmeRuntimeCommitLiveProvider";
const KOLME_LIVE_SIGNING_PROFILE: &str = "kolme-fork-secp256k1-v1";
const KOLME_IN_MEMORY_PROVIDER_MARKER: &str = "InMemoryKolmeRuntimeCommitClient";
const KOLME_LIVE_TRANSPORT_TIMEOUT_SECONDS: u64 = 30;
const KOLME_LIVE_FINALITY_STATUS_PATH: &str = "/runtime-commit/status";
const KOLME_LIVE_FINALITY_MAX_ATTEMPTS: u32 = 2;
const KOLME_LIVE_NONCE_PATH: &str = "/get-next-nonce";
const KOLME_LIVE_SIGNER_PROFILE_ENV: &str = "KAMN_KOLME_LIVE_SIGNER_PROFILE";
const KOLME_LIVE_SIGNER_PROFILE_PRIMARY: &str = "ops-primary";
const KOLME_LIVE_SIGNER_PROFILE_SECONDARY: &str = "ops-secondary";
const KOLME_LIVE_SIGNER_KEY_SOURCE_ENV_LOCAL: &str = "env-local";
const KOLME_LIVE_SIGNER_KEY_SOURCE_MANAGED_EXTERNAL: &str = "managed-external";
const KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX_ENV: &str = "KAMN_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX";
const KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX_SECONDARY_ENV: &str =
    "KAMN_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX_SECONDARY";
const KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX_FALLBACK_ENV: &str =
    "KAMN_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX_FALLBACK";
const KOLME_LIVE_SIGNER_PUBLIC_KEY_HEX_ENV: &str = "KAMN_KOLME_LIVE_SIGNER_PUBLIC_KEY_HEX";
const KOLME_LIVE_SIGNER_PUBLIC_KEY_HEX_SECONDARY_ENV: &str =
    "KAMN_KOLME_LIVE_SIGNER_PUBLIC_KEY_HEX_SECONDARY";
const KOLME_LIVE_SIGNER_KEY_REF_ENV: &str = "KAMN_KOLME_LIVE_SIGNER_KEY_REF";
const KOLME_LIVE_SIGNER_KEY_REF_SECONDARY_ENV: &str = "KAMN_KOLME_LIVE_SIGNER_KEY_REF_SECONDARY";
const KOLME_LIVE_MANAGED_SIGNER_COMMAND_ENV: &str = "KAMN_KOLME_LIVE_MANAGED_SIGNER_COMMAND";
const KOLME_LIVE_MANAGED_SIGNER_REQUIRED_ENV: &str = "KAMN_KOLME_LIVE_MANAGED_SIGNER_REQUIRED";
const KOLME_LIVE_ALLOW_LOCAL_SIGNER_TESTING_ENV: &str =
    "KAMN_KOLME_LIVE_ALLOW_LOCAL_SIGNER_TESTING";
const KOLME_LIVE_ENV_LOCAL_SIGNER_KEY_SOURCE_FORBIDDEN_REASON_CODE: &str =
    "production_signer_key_source_env_local_forbidden";
const KOLME_LIVE_MANAGED_SIGNER_TIMEOUT_SECONDS_ENV: &str =
    "KAMN_KOLME_LIVE_MANAGED_SIGNER_TIMEOUT_SECONDS";
const KOLME_LIVE_MANAGED_SIGNER_TIMEOUT_SECONDS_DEFAULT: u64 = 5;
const KOLME_LIVE_MANAGED_SIGNER_POLL_INTERVAL_MILLIS: u64 = 10;
const KOLME_LIVE_NATIVE_CREATED_AT: &str = "2026-02-12T00:00:00Z";
const FULL_RUNTIME_BOOTSTRAP_COMPONENT_SEQUENCE: [&str; 4] =
    ["daemon", "api", "transport", "kolme-commit"];

#[cfg(test)]
pub(crate) fn daemon_test_env_lock() -> &'static std::sync::Mutex<()> {
    use std::sync::{Mutex, OnceLock};

    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| Mutex::new(()))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct OutputMode {
    kind: OutputModeKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum OutputModeKind {
    Text,
    Json,
}

impl OutputMode {
    fn text() -> Self {
        Self {
            kind: OutputModeKind::Text,
        }
    }

    fn json() -> Self {
        Self {
            kind: OutputModeKind::Json,
        }
    }

    fn parse(value: &str) -> Result<Self, ConfigError> {
        match value {
            "text" => Ok(Self::text()),
            "json" => Ok(Self::json()),
            other => Err(ConfigError::InvalidOutputMode(other.to_owned())),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct RuntimeMode {
    kind: RuntimeModeKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RuntimeModeKind {
    Bootstrap,
    Planning,
    RecoveryCheck,
    Daemon,
    Api,
    Full,
    KolmeLive,
}

impl RuntimeMode {
    fn bootstrap() -> Self {
        Self {
            kind: RuntimeModeKind::Bootstrap,
        }
    }

    fn planning() -> Self {
        Self {
            kind: RuntimeModeKind::Planning,
        }
    }

    fn recovery_check() -> Self {
        Self {
            kind: RuntimeModeKind::RecoveryCheck,
        }
    }

    fn daemon() -> Self {
        Self {
            kind: RuntimeModeKind::Daemon,
        }
    }

    fn api() -> Self {
        Self {
            kind: RuntimeModeKind::Api,
        }
    }

    fn full() -> Self {
        Self {
            kind: RuntimeModeKind::Full,
        }
    }

    fn kolme_live() -> Self {
        Self {
            kind: RuntimeModeKind::KolmeLive,
        }
    }

    fn parse(value: &str) -> Result<Self, ConfigError> {
        match value {
            "bootstrap" => Ok(Self::bootstrap()),
            "planning" => Ok(Self::planning()),
            "recovery-check" => Ok(Self::recovery_check()),
            "daemon" => Ok(Self::daemon()),
            "api" => Ok(Self::api()),
            "full" => Ok(Self::full()),
            "kolme-live" => Ok(Self::kolme_live()),
            other => Err(ConfigError::InvalidRuntimeMode(other.to_owned())),
        }
    }

    fn as_str(self) -> &'static str {
        match self.kind {
            RuntimeModeKind::Bootstrap => "bootstrap",
            RuntimeModeKind::Planning => "planning",
            RuntimeModeKind::RecoveryCheck => "recovery-check",
            RuntimeModeKind::Daemon => "daemon",
            RuntimeModeKind::Api => "api",
            RuntimeModeKind::Full => "full",
            RuntimeModeKind::KolmeLive => "kolme-live",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DiagnosticsMode {
    Basic,
    Snapshot,
}

impl DiagnosticsMode {
    fn basic() -> Self {
        Self::Basic
    }

    fn snapshot() -> Self {
        Self::Snapshot
    }

    fn parse(value: &str) -> Result<Self, ConfigError> {
        match value {
            "basic" => Ok(Self::basic()),
            "snapshot" => Ok(Self::snapshot()),
            other => Err(ConfigError::InvalidDiagnosticsMode(other.to_owned())),
        }
    }

    fn as_str(self) -> &'static str {
        match self {
            Self::Basic => "basic",
            Self::Snapshot => "snapshot",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum LocalProfile {
    Processor,
    Listener,
    Approver,
}

impl LocalProfile {
    fn parse(value: &str) -> Result<Self, ConfigError> {
        match value {
            "local-processor" => Ok(Self::Processor),
            "local-listener" => Ok(Self::Listener),
            "local-approver" => Ok(Self::Approver),
            other => Err(ConfigError::InvalidNodeProfile(other.to_owned())),
        }
    }

    fn as_str(self) -> &'static str {
        match self {
            Self::Processor => "local-processor",
            Self::Listener => "local-listener",
            Self::Approver => "local-approver",
        }
    }

    fn default_role(self) -> NodeRole {
        match self {
            Self::Processor => NodeRole::Processor,
            Self::Listener => NodeRole::Listener,
            Self::Approver => NodeRole::Approver,
        }
    }

    fn default_storage_dir(self) -> &'static str {
        match self {
            Self::Processor => "./data/processor",
            Self::Listener => "./data/listener",
            Self::Approver => "./data/approver",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct NodeCli {
    profile: Option<LocalProfile>,
    role: NodeRole,
    chain_id: String,
    chain_version: String,
    storage_dir: String,
    enable_gossip: bool,
    sync_mode: SyncMode,
    runtime_mode: RuntimeMode,
    expected_state_version: Option<u64>,
    expected_state_hash: Option<String>,
    proposals: Vec<ProposalCandidate>,
    rejoin_attempts: Vec<RejoinAttempt>,
    daemon_max_ticks: Option<u64>,
    daemon_tick_interval_ms: Option<u64>,
    daemon_shutdown_signal_ticks: Vec<u64>,
    daemon_shutdown_os_signals: bool,
    daemon_shutdown_drain_ticks: Option<u64>,
    daemon_shutdown_timeout_ticks: Option<u64>,
    daemon_peer_id: Option<String>,
    daemon_lifecycle_events: Vec<PeerLifecycleEvent>,
    kolme_live_base_url: Option<String>,
    kolme_live_provider_hint: Option<String>,
    kolme_live_signing_profile: Option<String>,
    kolme_live_strict_signer_contracts: bool,
    kolme_live_signer_profile: Option<String>,
    kolme_live_signer_key_source: Option<String>,
    api_bind_addr: Option<String>,
    api_max_requests: u64,
    api_idle_timeout_ms: u64,
    api_body_limit_bytes: u64,
    api_concurrency_limit: u64,
    api_rate_limit_per_second: u64,
    observability_endpoint_bind_addr: Option<String>,
    observability_endpoint_metrics_path: String,
    observability_endpoint_health_path: String,
    observability_endpoint_max_requests: u64,
    observability_endpoint_idle_timeout_ms: u64,
    output_mode: OutputMode,
    diagnostics_mode: DiagnosticsMode,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct PlanningExecution {
    expected_state_hash: String,
    candidate_count: usize,
    scheduled_candidate_ids: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct RecoveryExecution {
    expected_state_version: u64,
    expected_state_hash: String,
    attempt_count: usize,
    decisions: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct DaemonExecution {
    max_ticks: u64,
    tick_interval_ms: u64,
    executed_ticks: u64,
    completion_reason: String,
    observability_latency_p50_ms: u64,
    observability_latency_p99_ms: u64,
    observability_throughput_tps: u64,
    observability_error_rate_bps: u64,
    observability_availability_bps: u64,
    observability_health: String,
    observability_alert_count: usize,
    observability_reason_code: String,
    observability_transport_checkpoint_failures: u64,
    observability_signer_checkpoint_failures: u64,
    observability_commit_checkpoint_failures: u64,
    peer_id: Option<String>,
    peer_lifecycle_final_state: Option<String>,
    peer_lifecycle_applied_events: Option<Vec<String>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct DaemonRuntimeOptions {
    daemon_max_ticks: Option<u64>,
    daemon_tick_interval_ms: Option<u64>,
    daemon_shutdown_signal_ticks: Vec<u64>,
    daemon_shutdown_os_signals: bool,
    daemon_shutdown_drain_ticks: Option<u64>,
    daemon_shutdown_timeout_ticks: Option<u64>,
    daemon_peer_id: Option<String>,
    daemon_lifecycle_events: Vec<PeerLifecycleEvent>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct KolmeLiveExecution {
    provider_client_contract: String,
    base_url: String,
    provider_hint: String,
    signing_profile: String,
    signer_profile_selector_env: String,
    signer_profile: String,
    signer_key_source: String,
    signer_private_key_env: String,
    execution_status: String,
    observability_latency_p50_ms: u64,
    observability_latency_p99_ms: u64,
    observability_throughput_tps: u64,
    observability_error_rate_bps: u64,
    observability_availability_bps: u64,
    observability_health: String,
    observability_alert_count: usize,
    observability_reason_code: String,
    observability_transport_checkpoint_failures: u64,
    observability_signer_checkpoint_failures: u64,
    observability_commit_checkpoint_failures: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
struct RuntimeExecutionBundle {
    planning: Option<PlanningExecution>,
    recovery: Option<RecoveryExecution>,
    daemon: Option<DaemonExecution>,
    kolme_live: Option<KolmeLiveExecution>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct NodeBootstrapReport {
    runtime_mode: String,
    diagnostics_mode: String,
    component_count: usize,
    planning_expected_state_hash: Option<String>,
    planning_candidate_count: Option<usize>,
    planning_scheduled_candidate_ids: Option<Vec<String>>,
    recovery_expected_state_version: Option<u64>,
    recovery_expected_state_hash: Option<String>,
    recovery_attempt_count: Option<usize>,
    recovery_decisions: Option<Vec<String>>,
    daemon_max_ticks: Option<u64>,
    daemon_tick_interval_ms: Option<u64>,
    daemon_executed_ticks: Option<u64>,
    daemon_completion_reason: Option<String>,
    daemon_observability_latency_p50_ms: Option<u64>,
    daemon_observability_latency_p99_ms: Option<u64>,
    daemon_observability_throughput_tps: Option<u64>,
    daemon_observability_error_rate_bps: Option<u64>,
    daemon_observability_availability_bps: Option<u64>,
    daemon_observability_health: Option<String>,
    daemon_observability_alert_count: Option<usize>,
    daemon_observability_reason_code: Option<String>,
    daemon_observability_transport_checkpoint_failures: Option<u64>,
    daemon_observability_signer_checkpoint_failures: Option<u64>,
    daemon_observability_commit_checkpoint_failures: Option<u64>,
    daemon_peer_id: Option<String>,
    daemon_peer_lifecycle_final_state: Option<String>,
    daemon_peer_lifecycle_applied_events: Option<Vec<String>>,
    kolme_live_provider_client_contract: Option<String>,
    kolme_live_base_url: Option<String>,
    kolme_live_provider_hint: Option<String>,
    kolme_live_signing_profile: Option<String>,
    kolme_live_signer_profile_selector_env: Option<String>,
    kolme_live_signer_profile: Option<String>,
    kolme_live_signer_key_source: Option<String>,
    kolme_live_signer_private_key_env: Option<String>,
    kolme_live_execution_status: Option<String>,
    kolme_live_observability_latency_p50_ms: Option<u64>,
    kolme_live_observability_latency_p99_ms: Option<u64>,
    kolme_live_observability_throughput_tps: Option<u64>,
    kolme_live_observability_error_rate_bps: Option<u64>,
    kolme_live_observability_availability_bps: Option<u64>,
    kolme_live_observability_health: Option<String>,
    kolme_live_observability_alert_count: Option<usize>,
    kolme_live_observability_reason_code: Option<String>,
    kolme_live_observability_transport_checkpoint_failures: Option<u64>,
    kolme_live_observability_signer_checkpoint_failures: Option<u64>,
    kolme_live_observability_commit_checkpoint_failures: Option<u64>,
    profile: Option<String>,
    role: String,
    chain_id: String,
    chain_version: String,
    storage_dir: String,
    gossip_enabled: bool,
    sync_mode: String,
    sync_startup: String,
    sync_recovery: String,
    state_version: u32,
    pending_migrations: usize,
    components: Vec<String>,
}

fn run() -> Result<(), ConfigError> {
    let cli = parse_args(env::args())?;
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
    println!("{}", render_bootstrap_report(&report, output_mode));
    if let Some(endpoint_config) = service_api_endpoint_config {
        if report.runtime_mode != "api" && report.runtime_mode != "full" {
            return Err(ConfigError::RuntimeDaemonLifecycle(
                "service api endpoint requires runtime-mode api or full".to_owned(),
            ));
        }
        let snapshot = build_service_api_snapshot(&report);
        let max_requests_label = endpoint_config.max_requests.to_string();
        let idle_timeout_ms_label = endpoint_config.idle_timeout_ms.to_string();
        let body_limit_bytes_label = endpoint_config.body_limit_bytes.to_string();
        let concurrency_limit_label = endpoint_config.concurrency_limit.to_string();
        let rate_limit_per_second_label = endpoint_config.rate_limit_per_second.to_string();
        log_info(
            "node.runtime.service_api.endpoint.start",
            &[
                ("bind_addr", endpoint_config.bind_addr.as_str()),
                ("max_requests", max_requests_label.as_str()),
                ("idle_timeout_ms", idle_timeout_ms_label.as_str()),
                ("body_limit_bytes", body_limit_bytes_label.as_str()),
                ("concurrency_limit", concurrency_limit_label.as_str()),
                (
                    "rate_limit_per_second",
                    rate_limit_per_second_label.as_str(),
                ),
                ("execution_id", execution_id.as_str()),
            ],
        )?;
        serve_service_api_endpoint(&endpoint_config, &snapshot)
            .map_err(ConfigError::RuntimeDaemonLifecycle)?;
        log_info(
            "node.runtime.service_api.endpoint.complete",
            &[
                ("bind_addr", endpoint_config.bind_addr.as_str()),
                ("execution_id", execution_id.as_str()),
            ],
        )?;
    }
    if let Some(endpoint_config) = observability_endpoint_config {
        let snapshot = build_runtime_observability_snapshot(&report).ok_or_else(|| {
            ConfigError::RuntimeDaemonLifecycle(
                "observability endpoint export requires daemon or kolme-live telemetry".to_owned(),
            )
        })?;
        let max_requests_label = endpoint_config.max_requests.to_string();
        let idle_timeout_ms_label = endpoint_config.idle_timeout_ms.to_string();
        log_info(
            "node.runtime.observability.endpoint.start",
            &[
                ("bind_addr", endpoint_config.bind_addr.as_str()),
                ("metrics_path", endpoint_config.metrics_path.as_str()),
                ("health_path", endpoint_config.health_path.as_str()),
                ("max_requests", max_requests_label.as_str()),
                ("idle_timeout_ms", idle_timeout_ms_label.as_str()),
                ("execution_id", execution_id.as_str()),
            ],
        )?;
        serve_observability_endpoint(&endpoint_config, &snapshot)
            .map_err(ConfigError::RuntimeDaemonLifecycle)?;
        log_info(
            "node.runtime.observability.endpoint.complete",
            &[
                ("bind_addr", endpoint_config.bind_addr.as_str()),
                ("execution_id", execution_id.as_str()),
            ],
        )?;
    }

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
            eprintln!("{error}");
            ExitCode::FAILURE
        }
    }
}

#[cfg(test)]
mod cli_tests;
#[cfg(test)]
mod main_tests;
