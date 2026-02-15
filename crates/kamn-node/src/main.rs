use kamn_core::{
    bootstrap, ConfigError, DeterministicProposalPlanner, NodeConfig, NodeRole, PeerLifecycle,
    PeerLifecycleEvent, PeerLifecycleState, ProposalCandidate, RecoveryRejoinGuard, RecoveryStatus,
    RejoinAttempt, SyncMode,
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
mod service_api_endpoint;
mod signer;
mod wire_payload;

use cli::parse_args;
use daemon_observability::build_daemon_observability_telemetry;
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
    normalize_kolme_live_signer_key_source, normalize_kolme_live_signer_profile_selector,
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

fn daemon_lifecycle_event_as_str(event: PeerLifecycleEvent) -> &'static str {
    match event {
        PeerLifecycleEvent::StartConnect => "start-connect",
        PeerLifecycleEvent::HandshakeSucceeded => "handshake-succeeded",
        PeerLifecycleEvent::HeartbeatMissed => "heartbeat-missed",
        PeerLifecycleEvent::HeartbeatRestored => "heartbeat-restored",
        PeerLifecycleEvent::Disconnect => "disconnect",
        PeerLifecycleEvent::Rejoin => "rejoin",
    }
}

fn peer_lifecycle_state_as_str(state: PeerLifecycleState) -> &'static str {
    match state {
        PeerLifecycleState::Disconnected => "disconnected",
        PeerLifecycleState::Connecting => "connecting",
        PeerLifecycleState::Active => "active",
        PeerLifecycleState::Degraded => "degraded",
    }
}

fn build_runtime_execution_id(runtime_mode: RuntimeMode, chain_id: &str, role: &str) -> String {
    format!("node-runtime:{}:{chain_id}:{role}", runtime_mode.as_str())
}

fn daemon_shutdown_drain_status(completion_reason: &str) -> &'static str {
    if completion_reason.starts_with("graceful-shutdown:signal@") {
        "completed"
    } else if completion_reason.starts_with("graceful-shutdown-timeout:signal@") {
        "timeout"
    } else {
        "not-signaled"
    }
}

fn daemon_shutdown_signal_tick(completion_reason: &str) -> Option<&str> {
    completion_reason
        .strip_prefix("graceful-shutdown:signal@")
        .or_else(|| completion_reason.strip_prefix("graceful-shutdown-timeout:signal@"))
        .map(|value| value.split(';').next().unwrap_or(value))
}

fn daemon_shutdown_reason_field<'a>(completion_reason: &'a str, key: &str) -> Option<&'a str> {
    completion_reason.split(';').find_map(|segment| {
        let (field, value) = segment.split_once('=')?;
        if field == key {
            return Some(value);
        }
        None
    })
}

fn classify_full_bootstrap_component_contract_violation(
    components: &[&str],
) -> Option<&'static str> {
    if components.len() != FULL_RUNTIME_BOOTSTRAP_COMPONENT_SEQUENCE.len() {
        return Some("full_supervisor_bootstrap_component_count_mismatch");
    }
    for (component, expected_component) in components
        .iter()
        .zip(FULL_RUNTIME_BOOTSTRAP_COMPONENT_SEQUENCE.iter())
    {
        if component != expected_component {
            return Some("full_supervisor_bootstrap_component_order_mismatch");
        }
    }
    None
}

fn validate_full_bootstrap_component_contract(components: &[&str]) -> Result<(), ConfigError> {
    if let Some(reason) = classify_full_bootstrap_component_contract_violation(components) {
        return Err(ConfigError::RuntimeDaemonLifecycle(format!(
            "full_supervisor_invariant_violation:{reason}"
        )));
    }
    Ok(())
}

