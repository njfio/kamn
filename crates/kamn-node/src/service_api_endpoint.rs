use crate::{
    logging::{log_info, log_warn},
    NodeBootstrapReport,
};
use axum::{
    body::{to_bytes, Bytes},
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        FromRequest, Request, State,
    },
    http::{HeaderMap, HeaderValue, StatusCode},
    response::{IntoResponse, Response},
    routing::any,
    Router,
};
use kamn_core::{signature_matches_supported_profile_for_fields, AgentDid};
use std::collections::{BTreeMap, BTreeSet};
use std::sync::{
    atomic::{AtomicBool, AtomicU64, Ordering},
    Arc,
};
use std::time::{Duration, Instant};
use tokio::runtime::Builder;
use tokio::sync::{Mutex, Notify};

pub(crate) const DEFAULT_SERVICE_API_MAX_REQUESTS: u64 = 1;
pub(crate) const DEFAULT_SERVICE_API_IDLE_TIMEOUT_MS: u64 = 5_000;

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

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ServiceApiEndpointConfig {
    pub(crate) bind_addr: String,
    pub(crate) max_requests: u64,
    pub(crate) idle_timeout_ms: u64,
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

#[derive(Debug, Clone, PartialEq, Eq)]
struct ParsedRequest {
    method: String,
    path: String,
    body: String,
    headers: BTreeMap<String, String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum RequestAuthFailure {
    Unauthorized(String),
    Replay(String),
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

#[derive(Debug)]
struct ServiceApiRuntimeState {
    snapshot: ServiceApiSnapshot,
    replay_guard: Arc<Mutex<BTreeSet<(String, u64)>>>,
    request_budget: Arc<ServiceApiRequestBudget>,
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
        return ServiceApiEndpointResponse {
            status_code: 200,
            content_type: "application/json",
            body: format!(
                "{{\"status\":\"ok\",\"runtime_mode\":\"{}\",\"role\":\"{}\",\"observability_source\":\"{}\",\"observability_health\":\"{}\"}}",
                escape_json_string(snapshot.runtime_mode.as_str()),
                escape_json_string(snapshot.role.as_str()),
                escape_json_string(snapshot.observability_source.as_str()),
                escape_json_string(snapshot.observability_health.as_str()),
            ),
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
        return ServiceApiEndpointResponse {
            status_code: 400,
            content_type: "application/json",
            body: "{\"error\":\"bad-request\",\"reason\":\"websocket upgrade required\"}"
                .to_owned(),
        };
    }
    if method == "POST" && path == ROUTE_MESSAGES_SEND {
        let message_id = format!("msg-local-{}", deterministic_body_tag(body.as_bytes()));
        return ServiceApiEndpointResponse {
            status_code: 202,
            content_type: "application/json",
            body: format!(
                "{{\"message_id\":\"{}\",\"status\":\"created\",\"runtime_mode\":\"{}\"}}",
                escape_json_string(message_id.as_str()),
                escape_json_string(snapshot.runtime_mode.as_str())
            ),
        };
    }
    if method == "POST" && path == ROUTE_CHANNELS_CREATE {
        let channel_id = format!("channel-local-{}", deterministic_body_tag(body.as_bytes()));
        return ServiceApiEndpointResponse {
            status_code: 201,
            content_type: "application/json",
            body: format!(
                "{{\"channel_id\":\"{}\",\"status\":\"created\"}}",
                escape_json_string(channel_id.as_str()),
            ),
        };
    }
    if method == "POST" && path == ROUTE_TASKS_CREATE {
        let task_id = format!("task-local-{}", deterministic_body_tag(body.as_bytes()));
        return ServiceApiEndpointResponse {
            status_code: 201,
            content_type: "application/json",
            body: format!(
                "{{\"task_id\":\"{}\",\"state\":\"submitted\"}}",
                escape_json_string(task_id.as_str()),
            ),
        };
    }
    if method == "GET" {
        if let Some(message_id) = message_path_id(path) {
            return ServiceApiEndpointResponse {
                status_code: 200,
                content_type: "application/json",
                body: format!(
                    "{{\"message_id\":\"{}\",\"status\":\"created\"}}",
                    escape_json_string(message_id)
                ),
            };
        }
        if let Some(channel_id) = channel_messages_path_id(path) {
            return ServiceApiEndpointResponse {
                status_code: 200,
                content_type: "application/json",
                body: format!(
                    "{{\"channel_id\":\"{}\",\"messages\":[]}}",
                    escape_json_string(channel_id)
                ),
            };
        }
        if let Some(task_id) = task_path_id(path) {
            return ServiceApiEndpointResponse {
                status_code: 200,
                content_type: "application/json",
                body: format!(
                    "{{\"task_id\":\"{}\",\"state\":\"submitted\"}}",
                    escape_json_string(task_id)
                ),
            };
        }
        if let Some(agent_did) = agent_path_id(path) {
            return ServiceApiEndpointResponse {
                status_code: 200,
                content_type: "application/json",
                body: format!(
                    "{{\"did\":\"{}\",\"reputation_score\":500}}",
                    escape_json_string(agent_did)
                ),
            };
        }
    }

    if route_exists_for_other_method(path) {
        return ServiceApiEndpointResponse {
            status_code: 405,
            content_type: "text/plain; charset=utf-8",
            body: "method not allowed\n".to_owned(),
        };
    }
    ServiceApiEndpointResponse {
        status_code: 404,
        content_type: "text/plain; charset=utf-8",
        body: "not found\n".to_owned(),
    }
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

async fn serve_service_api_endpoint_async(
    config: ServiceApiEndpointConfig,
    snapshot: ServiceApiSnapshot,
) -> Result<(), String> {
    let listener = tokio::net::TcpListener::bind(config.bind_addr.as_str())
        .await
        .map_err(|error| format!("service api bind failed: {error}"))?;

    let runtime_state = Arc::new(ServiceApiRuntimeState {
        snapshot,
        replay_guard: Arc::new(Mutex::new(BTreeSet::new())),
        request_budget: Arc::new(ServiceApiRequestBudget::new(config.max_requests)),
    });
    let timeout_reached = Arc::new(AtomicBool::new(false));
    let request_budget = runtime_state.request_budget.clone();
    let timeout_flag = timeout_reached.clone();
    let deadline = Instant::now() + Duration::from_millis(config.idle_timeout_ms);

    let app = build_service_api_router(runtime_state);

    axum::serve(listener, app)
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

    if timeout_reached.load(Ordering::SeqCst) {
        return Err(format!(
            "service api timed out after {} ms waiting for requests",
            config.idle_timeout_ms
        ));
    }

    Ok(())
}

fn build_service_api_router(state: Arc<ServiceApiRuntimeState>) -> Router {
    Router::new()
        .route("/", any(handle_service_api_request))
        .route("/{*path}", any(handle_service_api_request))
        .with_state(state)
}

async fn handle_service_api_request(
    State(state): State<Arc<ServiceApiRuntimeState>>,
    request: Request,
) -> Response {
    let method_label = request.method().to_string();
    let path = request.uri().path().to_owned();
    let headers = request.headers().clone();
    if method_label == "GET" && path == ROUTE_EVENTS_WS {
        let parsed_request = match build_parsed_request(
            method_label.as_str(),
            path.as_str(),
            &headers,
            Bytes::new(),
        ) {
            Ok(request) => request,
            Err(reason) => {
                let correlation_id = format!(
                    "service-api:parse-error:{:016x}",
                    deterministic_body_tag(reason.as_bytes())
                );
                let status_code = StatusCode::BAD_REQUEST;
                let outcome = "bad-request";
                let response = json_error_response(status_code, "bad-request", reason.as_str());
                let _ = emit_service_api_request_outcome(
                    correlation_id.as_str(),
                    "unknown",
                    "unknown",
                    status_code.as_u16(),
                    outcome,
                );
                state.request_budget.record_request();
                return response;
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
            state.request_budget.record_request();
            return json_error_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                "internal",
                reason.as_str(),
            );
        }
        {
            let mut replay_guard = state.replay_guard.lock().await;
            if let Err(error) =
                authorize_service_api_request(&state.snapshot, &parsed_request, &mut replay_guard)
            {
                let (status_code, error_label, reason, outcome) = match error {
                    RequestAuthFailure::Unauthorized(reason) => (
                        StatusCode::UNAUTHORIZED,
                        "unauthorized",
                        reason,
                        "unauthorized",
                    ),
                    RequestAuthFailure::Replay(reason) => {
                        (StatusCode::CONFLICT, "replay", reason, "replay")
                    }
                };
                let response = json_error_response(status_code, error_label, reason.as_str());
                let _ = emit_service_api_request_outcome(
                    correlation_id.as_str(),
                    parsed_request.method.as_str(),
                    parsed_request.path.as_str(),
                    status_code.as_u16(),
                    outcome,
                );
                state.request_budget.record_request();
                return response;
            }
        }
        let (response, outcome_label) =
            match validate_websocket_upgrade_headers(&parsed_request.headers) {
                Ok(()) => match WebSocketUpgrade::from_request(request, &()).await {
                    Ok(upgrade) => (
                        websocket_upgrade_response(upgrade, state.snapshot.clone()),
                        "websocket-upgrade",
                    ),
                    Err(_) => (
                        json_error_response(
                            StatusCode::BAD_REQUEST,
                            "bad-request",
                            "websocket upgrade required",
                        ),
                        "websocket-bad-request",
                    ),
                },
                Err(reason) => (
                    json_error_response(StatusCode::BAD_REQUEST, "bad-request", reason.as_str()),
                    "websocket-bad-request",
                ),
            };
        let _ = emit_service_api_request_outcome(
            correlation_id.as_str(),
            parsed_request.method.as_str(),
            parsed_request.path.as_str(),
            response.status().as_u16(),
            outcome_label,
        );
        state.request_budget.record_request();
        return response;
    }

    let body_limit = 64 * 1024;
    let body = match to_bytes(request.into_body(), body_limit).await {
        Ok(body) => body,
        Err(error) => {
            let reason = format!("request read failed: {error}");
            let correlation_id = format!(
                "service-api:parse-error:{:016x}",
                deterministic_body_tag(reason.as_bytes())
            );
            let status_code = StatusCode::BAD_REQUEST;
            let outcome = "bad-request";
            let response = json_error_response(status_code, "bad-request", reason.as_str());
            let _ = emit_service_api_request_outcome(
                correlation_id.as_str(),
                "unknown",
                "unknown",
                status_code.as_u16(),
                outcome,
            );
            state.request_budget.record_request();
            return response;
        }
    };
    let parsed_request =
        match build_parsed_request(method_label.as_str(), path.as_str(), &headers, body) {
            Ok(request) => request,
            Err(reason) => {
                let correlation_id = format!(
                    "service-api:parse-error:{:016x}",
                    deterministic_body_tag(reason.as_bytes())
                );
                let status_code = StatusCode::BAD_REQUEST;
                let outcome = "bad-request";
                let response = json_error_response(status_code, "bad-request", reason.as_str());
                let _ = emit_service_api_request_outcome(
                    correlation_id.as_str(),
                    "unknown",
                    "unknown",
                    status_code.as_u16(),
                    outcome,
                );
                state.request_budget.record_request();
                return response;
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
        state.request_budget.record_request();
        return json_error_response(
            StatusCode::INTERNAL_SERVER_ERROR,
            "internal",
            reason.as_str(),
        );
    }

    {
        let mut replay_guard = state.replay_guard.lock().await;
        if let Err(error) =
            authorize_service_api_request(&state.snapshot, &parsed_request, &mut replay_guard)
        {
            let (status_code, error_label, reason, outcome) = match error {
                RequestAuthFailure::Unauthorized(reason) => (
                    StatusCode::UNAUTHORIZED,
                    "unauthorized",
                    reason,
                    "unauthorized",
                ),
                RequestAuthFailure::Replay(reason) => {
                    (StatusCode::CONFLICT, "replay", reason, "replay")
                }
            };
            let response = json_error_response(status_code, error_label, reason.as_str());
            let _ = emit_service_api_request_outcome(
                correlation_id.as_str(),
                parsed_request.method.as_str(),
                parsed_request.path.as_str(),
                status_code.as_u16(),
                outcome,
            );
            state.request_budget.record_request();
            return response;
        }
    }

    let rendered = render_service_api_endpoint_response(
        &state.snapshot,
        parsed_request.method.as_str(),
        parsed_request.path.as_str(),
        parsed_request.body.as_str(),
    );
    let response = contract_response(rendered);
    let _ = emit_service_api_request_outcome(
        correlation_id.as_str(),
        parsed_request.method.as_str(),
        parsed_request.path.as_str(),
        response.status().as_u16(),
        "handled",
    );
    state.request_budget.record_request();
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
            RequestAuthFailure::Unauthorized(format!(
                "missing required header: {REQUEST_AUTH_SENDER_DID_HEADER}"
            ))
        })?;
    AgentDid::parse(sender_did).map_err(|error| {
        RequestAuthFailure::Unauthorized(format!("invalid sender did: {error}"))
    })?;
    let nonce_raw = header_value(&request.headers, REQUEST_AUTH_NONCE_HEADER).ok_or_else(|| {
        RequestAuthFailure::Unauthorized(format!(
            "missing required header: {REQUEST_AUTH_NONCE_HEADER}"
        ))
    })?;
    let nonce = nonce_raw.parse::<u64>().map_err(|_| {
        RequestAuthFailure::Unauthorized(format!(
            "invalid request nonce header: {REQUEST_AUTH_NONCE_HEADER}"
        ))
    })?;
    if nonce == 0 {
        return Err(RequestAuthFailure::Unauthorized(format!(
            "request nonce must be positive: {REQUEST_AUTH_NONCE_HEADER}"
        )));
    }
    let signature =
        header_value(&request.headers, REQUEST_AUTH_SIGNATURE_HEADER).ok_or_else(|| {
            RequestAuthFailure::Unauthorized(format!(
                "missing required header: {REQUEST_AUTH_SIGNATURE_HEADER}"
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
            "signature verification failed for request envelope".to_owned(),
        ));
    }
    if !replay_guard.insert((sender_did.to_owned(), nonce)) {
        return Err(RequestAuthFailure::Replay(
            "request nonce replay detected for sender".to_owned(),
        ));
    }
    Ok(())
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
) -> Result<ParsedRequest, String> {
    let mut normalized_headers = BTreeMap::new();
    for (header_name, header_value) in headers {
        let value = header_value
            .to_str()
            .map_err(|_| format!("request header value was not valid utf-8: {header_name}"))?;
        normalized_headers.insert(
            header_name.as_str().to_ascii_lowercase(),
            value.trim().to_owned(),
        );
    }
    let body =
        String::from_utf8(body.to_vec()).map_err(|_| "request was not valid utf-8".to_owned())?;
    Ok(ParsedRequest {
        method: method.to_owned(),
        path: path.to_owned(),
        body,
        headers: normalized_headers,
    })
}

fn validate_websocket_upgrade_headers(headers: &BTreeMap<String, String>) -> Result<(), String> {
    let upgrade = header_value(headers, "upgrade")
        .ok_or_else(|| "missing required websocket upgrade header".to_owned())?;
    let connection = header_value(headers, "connection")
        .ok_or_else(|| "missing required websocket connection header".to_owned())?;
    let websocket_key = header_value(headers, "sec-websocket-key")
        .ok_or_else(|| "missing required websocket key header".to_owned())?;
    let websocket_version = header_value(headers, "sec-websocket-version")
        .ok_or_else(|| "missing required websocket version header".to_owned())?;

    if !upgrade.eq_ignore_ascii_case("websocket") {
        return Err("invalid websocket upgrade header".to_owned());
    }
    if !connection.to_ascii_lowercase().contains("upgrade") {
        return Err("invalid websocket connection header".to_owned());
    }
    if websocket_key.trim().is_empty() {
        return Err("websocket key header must not be empty".to_owned());
    }
    if websocket_version.trim() != "13" {
        return Err("invalid websocket version header".to_owned());
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
    let event_payload = format!(
        "{{\"event\":\"state-transition\",\"runtime_mode\":\"{}\",\"role\":\"{}\",\"sequence\":1}}",
        escape_json_string(snapshot.runtime_mode.as_str()),
        escape_json_string(snapshot.role.as_str()),
    );
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

fn json_error_response(status_code: StatusCode, error: &str, reason: &str) -> Response {
    (
        status_code,
        [("Content-Type", "application/json")],
        format!(
            "{{\"error\":\"{}\",\"reason\":\"{}\"}}",
            escape_json_string(error),
            escape_json_string(reason),
        ),
    )
        .into_response()
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
