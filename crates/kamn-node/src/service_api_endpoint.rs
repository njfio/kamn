use crate::{
    logging::{log_info, log_warn},
    NodeBootstrapReport,
};
use axum::{
    body::{to_bytes, Body, Bytes},
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        Request, State,
    },
    http::{HeaderMap, HeaderValue, StatusCode},
    middleware::{self, Next},
    response::{IntoResponse, Response},
    routing::{any, get},
    Extension, Router,
};
use kamn_core::{
    signature_matches_supported_profile_for_fields, AgentDid, AntiSpamConfig, AntiSpamDecision,
    AntiSpamEngine, AntiSpamRejection,
};
#[cfg(test)]
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::env;
use std::fs;
use std::io::BufReader;
use std::net::SocketAddr;
use std::sync::{
    atomic::{AtomicBool, AtomicU64, Ordering},
    Arc,
};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use tokio::runtime::Builder;
use tokio::sync::{Mutex, Notify, Semaphore};

pub(crate) const DEFAULT_SERVICE_API_MAX_REQUESTS: u64 = 1;
pub(crate) const DEFAULT_SERVICE_API_IDLE_TIMEOUT_MS: u64 = 5_000;
pub(crate) const DEFAULT_SERVICE_API_BODY_LIMIT_BYTES: u64 = 64 * 1024;
pub(crate) const DEFAULT_SERVICE_API_CONCURRENCY_LIMIT: u64 = 32;
pub(crate) const DEFAULT_SERVICE_API_RATE_LIMIT_PER_SECOND: u64 = 120;

const ROUTE_MESSAGES_SEND: &str = "/v1/messages/send";
const ROUTE_CHANNELS_CREATE: &str = "/v1/channels/create";
const ROUTE_TASKS_CREATE: &str = "/v1/tasks/create";
const ROUTE_MESSAGES_PREFIX: &str = "/v1/messages/";
const ROUTE_CHANNELS_PREFIX: &str = "/v1/channels/";
const ROUTE_CHANNELS_MESSAGES_SUFFIX: &str = "/messages";
const ROUTE_TASKS_PREFIX: &str = "/v1/tasks/";
const ROUTE_AGENTS_PREFIX: &str = "/v1/agents/";
const ROUTE_EVENTS_WS: &str = "/v1/events/ws";
const ROUTE_HEALTHZ: &str = "/healthz";
const ROUTE_METRICS: &str = "/metrics";
const REQUEST_AUTH_SENDER_DID_HEADER: &str = "x-kamn-sender-did";
const REQUEST_AUTH_NONCE_HEADER: &str = "x-kamn-request-nonce";
const REQUEST_AUTH_SIGNATURE_HEADER: &str = "x-kamn-request-signature";
const REASON_CODE_WEBSOCKET_UPGRADE_REQUIRED: &str = "service_api_websocket_upgrade_required";
const REASON_CODE_METHOD_NOT_ALLOWED: &str = "service_api_method_not_allowed";
const REASON_CODE_ROUTE_NOT_FOUND: &str = "service_api_route_not_found";
const REASON_CODE_REQUEST_READ_FAILED: &str = "service_api_request_read_failed";
const REASON_CODE_INGRESS_BODY_SIZE_LIMIT_EXCEEDED: &str =
    "service_api_ingress_body_size_limit_exceeded";
const REASON_CODE_INGRESS_CONCURRENCY_LIMIT_EXCEEDED: &str =
    "service_api_ingress_concurrency_limit_exceeded";
const REASON_CODE_INGRESS_RATE_LIMIT_EXCEEDED: &str = "service_api_ingress_rate_limit_exceeded";
const REASON_CODE_INGRESS_SENDER_RATE_LIMIT_EXCEEDED: &str =
    "service_api_ingress_sender_rate_limit_exceeded";
const REASON_CODE_INGRESS_SENDER_SUSPENDED: &str = "service_api_ingress_sender_suspended";
const REASON_CODE_INGRESS_SENDER_DUPLICATE_MESSAGE_ID: &str =
    "service_api_ingress_sender_duplicate_message_id";
const REASON_CODE_INGRESS_SENDER_INSUFFICIENT_DEPOSIT: &str =
    "service_api_ingress_sender_insufficient_deposit";
const REASON_CODE_INGRESS_ANTI_SPAM_ENGINE_INVALID: &str =
    "service_api_ingress_anti_spam_engine_invalid";
const REASON_CODE_REQUEST_HEADER_UTF8_INVALID: &str = "service_api_request_header_utf8_invalid";
const REASON_CODE_REQUEST_BODY_UTF8_INVALID: &str = "service_api_request_body_utf8_invalid";
const REASON_CODE_REQUEST_LOG_EMISSION_FAILED: &str = "service_api_request_log_emission_failed";
const REASON_CODE_AUTH_SENDER_DID_HEADER_MISSING: &str =
    "service_api_auth_sender_did_header_missing";
const REASON_CODE_AUTH_SENDER_DID_INVALID: &str = "service_api_auth_sender_did_invalid";
const REASON_CODE_AUTH_NONCE_HEADER_MISSING: &str = "service_api_auth_nonce_header_missing";
const REASON_CODE_AUTH_NONCE_INVALID: &str = "service_api_auth_nonce_invalid";
const REASON_CODE_AUTH_NONCE_NON_POSITIVE: &str = "service_api_auth_nonce_non_positive";
const REASON_CODE_AUTH_SIGNATURE_HEADER_MISSING: &str = "service_api_auth_signature_header_missing";
const REASON_CODE_AUTH_SIGNATURE_VERIFICATION_FAILED: &str =
    "service_api_auth_signature_verification_failed";