fn classify_full_supervisor_stop_contract_violation(
    completion_reason: &str,
    shutdown_drain_status: &str,
) -> Option<&'static str> {
    if !matches!(
        shutdown_drain_status,
        "completed" | "timeout" | "not-signaled"
    ) {
        return Some("full_supervisor_stop_invalid_shutdown_drain_status");
    }
    if completion_reason == "tick-budget-exhausted"
        || completion_reason.starts_with("tick-budget-exhausted;ignored_signals=")
    {
        if shutdown_drain_status != "not-signaled" {
            return Some("full_supervisor_stop_not_signaled_status_mismatch");
        }
        return None;
    }
    if completion_reason.starts_with("graceful-shutdown:signal@") {
        if daemon_shutdown_signal_tick(completion_reason).is_none() {
            return Some("full_supervisor_stop_missing_signal_tick");
        }
        if daemon_shutdown_reason_field(completion_reason, "drain_ticks").is_none() {
            return Some("full_supervisor_stop_missing_drain_ticks");
        }
        if daemon_shutdown_reason_field(completion_reason, "timeout_ticks").is_none() {
            return Some("full_supervisor_stop_missing_timeout_ticks");
        }
        if daemon_shutdown_reason_field(completion_reason, "ignored_signals").is_none() {
            return Some("full_supervisor_stop_missing_ignored_signals");
        }
        if shutdown_drain_status != "completed" {
            return Some("full_supervisor_stop_graceful_status_mismatch");
        }
        return None;
    }
    if completion_reason.starts_with("graceful-shutdown-timeout:signal@") {
        if daemon_shutdown_signal_tick(completion_reason).is_none() {
            return Some("full_supervisor_stop_missing_signal_tick");
        }
        if daemon_shutdown_reason_field(completion_reason, "drain_ticks").is_none() {
            return Some("full_supervisor_stop_missing_drain_ticks");
        }
        if daemon_shutdown_reason_field(completion_reason, "timeout_ticks").is_none() {
            return Some("full_supervisor_stop_missing_timeout_ticks");
        }
        if daemon_shutdown_reason_field(completion_reason, "ignored_signals").is_none() {
            return Some("full_supervisor_stop_missing_ignored_signals");
        }
        if shutdown_drain_status != "timeout" {
            return Some("full_supervisor_stop_graceful_timeout_status_mismatch");
        }
        return None;
    }
    Some("full_supervisor_stop_unknown_completion_reason")
}

fn validate_full_supervisor_stop_contract(
    completion_reason: &str,
    shutdown_drain_status: &str,
) -> Result<(), ConfigError> {
    if let Some(reason) =
        classify_full_supervisor_stop_contract_violation(completion_reason, shutdown_drain_status)
    {
        return Err(ConfigError::RuntimeDaemonLifecycle(format!(
            "full_supervisor_invariant_violation:{reason}"
        )));
    }
    Ok(())
}

fn execute_daemon_runtime(
    runtime_mode: RuntimeMode,
    execution_id: &str,
    options: DaemonRuntimeOptions,
) -> Result<DaemonExecution, ConfigError> {
    let DaemonRuntimeOptions {
        daemon_max_ticks,
        daemon_tick_interval_ms,
        daemon_shutdown_signal_ticks,
        daemon_shutdown_os_signals,
        daemon_shutdown_drain_ticks,
        daemon_shutdown_timeout_ticks,
        daemon_peer_id,
        daemon_lifecycle_events,
    } = options;
    let max_ticks =
        daemon_max_ticks.ok_or(ConfigError::MissingArgumentValue("--daemon-max-ticks"))?;
    let tick_interval_ms = daemon_tick_interval_ms.ok_or(ConfigError::MissingArgumentValue(
        "--daemon-tick-interval-ms",
    ))?;
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
    )?;
    let (peer_id, peer_lifecycle_final_state, peer_lifecycle_applied_events) = match daemon_peer_id
    {
        Some(peer_id) => {
            let mut lifecycle = PeerLifecycle::new(&peer_id)
                .map_err(|error| ConfigError::RuntimeDaemonLifecycle(error.to_string()))?;
            let mut applied_events = Vec::with_capacity(daemon_lifecycle_events.len());
            for event in daemon_lifecycle_events {
                lifecycle
                    .transition(event)
                    .map_err(|error| ConfigError::RuntimeDaemonLifecycle(error.to_string()))?;
                applied_events.push(daemon_lifecycle_event_as_str(event).to_owned());
            }
            (
                Some(peer_id),
                Some(peer_lifecycle_state_as_str(lifecycle.state()).to_owned()),
                Some(applied_events),
            )
        }
        None => (None, None, None),
    };
    let daemon_completion = if daemon_shutdown_os_signals && daemon_shutdown_signal_ticks.is_empty()
    {
        evaluate_daemon_completion_with_os_signals(
            max_ticks,
            tick_interval_ms,
            daemon_shutdown_drain_ticks,
            daemon_shutdown_timeout_ticks,
        )
        .map_err(|error| ConfigError::RuntimeDaemonLifecycle(error.to_string()))?
    } else {
        evaluate_daemon_completion(
            max_ticks,
            daemon_shutdown_signal_ticks.as_slice(),
            daemon_shutdown_drain_ticks,
            daemon_shutdown_timeout_ticks,
        )
    };
    let daemon_observability = build_daemon_observability_telemetry(
        tick_interval_ms,
        daemon_completion.completion_reason.as_str(),
    )
    .map_err(|error| ConfigError::RuntimeDaemonLifecycle(error.to_string()))?;
    let shutdown_drain_status =
        daemon_shutdown_drain_status(daemon_completion.completion_reason.as_str());
    let shutdown_signal_tick =
        daemon_shutdown_signal_tick(daemon_completion.completion_reason.as_str()).unwrap_or("none");
    let shutdown_drain_ticks =
        daemon_shutdown_reason_field(daemon_completion.completion_reason.as_str(), "drain_ticks")
            .unwrap_or("0");
    let shutdown_timeout_ticks = daemon_shutdown_reason_field(
        daemon_completion.completion_reason.as_str(),
        "timeout_ticks",
    )
    .unwrap_or("0");
    let shutdown_ignored_signals = daemon_shutdown_reason_field(
        daemon_completion.completion_reason.as_str(),
        "ignored_signals",
    )
    .unwrap_or("0");
    let executed_ticks_label = daemon_completion.executed_ticks.to_string();
    log_info(
        "node.runtime.daemon.execute.complete",
        &[
            ("runtime_mode", runtime_mode.as_str()),
            ("executed_ticks", executed_ticks_label.as_str()),
            (
                "completion_reason",
                daemon_completion.completion_reason.as_str(),
            ),
            ("shutdown_drain_status", shutdown_drain_status),
            ("shutdown_signal_tick", shutdown_signal_tick),
            ("shutdown_drain_ticks", shutdown_drain_ticks),
            ("shutdown_timeout_ticks", shutdown_timeout_ticks),
            ("shutdown_ignored_signals", shutdown_ignored_signals),
            ("execution_id", execution_id),
        ],
    )?;
    Ok(DaemonExecution {
        max_ticks,
        tick_interval_ms,
        executed_ticks: daemon_completion.executed_ticks,
        completion_reason: daemon_completion.completion_reason,
        observability_latency_p50_ms: daemon_observability.latency_p50_ms,
        observability_latency_p99_ms: daemon_observability.latency_p99_ms,
        observability_throughput_tps: daemon_observability.throughput_tps,
        observability_error_rate_bps: daemon_observability.error_rate_bps,
        observability_availability_bps: daemon_observability.availability_bps,
        observability_health: daemon_observability.health,
        observability_alert_count: daemon_observability.alert_count,
        peer_id,
        peer_lifecycle_final_state,
        peer_lifecycle_applied_events,
    })
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

fn resolve_kolme_live_allow_local_signer_testing_override() -> Result<bool, ConfigError> {
    match env::var(KOLME_LIVE_ALLOW_LOCAL_SIGNER_TESTING_ENV) {
        Ok(value) => {
            let trimmed = value.trim();
            if trimmed.is_empty() {
                return Err(ConfigError::RuntimeKolmeLive(format!(
                    "{KOLME_LIVE_ALLOW_LOCAL_SIGNER_TESTING_ENV} must not be empty when present (legacy_local_signer_path_override_invalid)"
                )));
            }
            match trimmed {
                "true" => Ok(true),
                "false" => Ok(false),
                _ => Err(ConfigError::RuntimeKolmeLive(format!(
                    "{KOLME_LIVE_ALLOW_LOCAL_SIGNER_TESTING_ENV} must be 'true' or 'false', found '{trimmed}' (legacy_local_signer_path_override_invalid)"
                ))),
            }
        }
        Err(env::VarError::NotPresent) => Ok(false),
        Err(env::VarError::NotUnicode(_)) => Err(ConfigError::RuntimeKolmeLive(format!(
            "{KOLME_LIVE_ALLOW_LOCAL_SIGNER_TESTING_ENV} must be valid utf-8 when present (legacy_local_signer_path_override_invalid)"
        ))),
    }
}