const REASON_CODE_AUTH_REPLAY_NONCE_DETECTED: &str = "service_api_auth_replay_nonce_detected";
const REASON_CODE_WS_UPGRADE_HEADER_MISSING: &str = "service_api_ws_upgrade_header_missing";
const REASON_CODE_WS_CONNECTION_HEADER_MISSING: &str = "service_api_ws_connection_header_missing";
const REASON_CODE_WS_KEY_HEADER_MISSING: &str = "service_api_ws_key_header_missing";
const REASON_CODE_WS_VERSION_HEADER_MISSING: &str = "service_api_ws_version_header_missing";
const REASON_CODE_WS_UPGRADE_HEADER_INVALID: &str = "service_api_ws_upgrade_header_invalid";
const REASON_CODE_WS_CONNECTION_HEADER_INVALID: &str = "service_api_ws_connection_header_invalid";
const REASON_CODE_WS_KEY_HEADER_EMPTY: &str = "service_api_ws_key_header_empty";
const REASON_CODE_WS_VERSION_HEADER_INVALID: &str = "service_api_ws_version_header_invalid";
const SERVICE_API_TLS_MODE_ENV: &str = "KAMN_SERVICE_API_TLS_MODE";
const SERVICE_API_TLS_CERT_FILE_ENV: &str = "KAMN_SERVICE_API_TLS_CERT_FILE";
const SERVICE_API_TLS_KEY_FILE_ENV: &str = "KAMN_SERVICE_API_TLS_KEY_FILE";
const SERVICE_API_TLS_MODE_DISABLED: &str = "disabled";
const SERVICE_API_TLS_MODE_REQUIRE: &str = "require";

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ServiceApiEndpointConfig {
    pub(crate) bind_addr: String,
    pub(crate) max_requests: u64,
    pub(crate) idle_timeout_ms: u64,
    pub(crate) body_limit_bytes: u64,
    pub(crate) concurrency_limit: u64,
    pub(crate) rate_limit_per_second: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ServiceApiSnapshot {
    pub(crate) runtime_mode: String,
    pub(crate) role: String,
    pub(crate) chain_id: String,
    pub(crate) chain_version: String,
    pub(crate) observability_source: String,
    pub(crate) observability_latency_p50_ms: u64,
    pub(crate) observability_latency_p99_ms: u64,
    pub(crate) observability_throughput_tps: u64,
    pub(crate) observability_error_rate_bps: u64,
    pub(crate) observability_availability_bps: u64,
    pub(crate) observability_health: String,
    pub(crate) observability_alert_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ServiceApiEndpointResponse {
    pub(crate) status_code: u16,
    pub(crate) content_type: &'static str,
    pub(crate) body: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct ServiceApiErrorBody {
    pub(crate) error: String,
    pub(crate) reason_code: String,
    pub(crate) message: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct ServiceApiHealthBody {
    pub(crate) status: String,
    pub(crate) runtime_mode: String,
    pub(crate) role: String,
    pub(crate) observability_source: String,
    pub(crate) observability_health: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct ServiceApiMessageCreateBody {
    pub(crate) message_id: String,
    pub(crate) status: String,
    pub(crate) runtime_mode: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct ServiceApiMessageGetBody {
    pub(crate) message_id: String,
    pub(crate) status: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct ServiceApiChannelCreateBody {
    pub(crate) channel_id: String,
    pub(crate) status: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct ServiceApiChannelMessagesBody {
    pub(crate) channel_id: String,
    pub(crate) messages: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct ServiceApiTaskCreateBody {
    pub(crate) task_id: String,
    pub(crate) state: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct ServiceApiTaskGetBody {
    pub(crate) task_id: String,
    pub(crate) state: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct ServiceApiAgentGetBody {
    pub(crate) did: String,
    pub(crate) reputation_score: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct ServiceApiWebsocketStateTransitionBody {
    pub(crate) event: String,
    pub(crate) runtime_mode: String,
    pub(crate) role: String,
    pub(crate) sequence: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ParsedRequest {
    method: String,
    path: String,
    body: String,
    headers: BTreeMap<String, String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum RequestAuthFailure {
    Unauthorized(ServiceApiReasonedError),
    Replay(ServiceApiReasonedError),
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ServiceApiReasonedError {
    reason_code: &'static str,
    message: String,
}

impl ServiceApiReasonedError {
    fn new(reason_code: &'static str, message: impl Into<String>) -> Self {
        Self {
            reason_code,
            message: message.into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum ServiceApiTlsMode {
    Disabled,
    Require { cert_file: String, key_file: String },
}

#[derive(Debug)]
struct ServiceApiRequestBudget {
    max_requests: u64,
    served_requests: AtomicU64,
    completion: Notify,
}

impl ServiceApiRequestBudget {
    fn new(max_requests: u64) -> Self {
        Self {
            max_requests,
            served_requests: AtomicU64::new(0),
            completion: Notify::new(),
        }
    }

    fn record_request(&self) {
        let served = self.served_requests.fetch_add(1, Ordering::SeqCst) + 1;
        if served >= self.max_requests {
            self.completion.notify_waiters();
        }
    }

    async fn wait_until_complete(&self) {
        loop {
            if self.served_requests.load(Ordering::SeqCst) >= self.max_requests {
                return;
            }
            self.completion.notified().await;
        }
    }
}

impl ServiceApiIngressRateWindow {
    fn new(max_requests_per_second: u64) -> Self {
        Self {
            window_start: Instant::now(),
            accepted_requests: 0,
            max_requests_per_second,
        }
    }

    fn try_record_request(&mut self, now: Instant) -> bool {
        if now.duration_since(self.window_start) >= Duration::from_secs(1) {
            self.window_start = now;
            self.accepted_requests = 0;
        }
        if self.accepted_requests >= self.max_requests_per_second {
            return false;
        }
        self.accepted_requests += 1;
        true
    }
}

#[derive(Debug)]
struct ServiceApiRuntimeState {
    snapshot: ServiceApiSnapshot,
    replay_guard: Arc<Mutex<BTreeSet<(String, u64)>>>,
    request_budget: Arc<ServiceApiRequestBudget>,
    body_limit_bytes: usize,
    concurrency_limiter: Arc<Semaphore>,
    ingress_rate_window: Arc<Mutex<ServiceApiIngressRateWindow>>,
    sender_anti_spam: Arc<Mutex<AntiSpamEngine>>,
}

#[derive(Debug)]
struct ServiceApiIngressRateWindow {
    window_start: Instant,
    accepted_requests: u64,
    max_requests_per_second: u64,
}

#[derive(Debug, Clone)]
struct ServiceApiRequestContext {
    parsed_request: ParsedRequest,
    correlation_id: String,
}

#[derive(Debug, Clone, Copy)]
struct ServiceApiRequestOutcome(&'static str);

#[derive(Debug, Clone)]
struct ServiceApiMiddlewareError<'a> {
    correlation_id: &'a str,
    method: &'a str,
    path: &'a str,
    status_code: StatusCode,
    error_label: &'a str,
    reason_code: &'a str,
    message: &'a str,
    outcome: &'a str,
}

pub(crate) fn build_service_api_snapshot(report: &NodeBootstrapReport) -> ServiceApiSnapshot {
    let observability = resolve_service_api_observability(report);
    ServiceApiSnapshot {
        runtime_mode: report.runtime_mode.clone(),
        role: report.role.clone(),
        chain_id: report.chain_id.clone(),
        chain_version: report.chain_version.clone(),
        observability_source: observability.source,
        observability_latency_p50_ms: observability.latency_p50_ms,
        observability_latency_p99_ms: observability.latency_p99_ms,
        observability_throughput_tps: observability.throughput_tps,
        observability_error_rate_bps: observability.error_rate_bps,
        observability_availability_bps: observability.availability_bps,
        observability_health: observability.health,
        observability_alert_count: observability.alert_count,
    }
}

pub(crate) fn render_service_api_endpoint_response(
    snapshot: &ServiceApiSnapshot,
    method: &str,
    path: &str,
    body: &str,
) -> ServiceApiEndpointResponse {
    if method == "GET" && path == ROUTE_HEALTHZ {
        let payload = ServiceApiHealthBody {
            status: "ok".to_owned(),
            runtime_mode: snapshot.runtime_mode.clone(),
            role: snapshot.role.clone(),
            observability_source: snapshot.observability_source.clone(),
            observability_health: snapshot.observability_health.clone(),
        };
        return ServiceApiEndpointResponse {
            status_code: 200,
            content_type: "application/json",
            body: serialize_service_api_json(&payload),
        };
    }
    if method == "GET" && path == ROUTE_METRICS {
        let health_value = if snapshot.observability_health == "healthy" {
            1
        } else {
            0
        };
        let metrics = format!(
            "kamn_service_api_health{{runtime_mode=\"{}\"}} 1\nkamn_service_api_role{{role=\"{}\"}} 1\nkamn_service_api_chain_info{{chain_id=\"{}\",chain_version=\"{}\"}} 1\nkamn_service_api_observability_latency_p50_ms {}\nkamn_service_api_observability_latency_p99_ms {}\nkamn_service_api_observability_throughput_tps {}\nkamn_service_api_observability_error_rate_bps {}\nkamn_service_api_observability_availability_bps {}\nkamn_service_api_observability_alert_count {}\nkamn_service_api_observability_source{{source=\"{}\"}} 1\nkamn_service_api_observability_health{{health=\"{}\"}} {}\n",
            escape_metrics_label(snapshot.runtime_mode.as_str()),
            escape_metrics_label(snapshot.role.as_str()),
            escape_metrics_label(snapshot.chain_id.as_str()),
            escape_metrics_label(snapshot.chain_version.as_str()),
            snapshot.observability_latency_p50_ms,
            snapshot.observability_latency_p99_ms,
            snapshot.observability_throughput_tps,
            snapshot.observability_error_rate_bps,
            snapshot.observability_availability_bps,
            snapshot.observability_alert_count,
            escape_metrics_label(snapshot.observability_source.as_str()),
            escape_metrics_label(snapshot.observability_health.as_str()),
            health_value,
        );
        return ServiceApiEndpointResponse {
            status_code: 200,
            content_type: "text/plain; version=0.0.4",
            body: metrics,
        };
    }
    if method == "GET" && path == ROUTE_EVENTS_WS {
        return json_error_endpoint_response(
            StatusCode::BAD_REQUEST,
            "bad-request",
            REASON_CODE_WEBSOCKET_UPGRADE_REQUIRED,
            "websocket upgrade required",
        );
    }
    if method == "POST" && path == ROUTE_MESSAGES_SEND {
        let message_id = format!("msg-local-{}", deterministic_body_tag(body.as_bytes()));
        let payload = ServiceApiMessageCreateBody {
            message_id,
            status: "created".to_owned(),
            runtime_mode: snapshot.runtime_mode.clone(),
        };
        return ServiceApiEndpointResponse {
            status_code: 202,
            content_type: "application/json",
            body: serialize_service_api_json(&payload),
        };
    }
    if method == "POST" && path == ROUTE_CHANNELS_CREATE {
        let channel_id = format!("channel-local-{}", deterministic_body_tag(body.as_bytes()));
        let payload = ServiceApiChannelCreateBody {
            channel_id,
            status: "created".to_owned(),
        };
        return ServiceApiEndpointResponse {
            status_code: 201,
            content_type: "application/json",
            body: serialize_service_api_json(&payload),
        };
    }
    if method == "POST" && path == ROUTE_TASKS_CREATE {
        let task_id = format!("task-local-{}", deterministic_body_tag(body.as_bytes()));
        let payload = ServiceApiTaskCreateBody {
            task_id,
            state: "submitted".to_owned(),
        };
        return ServiceApiEndpointResponse {
            status_code: 201,
            content_type: "application/json",
            body: serialize_service_api_json(&payload),
        };
    }
    if method == "GET" {
        if let Some(message_id) = message_path_id(path) {
            let payload = ServiceApiMessageGetBody {
                message_id: message_id.to_owned(),
                status: "created".to_owned(),
            };
            return ServiceApiEndpointResponse {
                status_code: 200,
                content_type: "application/json",
                body: serialize_service_api_json(&payload),
            };
        }
        if let Some(channel_id) = channel_messages_path_id(path) {
            let payload = ServiceApiChannelMessagesBody {
                channel_id: channel_id.to_owned(),
                messages: Vec::new(),
            };
            return ServiceApiEndpointResponse {
                status_code: 200,
                content_type: "application/json",
                body: serialize_service_api_json(&payload),
            };
        }
        if let Some(task_id) = task_path_id(path) {
            let payload = ServiceApiTaskGetBody {
                task_id: task_id.to_owned(),
                state: "submitted".to_owned(),
            };
            return ServiceApiEndpointResponse {
                status_code: 200,
                content_type: "application/json",
                body: serialize_service_api_json(&payload),
            };
        }
        if let Some(agent_did) = agent_path_id(path) {
            let payload = ServiceApiAgentGetBody {
                did: agent_did.to_owned(),
                reputation_score: 500,
            };
            return ServiceApiEndpointResponse {
                status_code: 200,
                content_type: "application/json",
                body: serialize_service_api_json(&payload),
            };
        }
    }

    if route_exists_for_other_method(path) {
        return json_error_endpoint_response(
            StatusCode::METHOD_NOT_ALLOWED,
            "method-not-allowed",
            REASON_CODE_METHOD_NOT_ALLOWED,
            "method not allowed",
        );
    }
    json_error_endpoint_response(
        StatusCode::NOT_FOUND,
        "not-found",
        REASON_CODE_ROUTE_NOT_FOUND,
        "not found",
    )
}

pub(crate) fn serve_service_api_endpoint(
    config: &ServiceApiEndpointConfig,
    snapshot: &ServiceApiSnapshot,
) -> Result<(), String> {
    if config.max_requests == 0 {
        return Err("service api max requests must be greater than zero".to_owned());
    }
    if config.idle_timeout_ms == 0 {
        return Err("service api idle timeout must be greater than zero".to_owned());
    }
    if config.body_limit_bytes == 0 {
        return Err("service api body limit bytes must be greater than zero".to_owned());
    }
    if config.concurrency_limit == 0 {
        return Err("service api concurrency limit must be greater than zero".to_owned());
    }
    if config.rate_limit_per_second == 0 {
        return Err("service api rate limit per second must be greater than zero".to_owned());
    }
    if config.body_limit_bytes > usize::MAX as u64 {
        return Err("service api body limit bytes exceed platform usize range".to_owned());
    }
    if config.concurrency_limit > usize::MAX as u64 {
        return Err("service api concurrency limit exceeds platform usize range".to_owned());
    }

    let runtime = Builder::new_current_thread()
        .enable_io()
        .enable_time()
        .build()
        .map_err(|error| format!("service api runtime init failed: {error}"))?;
    runtime.block_on(serve_service_api_endpoint_async(
        config.clone(),
        snapshot.clone(),
    ))
}

fn resolve_service_api_tls_mode() -> Result<ServiceApiTlsMode, String> {
    match env::var(SERVICE_API_TLS_MODE_ENV) {
        Ok(value) => {
            let mode = value.trim().to_ascii_lowercase();
            if mode.is_empty() {
                return Err(format!(
                    "service api tls mode env must not be empty: {SERVICE_API_TLS_MODE_ENV}"
                ));
            }
            match mode.as_str() {
                SERVICE_API_TLS_MODE_DISABLED => Ok(ServiceApiTlsMode::Disabled),
                SERVICE_API_TLS_MODE_REQUIRE => {
                    let cert_file = env::var(SERVICE_API_TLS_CERT_FILE_ENV)
                        .map_err(|_| {
                            format!(
                                "service api tls mode requires env: {SERVICE_API_TLS_CERT_FILE_ENV}"
                            )
                        })?
                        .trim()
                        .to_owned();
                    if cert_file.is_empty() {
                        return Err(format!(
                            "service api tls cert env must not be empty: {SERVICE_API_TLS_CERT_FILE_ENV}"
                        ));
                    }
                    let key_file = env::var(SERVICE_API_TLS_KEY_FILE_ENV)
                        .map_err(|_| {
                            format!(
                                "service api tls mode requires env: {SERVICE_API_TLS_KEY_FILE_ENV}"
                            )
                        })?
                        .trim()
                        .to_owned();
                    if key_file.is_empty() {
                        return Err(format!(
                            "service api tls key env must not be empty: {SERVICE_API_TLS_KEY_FILE_ENV}"
                        ));
                    }
                    validate_service_api_tls_materials(cert_file.as_str(), key_file.as_str())?;
                    Ok(ServiceApiTlsMode::Require {
                        cert_file,
                        key_file,
                    })
                }
                other => Err(format!(
                    "service api tls mode is invalid: {other} (supported: {SERVICE_API_TLS_MODE_DISABLED},{SERVICE_API_TLS_MODE_REQUIRE})"
                )),
            }
        }
        Err(env::VarError::NotPresent) => Ok(ServiceApiTlsMode::Disabled),
        Err(env::VarError::NotUnicode(_)) => Err(format!(
            "service api tls mode env must be utf-8: {SERVICE_API_TLS_MODE_ENV}"
        )),
    }
}

fn validate_service_api_tls_materials(cert_file: &str, key_file: &str) -> Result<(), String> {
    let cert_bytes = fs::read(cert_file).map_err(|error| {
        format!("service api tls certificate file read failed: {cert_file}: {error}")
    })?;
    let key_bytes = fs::read(key_file)
        .map_err(|error| format!("service api tls key file read failed: {key_file}: {error}"))?;

    let mut cert_reader = BufReader::new(cert_bytes.as_slice());
    let certs = rustls_pemfile::certs(&mut cert_reader)
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| {
            format!("service api tls certificate file parse failed: {cert_file}: {error}")
        })?;
    if certs.is_empty() {
        return Err(format!(
            "service api tls certificate file parse failed: {cert_file}: no certificates found"
        ));
    }

    let mut key_reader = BufReader::new(key_bytes.as_slice());
    let private_key = rustls_pemfile::private_key(&mut key_reader)
        .map_err(|error| format!("service api tls key file parse failed: {key_file}: {error}"))?;
    if private_key.is_none() {
        return Err(format!(
            "service api tls key file parse failed: {key_file}: no private key found"
        ));
    }
    Ok(())
}

async fn serve_service_api_endpoint_async(
    config: ServiceApiEndpointConfig,
    snapshot: ServiceApiSnapshot,
) -> Result<(), String> {
    let tls_mode = resolve_service_api_tls_mode()?;
    let sender_anti_spam = build_service_api_sender_anti_spam_engine()
        .map_err(|error| format!("service api anti-spam init failed: {error}"))?;

    let runtime_state = Arc::new(ServiceApiRuntimeState {
        snapshot,
        replay_guard: Arc::new(Mutex::new(BTreeSet::new())),
        request_budget: Arc::new(ServiceApiRequestBudget::new(config.max_requests)),
        body_limit_bytes: config.body_limit_bytes as usize,
        concurrency_limiter: Arc::new(Semaphore::new(config.concurrency_limit as usize)),
        ingress_rate_window: Arc::new(Mutex::new(ServiceApiIngressRateWindow::new(
            config.rate_limit_per_second,
        ))),
        sender_anti_spam: Arc::new(Mutex::new(sender_anti_spam)),
    });
    let request_budget_shared = runtime_state.request_budget.clone();
    let timeout_reached = Arc::new(AtomicBool::new(false));
    let deadline = Instant::now() + Duration::from_millis(config.idle_timeout_ms);

    let app = build_service_api_router(runtime_state);

    match tls_mode {
        ServiceApiTlsMode::Disabled => {
            let request_budget = request_budget_shared.clone();
            let timeout_flag = timeout_reached.clone();
            let listener = tokio::net::TcpListener::bind(config.bind_addr.as_str())
                .await
                .map_err(|error| format!("service api bind failed: {error}"))?;
            axum::serve(listener, app.clone())
                .with_graceful_shutdown(async move {
                    let wait_for_budget = request_budget.wait_until_complete();
                    tokio::pin!(wait_for_budget);
                    let idle_timeout = tokio::time::sleep_until(deadline.into());
                    tokio::pin!(idle_timeout);
                    tokio::select! {
                        _ = &mut wait_for_budget => {}
                        _ = &mut idle_timeout => {
                            timeout_flag.store(true, Ordering::SeqCst);
                        }
                    }
                })
                .await
                .map_err(|error| format!("service api serve failed: {error}"))?;
        }
        ServiceApiTlsMode::Require {
            cert_file,
            key_file,
        } => {
            let request_budget = request_budget_shared.clone();
            let timeout_flag = timeout_reached.clone();
            let bind_addr = config.bind_addr.parse::<SocketAddr>().map_err(|error| {
                format!(
                    "service api tls bind address parse failed: {}: {error}",
                    config.bind_addr
                )
            })?;
            let rustls_config = axum_server::tls_rustls::RustlsConfig::from_pem_file(
                cert_file.clone(),
                key_file.clone(),
            )
            .await
            .map_err(|error| {
                format!(
                    "service api tls config load failed: cert_file={cert_file}, key_file={key_file}: {error}"
                )
            })?;

            let handle = axum_server::Handle::new();
            let shutdown_handle = handle.clone();
            tokio::spawn(async move {
                let wait_for_budget = request_budget.wait_until_complete();
                tokio::pin!(wait_for_budget);
                let idle_timeout = tokio::time::sleep_until(deadline.into());
                tokio::pin!(idle_timeout);
                tokio::select! {
                    _ = &mut wait_for_budget => {}
                    _ = &mut idle_timeout => {
                        timeout_flag.store(true, Ordering::SeqCst);
                    }
                }
                shutdown_handle.graceful_shutdown(None);
            });

            axum_server::bind_rustls(bind_addr, rustls_config)
                .handle(handle)
                .serve(app.clone().into_make_service())
                .await
                .map_err(|error| format!("service api tls serve failed: {error}"))?;
        }
    }

    if timeout_reached.load(Ordering::SeqCst) {
        return Err(format!(
            "service api timed out after {} ms waiting for requests",
            config.idle_timeout_ms
        ));
    }

    Ok(())
}

fn build_service_api_sender_anti_spam_engine() -> Result<AntiSpamEngine, String> {
    let config = AntiSpamConfig {
        minimum_sybil_deposit: 0,
        ..AntiSpamConfig::default()
    };
    AntiSpamEngine::new(config).map_err(|error| error.to_string())
}

fn build_service_api_router(state: Arc<ServiceApiRuntimeState>) -> Router {
    Router::new()
        .route(ROUTE_EVENTS_WS, get(handle_service_api_websocket_route))
        .route("/", any(handle_service_api_http_route))
        .route("/{*path}", any(handle_service_api_http_route))
        .layer(middleware::from_fn_with_state(
            state.clone(),
            service_api_auth_middleware,
        ))
        .with_state(state)
}

async fn service_api_auth_middleware(
    State(state): State<Arc<ServiceApiRuntimeState>>,
    request: Request,
    next: Next,
) -> Response {
    let method_label = request.method().to_string();
    let path = request.uri().path().to_owned();
    let is_websocket_route = method_label == "GET" && path == ROUTE_EVENTS_WS;
    let concurrency_permit = match state.concurrency_limiter.clone().try_acquire_owned() {
        Ok(permit) => permit,
        Err(_) => {
            let correlation_id = format!(
                "service-api:{}:{}:concurrency-limit",
                method_label.to_ascii_lowercase(),
                path
            );
            return service_api_middleware_error_response(
                &state,
                ServiceApiMiddlewareError {
                    correlation_id: correlation_id.as_str(),
                    method: method_label.as_str(),
                    path: path.as_str(),
                    status_code: StatusCode::TOO_MANY_REQUESTS,
                    error_label: "too-many-requests",
                    reason_code: REASON_CODE_INGRESS_CONCURRENCY_LIMIT_EXCEEDED,
                    message: "ingress concurrency limit exceeded",
                    outcome: "concurrency-limit",
                },
            );
        }
    };
    // Yield once so queued requests observe bounded in-flight concurrency on the
    // single-thread runtime and deterministically fail closed when over budget.
    tokio::task::yield_now().await;

    let (mut request, parsed_request) = match parse_service_api_request(
        request,
        is_websocket_route,
        state.body_limit_bytes,
    )
    .await
    {
        Ok(parsed_request) => parsed_request,
        Err(error) => {
            let correlation_id = format!(
                "service-api:parse-error:{:016x}",
                deterministic_body_tag(error.message.as_bytes())
            );
            return service_api_middleware_error_response(
                &state,
                ServiceApiMiddlewareError {
                    correlation_id: correlation_id.as_str(),
                    method: "unknown",
                    path: "unknown",
                    status_code: StatusCode::BAD_REQUEST,
                    error_label: "bad-request",
                    reason_code: error.reason_code,
                    message: error.message.as_str(),
                    outcome: "bad-request",
                },
            );
        }
    };

    let correlation_id = service_api_request_correlation_id(&parsed_request);
    if let Err(reason) = log_service_api_event_info(
        "service.api.request.received",
        &[
            ("correlation_id", correlation_id.as_str()),
            ("method", parsed_request.method.as_str()),
            ("path", parsed_request.path.as_str()),
        ],
    ) {
        return service_api_middleware_error_response(
            &state,
            ServiceApiMiddlewareError {
                correlation_id: correlation_id.as_str(),
                method: parsed_request.method.as_str(),
                path: parsed_request.path.as_str(),
                status_code: StatusCode::INTERNAL_SERVER_ERROR,
                error_label: "internal",
                reason_code: REASON_CODE_REQUEST_LOG_EMISSION_FAILED,
                message: reason.as_str(),
                outcome: "log-error",
            },
        );
    }

    {
        let mut replay_guard = state.replay_guard.lock().await;
        if let Err(error) =
            authorize_service_api_request(&state.snapshot, &parsed_request, &mut replay_guard)
        {
            let (status_code, error_label, auth_error, outcome) = match error {
                RequestAuthFailure::Unauthorized(reasoned_error) => (
                    StatusCode::UNAUTHORIZED,
                    "unauthorized",
                    reasoned_error,
                    "unauthorized",
                ),
                RequestAuthFailure::Replay(reasoned_error) => {
                    (StatusCode::CONFLICT, "replay", reasoned_error, "replay")
                }
            };
            return service_api_middleware_error_response(
                &state,
                ServiceApiMiddlewareError {
                    correlation_id: correlation_id.as_str(),
                    method: parsed_request.method.as_str(),
                    path: parsed_request.path.as_str(),
                    status_code,
                    error_label,
                    reason_code: auth_error.reason_code,
                    message: auth_error.message.as_str(),
                    outcome,
                },
            );
        }
    }

    if let Err(error) = enforce_sender_anti_spam(&state, &parsed_request).await {
        let (status_code, error_label, outcome) =
            if error.reason_code == REASON_CODE_INGRESS_ANTI_SPAM_ENGINE_INVALID {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "internal",
                    "anti-spam-error",
                )
            } else {
                (
                    StatusCode::TOO_MANY_REQUESTS,
                    "too-many-requests",
                    "anti-spam",
                )
            };
        return service_api_middleware_error_response(
            &state,
            ServiceApiMiddlewareError {
                correlation_id: correlation_id.as_str(),
                method: parsed_request.method.as_str(),
                path: parsed_request.path.as_str(),
                status_code,
                error_label,
                reason_code: error.reason_code,
                message: error.message.as_str(),
                outcome,
            },
        );
    }

    if route_requires_auth(parsed_request.method.as_str(), parsed_request.path.as_str()) {
        let mut ingress_rate_window = state.ingress_rate_window.lock().await;
        if !ingress_rate_window.try_record_request(Instant::now()) {
            return service_api_middleware_error_response(
                &state,
                ServiceApiMiddlewareError {
                    correlation_id: correlation_id.as_str(),
                    method: parsed_request.method.as_str(),
                    path: parsed_request.path.as_str(),
                    status_code: StatusCode::TOO_MANY_REQUESTS,
                    error_label: "too-many-requests",
                    reason_code: REASON_CODE_INGRESS_RATE_LIMIT_EXCEEDED,
                    message: "ingress rate limit exceeded",
                    outcome: "rate-limit",
                },
            );
        }
    }

    if let Err(error) =
        validate_websocket_route_requirements(is_websocket_route, &parsed_request.headers)
    {
        return service_api_middleware_error_response(
            &state,
            ServiceApiMiddlewareError {
                correlation_id: correlation_id.as_str(),
                method: parsed_request.method.as_str(),
                path: parsed_request.path.as_str(),
                status_code: StatusCode::BAD_REQUEST,
                error_label: "bad-request",
                reason_code: error.reason_code,
                message: error.message.as_str(),
                outcome: "websocket-bad-request",
            },
        );
    }

    let method_for_outcome = parsed_request.method.clone();
    let path_for_outcome = parsed_request.path.clone();
    request.extensions_mut().insert(ServiceApiRequestContext {
        parsed_request,
        correlation_id: correlation_id.clone(),
    });

    let response = next.run(request).await;
    let outcome = response
        .extensions()
        .get::<ServiceApiRequestOutcome>()
        .map(|outcome| outcome.0)
        .unwrap_or_else(|| {
            if is_websocket_route && response.status().is_client_error() {
                "websocket-bad-request"
            } else {
                "handled"
            }
        });
    let _ = emit_service_api_request_outcome(
        correlation_id.as_str(),
        method_for_outcome.as_str(),
        path_for_outcome.as_str(),
        response.status().as_u16(),
        outcome,
    );
    state.request_budget.record_request();
    drop(concurrency_permit);
    response
}

async fn parse_service_api_request(
    request: Request,
    is_websocket_route: bool,
    body_limit_bytes: usize,
) -> Result<(Request, ParsedRequest), ServiceApiReasonedError> {
    let method_label = request.method().to_string();
    let path = request.uri().path().to_owned();
    let headers = request.headers().clone();

    if is_websocket_route {
        let parsed_request =
            build_parsed_request(method_label.as_str(), path.as_str(), &headers, Bytes::new())?;
        return Ok((request, parsed_request));
    }

    let (parts, body) = request.into_parts();
    let body_limit = body_limit_bytes;
    let body = to_bytes(body, body_limit).await.map_err(|error| {
        let message = error.to_string();
        if message.contains("length limit exceeded") {
            ServiceApiReasonedError::new(
                REASON_CODE_INGRESS_BODY_SIZE_LIMIT_EXCEEDED,
                format!("request body size limit exceeded: {body_limit} bytes"),
            )
        } else {
            ServiceApiReasonedError::new(
                REASON_CODE_REQUEST_READ_FAILED,
                format!("request read failed: {error}"),
            )
        }
    })?;
    let parsed_request =
        build_parsed_request(method_label.as_str(), path.as_str(), &headers, body.clone())?;
    let request = Request::from_parts(parts, Body::from(body));
    Ok((request, parsed_request))
}

fn service_api_middleware_error_response(
    state: &ServiceApiRuntimeState,
    error: ServiceApiMiddlewareError<'_>,
) -> Response {
    let response = json_error_response(
        error.status_code,
        error.error_label,
        error.reason_code,
        error.message,
    );
    let _ = emit_service_api_request_outcome(
        error.correlation_id,
        error.method,
        error.path,
        error.status_code.as_u16(),
        error.outcome,
    );
    state.request_budget.record_request();
    response
}

async fn handle_service_api_http_route(
    State(state): State<Arc<ServiceApiRuntimeState>>,
    Extension(context): Extension<ServiceApiRequestContext>,
) -> Response {
    let _ = context.correlation_id.as_str();
    let rendered = render_service_api_endpoint_response(
        &state.snapshot,
        context.parsed_request.method.as_str(),
        context.parsed_request.path.as_str(),
        context.parsed_request.body.as_str(),
    );
    contract_response(rendered)
}

async fn handle_service_api_websocket_route(
    State(state): State<Arc<ServiceApiRuntimeState>>,
    Extension(context): Extension<ServiceApiRequestContext>,
    upgrade: WebSocketUpgrade,
) -> Response {
    let _ = context.correlation_id.as_str();
    let mut response = websocket_upgrade_response(upgrade, state.snapshot.clone());
    response
        .extensions_mut()
        .insert(ServiceApiRequestOutcome("websocket-upgrade"));
    response
}
fn route_requires_auth(method: &str, path: &str) -> bool {
    !(method == "GET" && (path == ROUTE_HEALTHZ || path == ROUTE_METRICS))
}

fn log_service_api_event_info(event: &str, fields: &[(&str, &str)]) -> Result<(), String> {
    log_info(event, fields).map_err(|error| format!("service api log emission failed: {error}"))
}

fn log_service_api_event_warn(event: &str, fields: &[(&str, &str)]) -> Result<(), String> {
    log_warn(event, fields).map_err(|error| format!("service api log emission failed: {error}"))
}

fn service_api_request_correlation_id(request: &ParsedRequest) -> String {
    let method = request.method.to_ascii_lowercase();
    if let (Some(sender_did), Some(nonce)) = (
        header_value(&request.headers, REQUEST_AUTH_SENDER_DID_HEADER),
        header_value(&request.headers, REQUEST_AUTH_NONCE_HEADER),
    ) {
        return format!("service-api:{method}:{}:{sender_did}:{nonce}", request.path);
    }
    let request_tag = deterministic_body_tag(request.body.as_bytes());
    format!("service-api:{method}:{}:{request_tag:016x}", request.path)
}

fn emit_service_api_request_outcome(
    correlation_id: &str,
    method: &str,
    path: &str,
    status_code: u16,
    outcome: &str,
) -> Result<(), String> {
    let status_code_label = status_code.to_string();
    let fields = [
        ("correlation_id", correlation_id),
        ("method", method),
        ("path", path),
        ("status_code", status_code_label.as_str()),
        ("outcome", outcome),
    ];
    if status_code >= 400 {
        log_service_api_event_warn("service.api.request.outcome", &fields)
    } else {
        log_service_api_event_info("service.api.request.outcome", &fields)
    }
}

fn service_api_signature_state_hash(snapshot: &ServiceApiSnapshot) -> String {
    format!(
        "service-api:{}:{}",
        snapshot.chain_id.as_str(),
        snapshot.chain_version.as_str()
    )
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ServiceApiObservabilitySnapshot {
    source: String,
    latency_p50_ms: u64,
    latency_p99_ms: u64,
    throughput_tps: u64,
    error_rate_bps: u64,
    availability_bps: u64,
    health: String,
    alert_count: usize,
}

fn resolve_service_api_observability(
    report: &NodeBootstrapReport,
) -> ServiceApiObservabilitySnapshot {
    if let (
        Some(latency_p50_ms),
        Some(latency_p99_ms),
        Some(throughput_tps),
        Some(error_rate_bps),
        Some(availability_bps),
        Some(health),
        Some(alert_count),
    ) = (
        report.daemon_observability_latency_p50_ms,
        report.daemon_observability_latency_p99_ms,
        report.daemon_observability_throughput_tps,
        report.daemon_observability_error_rate_bps,
        report.daemon_observability_availability_bps,
        report.daemon_observability_health.as_deref(),
        report.daemon_observability_alert_count,
    ) {
        return ServiceApiObservabilitySnapshot {
            source: "daemon".to_owned(),
            latency_p50_ms,
            latency_p99_ms,
            throughput_tps,
            error_rate_bps,
            availability_bps,
            health: health.to_owned(),
            alert_count,
        };
    }

    if let (
        Some(latency_p50_ms),
        Some(latency_p99_ms),
        Some(throughput_tps),
        Some(error_rate_bps),
        Some(availability_bps),
        Some(health),
        Some(alert_count),
    ) = (
        report.kolme_live_observability_latency_p50_ms,
        report.kolme_live_observability_latency_p99_ms,
        report.kolme_live_observability_throughput_tps,
        report.kolme_live_observability_error_rate_bps,
        report.kolme_live_observability_availability_bps,
        report.kolme_live_observability_health.as_deref(),
        report.kolme_live_observability_alert_count,
    ) {
        return ServiceApiObservabilitySnapshot {
            source: "kolme-live".to_owned(),
            latency_p50_ms,
            latency_p99_ms,
            throughput_tps,
            error_rate_bps,
            availability_bps,
            health: health.to_owned(),
            alert_count,
        };
    }

    ServiceApiObservabilitySnapshot {
        source: "unknown".to_owned(),
        latency_p50_ms: 0,
        latency_p99_ms: 0,
        throughput_tps: 0,
        error_rate_bps: 0,
        availability_bps: 0,
        health: "unknown".to_owned(),
        alert_count: 0,
    }
}

fn header_value<'a>(headers: &'a BTreeMap<String, String>, name: &str) -> Option<&'a str> {
    headers.get(name).map(String::as_str)
}

fn authorize_service_api_request(
    snapshot: &ServiceApiSnapshot,
    request: &ParsedRequest,
    replay_guard: &mut BTreeSet<(String, u64)>,
) -> Result<(), RequestAuthFailure> {
    if !route_requires_auth(request.method.as_str(), request.path.as_str()) {
        return Ok(());
    }
    let sender_did =
        header_value(&request.headers, REQUEST_AUTH_SENDER_DID_HEADER).ok_or_else(|| {
            RequestAuthFailure::Unauthorized(ServiceApiReasonedError::new(
                REASON_CODE_AUTH_SENDER_DID_HEADER_MISSING,
                format!("missing required header: {REQUEST_AUTH_SENDER_DID_HEADER}"),
            ))
        })?;
    AgentDid::parse(sender_did).map_err(|error| {
        RequestAuthFailure::Unauthorized(ServiceApiReasonedError::new(
            REASON_CODE_AUTH_SENDER_DID_INVALID,
            format!("invalid sender did: {error}"),
        ))
    })?;
    let nonce_raw = header_value(&request.headers, REQUEST_AUTH_NONCE_HEADER).ok_or_else(|| {
        RequestAuthFailure::Unauthorized(ServiceApiReasonedError::new(
            REASON_CODE_AUTH_NONCE_HEADER_MISSING,
            format!("missing required header: {REQUEST_AUTH_NONCE_HEADER}"),
        ))
    })?;
    let nonce = nonce_raw.parse::<u64>().map_err(|_| {
        RequestAuthFailure::Unauthorized(ServiceApiReasonedError::new(
            REASON_CODE_AUTH_NONCE_INVALID,
            format!("invalid request nonce header: {REQUEST_AUTH_NONCE_HEADER}"),
        ))
    })?;
    if nonce == 0 {
        return Err(RequestAuthFailure::Unauthorized(
            ServiceApiReasonedError::new(
                REASON_CODE_AUTH_NONCE_NON_POSITIVE,
                format!("request nonce must be positive: {REQUEST_AUTH_NONCE_HEADER}"),
            ),
        ));
    }
    let signature =
        header_value(&request.headers, REQUEST_AUTH_SIGNATURE_HEADER).ok_or_else(|| {
            RequestAuthFailure::Unauthorized(ServiceApiReasonedError::new(
                REASON_CODE_AUTH_SIGNATURE_HEADER_MISSING,
                format!("missing required header: {REQUEST_AUTH_SIGNATURE_HEADER}"),
            ))
        })?;
    let state_hash = service_api_signature_state_hash(snapshot);
    if !signature_matches_supported_profile_for_fields(
        signature,
        sender_did,
        nonce,
        state_hash.as_str(),
        request.body.as_str(),
    ) {
        return Err(RequestAuthFailure::Unauthorized(
            ServiceApiReasonedError::new(
                REASON_CODE_AUTH_SIGNATURE_VERIFICATION_FAILED,
                "signature verification failed for request envelope",
            ),
        ));
    }
    if !replay_guard.insert((sender_did.to_owned(), nonce)) {
        return Err(RequestAuthFailure::Replay(ServiceApiReasonedError::new(
            REASON_CODE_AUTH_REPLAY_NONCE_DETECTED,
            "request nonce replay detected for sender",
        )));
    }
    Ok(())
}

async fn enforce_sender_anti_spam(
    state: &ServiceApiRuntimeState,
    request: &ParsedRequest,
) -> Result<(), ServiceApiReasonedError> {
    if !route_requires_auth(request.method.as_str(), request.path.as_str()) {
        return Ok(());
    }

    let sender_did =
        header_value(&request.headers, REQUEST_AUTH_SENDER_DID_HEADER).ok_or_else(|| {
            ServiceApiReasonedError::new(
                REASON_CODE_AUTH_SENDER_DID_HEADER_MISSING,
                format!("missing required header: {REQUEST_AUTH_SENDER_DID_HEADER}"),
            )
        })?;
    let nonce_raw = header_value(&request.headers, REQUEST_AUTH_NONCE_HEADER).ok_or_else(|| {
        ServiceApiReasonedError::new(
            REASON_CODE_AUTH_NONCE_HEADER_MISSING,
            format!("missing required header: {REQUEST_AUTH_NONCE_HEADER}"),
        )
    })?;
    let nonce = nonce_raw.parse::<u64>().map_err(|_| {
        ServiceApiReasonedError::new(
            REASON_CODE_AUTH_NONCE_INVALID,
            format!("invalid request nonce header: {REQUEST_AUTH_NONCE_HEADER}"),
        )
    })?;
    let now_unix = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|error| {
            ServiceApiReasonedError::new(
                REASON_CODE_INGRESS_ANTI_SPAM_ENGINE_INVALID,
                format!("anti-spam clock evaluation failed: {error}"),
            )
        })?
        .as_secs();
    let message_id = format!("{sender_did}:{nonce}:{}", request.path);
    let decision = {
        let mut anti_spam = state.sender_anti_spam.lock().await;
        anti_spam
            .evaluate(sender_did, message_id.as_str(), now_unix)
            .map_err(|error| {
                ServiceApiReasonedError::new(
                    REASON_CODE_INGRESS_ANTI_SPAM_ENGINE_INVALID,
                    format!("anti-spam decision evaluation failed: {error}"),
                )
            })?
    };

    match decision {
        AntiSpamDecision::Accepted => Ok(()),
        AntiSpamDecision::Rejected(rejection) => {
            Err(map_anti_spam_rejection_to_reasoned_error(rejection))
        }
    }
}

fn map_anti_spam_rejection_to_reasoned_error(
    rejection: AntiSpamRejection,
) -> ServiceApiReasonedError {
    match rejection {
        AntiSpamRejection::InsufficientDeposit { required, provided } => ServiceApiReasonedError::new(
            REASON_CODE_INGRESS_SENDER_INSUFFICIENT_DEPOSIT,
            format!(
                "sender deposit below anti-spam minimum: required={required}, provided={provided}"
            ),
        ),
        AntiSpamRejection::RateLimitExceeded {
            limit,
            observed,
            window_seconds,
        } => ServiceApiReasonedError::new(
            REASON_CODE_INGRESS_SENDER_RATE_LIMIT_EXCEEDED,
            format!(
                "sender anti-spam rate limit exceeded: observed={observed}, limit={limit}, window_seconds={window_seconds}"
            ),
        ),
        AntiSpamRejection::SenderSuspended { until_unix } => ServiceApiReasonedError::new(
            REASON_CODE_INGRESS_SENDER_SUSPENDED,
            format!("sender suspended by anti-spam policy until unix={until_unix}"),
        ),
        AntiSpamRejection::DuplicateMessageId(message_id) => ServiceApiReasonedError::new(
            REASON_CODE_INGRESS_SENDER_DUPLICATE_MESSAGE_ID,
            format!("sender anti-spam duplicate message id rejected: {message_id}"),
        ),
    }
}

fn route_exists_for_other_method(path: &str) -> bool {
    path == ROUTE_MESSAGES_SEND
        || path == ROUTE_CHANNELS_CREATE
        || path == ROUTE_TASKS_CREATE
        || path == ROUTE_EVENTS_WS
        || path == ROUTE_HEALTHZ
        || path == ROUTE_METRICS
        || message_path_id(path).is_some()
        || channel_messages_path_id(path).is_some()
        || task_path_id(path).is_some()
        || agent_path_id(path).is_some()
}

fn message_path_id(path: &str) -> Option<&str> {
    path.strip_prefix(ROUTE_MESSAGES_PREFIX).and_then(|id| {
        if id.is_empty() || id == "send" || id.contains('/') {
            return None;
        }
        Some(id)
    })
}

fn channel_messages_path_id(path: &str) -> Option<&str> {
    let channel_path = path.strip_prefix(ROUTE_CHANNELS_PREFIX)?;
    let channel_id = channel_path.strip_suffix(ROUTE_CHANNELS_MESSAGES_SUFFIX)?;
    if channel_id.is_empty() || channel_id.contains('/') {
        return None;
    }
    Some(channel_id)
}

fn task_path_id(path: &str) -> Option<&str> {
    path.strip_prefix(ROUTE_TASKS_PREFIX).and_then(|id| {
        if id.is_empty() || id == "create" || id.contains('/') {
            return None;
        }
        Some(id)
    })
}

fn agent_path_id(path: &str) -> Option<&str> {
    path.strip_prefix(ROUTE_AGENTS_PREFIX).and_then(|did| {
        if did.is_empty() || did.contains('/') {
            return None;
        }
        Some(did)
    })
}

fn build_parsed_request(
    method: &str,
    path: &str,
    headers: &HeaderMap,
    body: Bytes,
) -> Result<ParsedRequest, ServiceApiReasonedError> {
    let mut normalized_headers = BTreeMap::new();
    for (header_name, header_value) in headers {
        let value = header_value.to_str().map_err(|_| {
            ServiceApiReasonedError::new(
                REASON_CODE_REQUEST_HEADER_UTF8_INVALID,
                format!("request header value was not valid utf-8: {header_name}"),
            )
        })?;
        normalized_headers.insert(
            header_name.as_str().to_ascii_lowercase(),
            value.trim().to_owned(),
        );
    }
    let body = String::from_utf8(body.to_vec()).map_err(|_| {
        ServiceApiReasonedError::new(
            REASON_CODE_REQUEST_BODY_UTF8_INVALID,
            "request was not valid utf-8",
        )
    })?;
    Ok(ParsedRequest {
        method: method.to_owned(),
        path: path.to_owned(),
        body,
        headers: normalized_headers,
    })
}

fn validate_websocket_route_requirements(
    is_websocket_route: bool,
    headers: &BTreeMap<String, String>,
) -> Result<(), ServiceApiReasonedError> {
    if !is_websocket_route {
        return Ok(());
    }
    validate_websocket_upgrade_headers(headers)
}

fn validate_websocket_upgrade_headers(
    headers: &BTreeMap<String, String>,
) -> Result<(), ServiceApiReasonedError> {
    let upgrade = header_value(headers, "upgrade").ok_or_else(|| {
        ServiceApiReasonedError::new(
            REASON_CODE_WS_UPGRADE_HEADER_MISSING,
            "missing required websocket upgrade header",
        )
    })?;
    let connection = header_value(headers, "connection").ok_or_else(|| {
        ServiceApiReasonedError::new(
            REASON_CODE_WS_CONNECTION_HEADER_MISSING,
            "missing required websocket connection header",
        )
    })?;
    let websocket_key = header_value(headers, "sec-websocket-key").ok_or_else(|| {
        ServiceApiReasonedError::new(
            REASON_CODE_WS_KEY_HEADER_MISSING,
            "missing required websocket key header",
        )
    })?;
    let websocket_version = header_value(headers, "sec-websocket-version").ok_or_else(|| {
        ServiceApiReasonedError::new(
            REASON_CODE_WS_VERSION_HEADER_MISSING,
            "missing required websocket version header",
        )
    })?;

    if !upgrade.eq_ignore_ascii_case("websocket") {
        return Err(ServiceApiReasonedError::new(
            REASON_CODE_WS_UPGRADE_HEADER_INVALID,
            "invalid websocket upgrade header",
        ));
    }
    if !connection.to_ascii_lowercase().contains("upgrade") {
        return Err(ServiceApiReasonedError::new(
            REASON_CODE_WS_CONNECTION_HEADER_INVALID,
            "invalid websocket connection header",
        ));
    }
    if websocket_key.trim().is_empty() {
        return Err(ServiceApiReasonedError::new(
            REASON_CODE_WS_KEY_HEADER_EMPTY,
            "websocket key header must not be empty",
        ));
    }
    if websocket_version.trim() != "13" {
        return Err(ServiceApiReasonedError::new(
            REASON_CODE_WS_VERSION_HEADER_INVALID,
            "invalid websocket version header",
        ));
    }
    Ok(())
}

fn websocket_upgrade_response(upgrade: WebSocketUpgrade, snapshot: ServiceApiSnapshot) -> Response {
    let mut response = upgrade
        .on_upgrade(move |socket| stream_websocket_event(socket, snapshot))
        .into_response();
    response
        .headers_mut()
        .insert("X-KAMN-WebSocket-Contract", HeaderValue::from_static("v1"));
    response
}

async fn stream_websocket_event(mut socket: WebSocket, snapshot: ServiceApiSnapshot) {
    let payload = ServiceApiWebsocketStateTransitionBody {
        event: "state-transition".to_owned(),
        runtime_mode: snapshot.runtime_mode,
        role: snapshot.role,
        sequence: 1,
    };
    let event_payload = serialize_service_api_json(&payload);
    let _ = socket.send(Message::Text(event_payload.into())).await;
}

fn contract_response(response: ServiceApiEndpointResponse) -> Response {
    let status =
        StatusCode::from_u16(response.status_code).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);
    (
        status,
        [("Content-Type", response.content_type)],
        response.body,
    )
        .into_response()
}

fn json_error_endpoint_response(
    status_code: StatusCode,
    error: &str,
    reason_code: &str,
    message: &str,
) -> ServiceApiEndpointResponse {
    ServiceApiEndpointResponse {
        status_code: status_code.as_u16(),
        content_type: "application/json",
        body: serialize_service_api_json(&ServiceApiErrorBody {
            error: error.to_owned(),
            reason_code: reason_code.to_owned(),
            message: message.to_owned(),
        }),
    }
}

fn json_error_response(
    status_code: StatusCode,
    error: &str,
    reason_code: &str,
    message: &str,
) -> Response {
    let payload = ServiceApiErrorBody {
        error: error.to_owned(),
        reason_code: reason_code.to_owned(),
        message: message.to_owned(),
    };
    (
        status_code,
        [("Content-Type", "application/json")],
        serialize_service_api_json(&payload),
    )
        .into_response()
}

#[cfg(test)]
pub(crate) fn parse_service_api_payload<T: DeserializeOwned>(payload: &str) -> Result<T, String> {
    serde_json::from_str(payload).map_err(|error| {
        format!(
            "{}:{}",
            service_api_payload_decode_reason_code(&error),
            error
        )
    })
}

#[cfg(test)]
pub(crate) fn service_api_payload_decode_reason_code(error: &serde_json::Error) -> &'static str {
    use serde_json::error::Category;
    match error.classify() {
        Category::Io => "service_api_payload_io_error",
        Category::Syntax | Category::Eof => "service_api_payload_json_syntax_invalid",
        Category::Data => "service_api_payload_structure_invalid",
    }
}

fn serialize_service_api_json<T: Serialize>(payload: &T) -> String {
    serde_json::to_string(payload).unwrap_or_else(|error| {
        format!(
            "{{\"error\":\"internal\",\"reason_code\":\"service_api_payload_serialization_failed\",\"message\":\"service api payload serialization failed: {}\"}}",
            escape_json_string(error.to_string().as_str())
        )
    })
}

fn deterministic_body_tag(payload: &[u8]) -> u64 {
    let mut acc: u64 = 0xcbf29ce484222325;
    for byte in payload {
        acc = acc.wrapping_mul(0x00000100000001B3);
        acc ^= u64::from(*byte);
    }
    acc
}

fn escape_json_string(input: &str) -> String {
    let mut escaped = String::with_capacity(input.len());
    for ch in input.chars() {
        match ch {
            '"' => escaped.push_str("\\\""),
            '\\' => escaped.push_str("\\\\"),
            '\n' => escaped.push_str("\\n"),
            '\r' => escaped.push_str("\\r"),
            '\t' => escaped.push_str("\\t"),
            _ => escaped.push(ch),
        }
    }
    escaped
}

fn escape_metrics_label(input: &str) -> String {
    input.replace('\\', "\\\\").replace('"', "\\\"")
}

#[cfg(test)]
mod tests {
    use super::{
        map_anti_spam_rejection_to_reasoned_error, AntiSpamRejection,
        REASON_CODE_INGRESS_SENDER_DUPLICATE_MESSAGE_ID,
        REASON_CODE_INGRESS_SENDER_INSUFFICIENT_DEPOSIT,
        REASON_CODE_INGRESS_SENDER_RATE_LIMIT_EXCEEDED, REASON_CODE_INGRESS_SENDER_SUSPENDED,
    };

    #[test]
    fn anti_spam_rate_limit_rejection_maps_to_sender_rate_limit_reason_code() {
        let error =
            map_anti_spam_rejection_to_reasoned_error(AntiSpamRejection::RateLimitExceeded {
                limit: 3,
                observed: 3,
                window_seconds: 5,
            });
        assert_eq!(
            error.reason_code,
            REASON_CODE_INGRESS_SENDER_RATE_LIMIT_EXCEEDED
        );
        assert!(error.message.contains("observed=3"));
    }

    #[test]
    fn anti_spam_sender_suspension_maps_to_sender_suspended_reason_code() {
        let error = map_anti_spam_rejection_to_reasoned_error(AntiSpamRejection::SenderSuspended {
            until_unix: 123_456,
        });
        assert_eq!(error.reason_code, REASON_CODE_INGRESS_SENDER_SUSPENDED);
        assert!(error.message.contains("123456"));
    }

    #[test]
    fn anti_spam_insufficient_deposit_maps_to_sender_deposit_reason_code() {
        let error =
            map_anti_spam_rejection_to_reasoned_error(AntiSpamRejection::InsufficientDeposit {
                required: 9,
                provided: 4,
            });
        assert_eq!(
            error.reason_code,
            REASON_CODE_INGRESS_SENDER_INSUFFICIENT_DEPOSIT
        );
        assert!(error.message.contains("required=9"));
        assert!(error.message.contains("provided=4"));
    }

    #[test]
    fn anti_spam_duplicate_message_maps_to_sender_duplicate_reason_code() {
        let error = map_anti_spam_rejection_to_reasoned_error(
            AntiSpamRejection::DuplicateMessageId("message-1".to_owned()),
        );
        assert_eq!(
            error.reason_code,
            REASON_CODE_INGRESS_SENDER_DUPLICATE_MESSAGE_ID
        );
        assert!(error.message.contains("message-1"));
    }
}