fn enforce_kolme_live_signer_contract_policy(
    strict_signer_contracts_enabled: bool,
    allow_local_signer_testing_override: bool,
    is_test_build: bool,
) -> Result<(), ConfigError> {
    if strict_signer_contracts_enabled || allow_local_signer_testing_override || is_test_build {
        return Ok(());
    }

    Err(ConfigError::RuntimeKolmeLive(format!(
        "--kolme-live-strict-signer-contracts is required for runtime-mode kolme-live; set {KOLME_LIVE_ALLOW_LOCAL_SIGNER_TESTING_ENV}=true only for explicit local testing override (legacy_local_signer_path_forbidden)"
    )))
}

fn classify_kolme_live_signer_key_source_policy_violation(
    strict_signer_contracts_enabled: bool,
    strict_signer_key_source: Option<&str>,
    allow_local_signer_testing_override: bool,
    is_test_build: bool,
) -> Option<&'static str> {
    if !strict_signer_contracts_enabled || allow_local_signer_testing_override || is_test_build {
        return None;
    }
    if strict_signer_key_source == Some(KOLME_LIVE_SIGNER_KEY_SOURCE_ENV_LOCAL) {
        return Some(KOLME_LIVE_ENV_LOCAL_SIGNER_KEY_SOURCE_FORBIDDEN_REASON_CODE);
    }
    None
}

fn enforce_kolme_live_signer_key_source_policy(
    strict_signer_contracts_enabled: bool,
    strict_signer_key_source: Option<&str>,
    allow_local_signer_testing_override: bool,
    is_test_build: bool,
) -> Result<(), ConfigError> {
    let reason_code = classify_kolme_live_signer_key_source_policy_violation(
        strict_signer_contracts_enabled,
        strict_signer_key_source,
        allow_local_signer_testing_override,
        is_test_build,
    );
    if let Some(reason_code) = reason_code {
        return Err(ConfigError::RuntimeKolmeLive(format!(
            "--kolme-live-signer-key-source={KOLME_LIVE_SIGNER_KEY_SOURCE_ENV_LOCAL} is not allowed when --kolme-live-strict-signer-contracts is enabled for production-targeted runs; use --kolme-live-signer-key-source={KOLME_LIVE_SIGNER_KEY_SOURCE_MANAGED_EXTERNAL} or set {KOLME_LIVE_ALLOW_LOCAL_SIGNER_TESTING_ENV}=true for explicit local testing override ({reason_code})"
        )));
    }
    Ok(())
}

fn execute(cli: NodeCli) -> Result<NodeBootstrapReport, ConfigError> {
    let NodeCli {
        profile,
        role,
        chain_id,
        chain_version,
        storage_dir,
        enable_gossip,
        sync_mode,
        runtime_mode,
        expected_state_version,
        expected_state_hash,
        proposals,
        rejoin_attempts,
        daemon_max_ticks,
        daemon_tick_interval_ms,
        daemon_shutdown_signal_ticks,
        daemon_shutdown_os_signals,
        daemon_shutdown_drain_ticks,
        daemon_shutdown_timeout_ticks,
        daemon_peer_id,
        daemon_lifecycle_events,
        kolme_live_base_url,
        kolme_live_provider_hint,
        kolme_live_signing_profile,
        kolme_live_strict_signer_contracts,
        kolme_live_signer_profile,
        kolme_live_signer_key_source,
        api_bind_addr: _,
        api_max_requests: _,
        api_idle_timeout_ms: _,
        api_body_limit_bytes: _,
        api_concurrency_limit: _,
        api_rate_limit_per_second: _,
        observability_endpoint_bind_addr: _,
        observability_endpoint_metrics_path: _,
        observability_endpoint_health_path: _,
        observability_endpoint_max_requests: _,
        observability_endpoint_idle_timeout_ms: _,
        output_mode: _,
        diagnostics_mode,
    } = cli;
    let execution_id = build_runtime_execution_id(runtime_mode, chain_id.as_str(), role.as_str());

    let config = NodeConfig {
        chain_id: chain_id.clone(),
        chain_version: chain_version.clone(),
        role,
        storage_dir: storage_dir.clone(),
        enable_gossip,
        sync_mode,
    };

    let plan = bootstrap(config)?;
    log_info(
        "node.runtime.mode.dispatch",
        &[
            ("runtime_mode", runtime_mode.as_str()),
            ("execution_id", execution_id.as_str()),
        ],
    )?;
    let runtime_execution = match runtime_mode.kind {
        RuntimeModeKind::Bootstrap => {
            log_info(
                "node.runtime.bootstrap.plan.ready",
                &[
                    ("chain_id", plan.config.chain_id.as_str()),
                    ("role", plan.config.role.as_str()),
                    ("execution_id", execution_id.as_str()),
                ],
            )?;
            RuntimeExecutionBundle::default()
        }
        RuntimeModeKind::Planning => {
            let expected_state_hash = expected_state_hash
                .ok_or(ConfigError::MissingArgumentValue("--expected-state-hash"))?;
            let planner = DeterministicProposalPlanner::new(&expected_state_hash);
            let proposal_plan = planner
                .plan(proposals)
                .map_err(|error| ConfigError::RuntimePlanner(error.to_string()))?;
            RuntimeExecutionBundle {
                planning: Some(PlanningExecution {
                    expected_state_hash,
                    candidate_count: proposal_plan.ordered_candidates().len(),
                    scheduled_candidate_ids: proposal_plan.ordered_candidate_ids(),
                }),
                ..RuntimeExecutionBundle::default()
            }
        }
        RuntimeModeKind::RecoveryCheck => {
            let expected_state_version = expected_state_version.ok_or(
                ConfigError::MissingArgumentValue("--expected-state-version"),
            )?;
            let expected_state_hash = expected_state_hash
                .ok_or(ConfigError::MissingArgumentValue("--expected-state-hash"))?;
            let mut guard = RecoveryRejoinGuard::new(expected_state_version, &expected_state_hash)
                .map_err(|error| ConfigError::RuntimeRecovery(error.to_string()))?;
            let mut decisions = Vec::with_capacity(rejoin_attempts.len());
            for attempt in rejoin_attempts {
                let decision = match guard
                    .evaluate(attempt)
                    .map_err(|error| ConfigError::RuntimeRecovery(error.to_string()))?
                {
                    RecoveryStatus::RejoinAccepted => "rejoin-accepted".to_owned(),
                    RecoveryStatus::CatchUpRequired {
                        from_version,
                        to_version,
                    } => {
                        format!("catch-up-required:{from_version}->{to_version}")
                    }
                };
                decisions.push(decision);
            }
            RuntimeExecutionBundle {
                recovery: Some(RecoveryExecution {
                    expected_state_version,
                    expected_state_hash,
                    attempt_count: decisions.len(),
                    decisions,
                }),
                ..RuntimeExecutionBundle::default()
            }
        }
        RuntimeModeKind::Daemon => {
            let daemon_execution = execute_daemon_runtime(
                runtime_mode,
                execution_id.as_str(),
                DaemonRuntimeOptions {
                    daemon_max_ticks,
                    daemon_tick_interval_ms,
                    daemon_shutdown_signal_ticks,
                    daemon_shutdown_os_signals,
                    daemon_shutdown_drain_ticks,
                    daemon_shutdown_timeout_ticks,
                    daemon_peer_id,
                    daemon_lifecycle_events,
                },
            )?;
            RuntimeExecutionBundle {
                daemon: Some(daemon_execution),
                ..RuntimeExecutionBundle::default()
            }
        }
        RuntimeModeKind::Api => {
            log_info(
                "node.runtime.service_api.mode.ready",
                &[
                    ("runtime_mode", runtime_mode.as_str()),
                    ("execution_id", execution_id.as_str()),
                ],
            )?;
            RuntimeExecutionBundle::default()
        }
        RuntimeModeKind::Full => {
            validate_full_bootstrap_component_contract(
                FULL_RUNTIME_BOOTSTRAP_COMPONENT_SEQUENCE.as_slice(),
            )?;
            log_info(
                "node.runtime.full.bootstrap.start",
                &[("execution_id", execution_id.as_str())],
            )?;
            for (stage_index, component) in FULL_RUNTIME_BOOTSTRAP_COMPONENT_SEQUENCE
                .into_iter()
                .enumerate()
            {
                let stage_index_label = (stage_index + 1).to_string();
                log_info(
                    "node.runtime.full.bootstrap.component.ready",
                    &[
                        ("component", component),
                        ("stage_index", stage_index_label.as_str()),
                        ("execution_id", execution_id.as_str()),
                    ],
                )?;
            }
            let daemon_execution = execute_daemon_runtime(
                runtime_mode,
                execution_id.as_str(),
                DaemonRuntimeOptions {
                    daemon_max_ticks,
                    daemon_tick_interval_ms,
                    daemon_shutdown_signal_ticks,
                    daemon_shutdown_os_signals,
                    daemon_shutdown_drain_ticks,
                    daemon_shutdown_timeout_ticks,
                    daemon_peer_id,
                    daemon_lifecycle_events,
                },
            )?;
            log_info(
                "node.runtime.full.bootstrap.ready",
                &[("execution_id", execution_id.as_str())],
            )?;
            let stop_reason = "daemon-execution-complete";
            let completion_reason = daemon_execution.completion_reason.as_str();
            let shutdown_drain_status = daemon_shutdown_drain_status(completion_reason);
            validate_full_supervisor_stop_contract(completion_reason, shutdown_drain_status)?;
            log_info(
                "node.runtime.full.supervisor.stop.requested",
                &[
                    ("stop_reason", stop_reason),
                    ("daemon_completion_reason", completion_reason),
                    ("execution_id", execution_id.as_str()),
                ],
            )?;
            log_info(
                "node.runtime.full.supervisor.stop.complete",
                &[
                    ("stop_reason", stop_reason),
                    ("daemon_completion_reason", completion_reason),
                    ("shutdown_drain_status", shutdown_drain_status),
                    ("execution_id", execution_id.as_str()),
                ],
            )?;
            RuntimeExecutionBundle {
                daemon: Some(daemon_execution),
                ..RuntimeExecutionBundle::default()
            }
        }
        RuntimeModeKind::KolmeLive => {
            let base_url = kolme_live_base_url
                .ok_or(ConfigError::MissingArgumentValue("--kolme-live-base-url"))?;
            let provider_hint = kolme_live_provider_hint.ok_or(
                ConfigError::MissingArgumentValue("--kolme-live-provider-hint"),
            )?;
            let signing_profile = kolme_live_signing_profile.ok_or(
                ConfigError::MissingArgumentValue("--kolme-live-signing-profile"),
            )?;
            let allow_local_signer_testing_override =
                resolve_kolme_live_allow_local_signer_testing_override()?;
            enforce_kolme_live_signer_contract_policy(
                kolme_live_strict_signer_contracts,
                allow_local_signer_testing_override,
                cfg!(test),
            )?;
            let declared_signer_profile = kolme_live_signer_profile
                .as_deref()
                .map(normalize_kolme_live_signer_profile_selector)
                .transpose()?;
            let declared_signer_key_source = kolme_live_signer_key_source
                .as_deref()
                .map(normalize_kolme_live_signer_key_source)
                .transpose()?;
            let strict_signer_profile = if kolme_live_strict_signer_contracts {
                Some(
                    declared_signer_profile.ok_or(ConfigError::MissingArgumentValue(
                        "--kolme-live-signer-profile",
                    ))?,
                )
            } else {
                declared_signer_profile
            };
            let strict_signer_key_source = if kolme_live_strict_signer_contracts {
                Some(
                    declared_signer_key_source.ok_or(ConfigError::MissingArgumentValue(
                        "--kolme-live-signer-key-source",
                    ))?,
                )
            } else {
                declared_signer_key_source
            };
            enforce_kolme_live_signer_key_source_policy(
                kolme_live_strict_signer_contracts,
                strict_signer_key_source,
                allow_local_signer_testing_override,
                cfg!(test),
            )?;
            let kolme_live_execution =
                if daemon_max_ticks.is_some() || daemon_tick_interval_ms.is_some() {
                    let max_cycles = daemon_max_ticks
                        .ok_or(ConfigError::MissingArgumentValue("--daemon-max-ticks"))?;
                    let cycle_interval_ms = daemon_tick_interval_ms.ok_or(
                        ConfigError::MissingArgumentValue("--daemon-tick-interval-ms"),
                    )?;
                    execute_kolme_live_runtime_continuous(
                        &plan,
                        base_url,
                        provider_hint,
                        signing_profile,
                        strict_signer_profile,
                        strict_signer_key_source,
                        KolmeLiveContinuousMode {
                            max_cycles,
                            cycle_interval_ms,
                        },
                    )?
                } else {
                    execute_kolme_live_runtime(
                        &plan,
                        base_url,
                        provider_hint,
                        signing_profile,
                        strict_signer_profile,
                        strict_signer_key_source,
                    )?
                };
            RuntimeExecutionBundle {
                kolme_live: Some(kolme_live_execution),
                ..RuntimeExecutionBundle::default()
            }
        }
    };
    let report = build_bootstrap_report(
        &plan,
        profile,
        diagnostics_mode,
        runtime_mode,
        runtime_execution,
    );

    Ok(report)
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
