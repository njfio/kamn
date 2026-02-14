use crate::{
    logging::{log_info, log_warn},
    NodeBootstrapReport,
};
use kamn_core::{signature_matches_supported_profile_for_fields, AgentDid};
use std::collections::{BTreeMap, BTreeSet};
use std::io::{ErrorKind, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::thread;
use std::time::{Duration, Instant};

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

    let listener = TcpListener::bind(config.bind_addr.as_str())
        .map_err(|error| format!("service api bind failed: {error}"))?;
    listener
        .set_nonblocking(true)
        .map_err(|error| format!("service api nonblocking mode failed: {error}"))?;

    let deadline = Instant::now() + Duration::from_millis(config.idle_timeout_ms);
    let mut served_requests = 0_u64;
    let mut replay_guard: BTreeSet<(String, u64)> = BTreeSet::new();
    'accept_loop: while served_requests < config.max_requests {
        if Instant::now() >= deadline {
            return Err(format!(
                "service api timed out after {} ms waiting for requests",
                config.idle_timeout_ms
            ));
        }
        match listener.accept() {
            Ok((mut stream, _)) => loop {
                if served_requests >= config.max_requests {
                    break 'accept_loop;
                }
                let request = match read_http_request(&mut stream) {
                    Ok(request) => request,
                    Err(error) => {
                        let correlation_id = format!(
                            "service-api:parse-error:{:016x}",
                            deterministic_body_tag(error.as_bytes())
                        );
                        emit_service_api_request_outcome(
                            correlation_id.as_str(),
                            "unknown",
                            "unknown",
                            400,
                            "bad-request",
                        )?;
                        let response = ServiceApiEndpointResponse {
                            status_code: 400,
                            content_type: "application/json",
                            body: format!(
                                "{{\"error\":\"bad-request\",\"reason\":\"{}\"}}",
                                escape_json_string(error.as_str())
                            ),
                        };
                        write_http_response(&mut stream, &response, false)?;
                        served_requests = served_requests.saturating_add(1);
                        break;
                    }
                };
                let keep_alive = request_prefers_keep_alive(&request);
                let correlation_id = service_api_request_correlation_id(&request);
                log_service_api_event_info(
                    "service.api.request.received",
                    &[
                        ("correlation_id", correlation_id.as_str()),
                        ("method", request.method.as_str()),
                        ("path", request.path.as_str()),
                    ],
                )?;
                let (response, outcome) =
                    match authorize_service_api_request(snapshot, &request, &mut replay_guard) {
                        Ok(()) => {
                            if request.method == "GET" && request.path == ROUTE_EVENTS_WS {
                                match write_websocket_upgrade_event_response(
                                    &mut stream,
                                    snapshot,
                                    &request.headers,
                                ) {
                                    Ok(()) => {
                                        emit_service_api_request_outcome(
                                            correlation_id.as_str(),
                                            request.method.as_str(),
                                            request.path.as_str(),
                                            101,
                                            "websocket-upgrade",
                                        )?;
                                        served_requests = served_requests.saturating_add(1);
                                        break;
                                    }
                                    Err(error) => (
                                        ServiceApiEndpointResponse {
                                            status_code: 400,
                                            content_type: "application/json",
                                            body: format!(
                                                "{{\"error\":\"bad-request\",\"reason\":\"{}\"}}",
                                                escape_json_string(error.as_str())
                                            ),
                                        },
                                        "websocket-bad-request",
                                    ),
                                }
                            } else {
                                (
                                    render_service_api_endpoint_response(
                                        snapshot,
                                        request.method.as_str(),
                                        request.path.as_str(),
                                        request.body.as_str(),
                                    ),
                                    "handled",
                                )
                            }
                        }
                        Err(RequestAuthFailure::Unauthorized(reason)) => (
                            ServiceApiEndpointResponse {
                                status_code: 401,
                                content_type: "application/json",
                                body: format!(
                                    "{{\"error\":\"unauthorized\",\"reason\":\"{}\"}}",
                                    escape_json_string(reason.as_str())
                                ),
                            },
                            "unauthorized",
                        ),
                        Err(RequestAuthFailure::Replay(reason)) => (
                            ServiceApiEndpointResponse {
                                status_code: 409,
                                content_type: "application/json",
                                body: format!(
                                    "{{\"error\":\"replay\",\"reason\":\"{}\"}}",
                                    escape_json_string(reason.as_str())
                                ),
                            },
                            "replay",
                        ),
                    };
                emit_service_api_request_outcome(
                    correlation_id.as_str(),
                    request.method.as_str(),
                    request.path.as_str(),
                    response.status_code,
                    outcome,
                )?;
                write_http_response(&mut stream, &response, keep_alive)?;
                served_requests = served_requests.saturating_add(1);
                if !keep_alive {
                    break;
                }
            },
            Err(error) if error.kind() == ErrorKind::WouldBlock => {
                thread::sleep(Duration::from_millis(5));
            }
            Err(error) => return Err(format!("service api accept failed: {error}")),
        }
    }
    Ok(())
}

fn route_requires_auth(method: &str, path: &str) -> bool {
    !(method == "GET" && (path == ROUTE_HEALTHZ || path == ROUTE_METRICS))
}

fn request_prefers_keep_alive(request: &ParsedRequest) -> bool {
    header_value(&request.headers, "connection")
        .map(|value| value.to_ascii_lowercase().contains("keep-alive"))
        .unwrap_or(false)
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

fn read_http_request(stream: &mut TcpStream) -> Result<ParsedRequest, String> {
    let mut request = Vec::new();
    let mut chunk = [0_u8; 1024];
    stream
        .set_read_timeout(Some(Duration::from_secs(1)))
        .map_err(|error| format!("service api read-timeout failed: {error}"))?;

    let mut expected_total_bytes: Option<usize> = None;
    let mut header_end: Option<usize> = None;
    loop {
        match stream.read(&mut chunk) {
            Ok(0) => break,
            Ok(read_count) => {
                request.extend_from_slice(&chunk[..read_count]);
                if header_end.is_none() {
                    header_end = request
                        .windows(4)
                        .position(|window| window == b"\r\n\r\n")
                        .map(|index| index + 4);
                    if let Some(header_end_index) = header_end {
                        let header = String::from_utf8(request[..header_end_index].to_vec())
                            .map_err(|_| "request header was not valid utf-8".to_owned())?;
                        let content_length = parse_content_length(header.as_str())?;
                        expected_total_bytes = Some(header_end_index + content_length);
                    }
                }
                if let Some(total) = expected_total_bytes {
                    if request.len() >= total {
                        break;
                    }
                }
                if request.len() > 64 * 1024 {
                    return Err("request header too large".to_owned());
                }
            }
            Err(error) if matches!(error.kind(), ErrorKind::WouldBlock | ErrorKind::TimedOut) => {
                break;
            }
            Err(error) => return Err(format!("request read failed: {error}")),
        }
    }

    if request.is_empty() {
        return Err("connection closed before request bytes arrived".to_owned());
    }

    let request_text =
        String::from_utf8(request).map_err(|_| "request was not valid utf-8".to_owned())?;
    let Some((request_head, request_body)) = request_text.split_once("\r\n\r\n") else {
        return Err("request header terminator missing".to_owned());
    };
    let request_line = request_head
        .lines()
        .next()
        .ok_or_else(|| "request line missing".to_owned())?;
    let mut headers = BTreeMap::new();
    for line in request_head.lines().skip(1) {
        if line.trim().is_empty() {
            continue;
        }
        let (name, value) = line
            .split_once(':')
            .ok_or_else(|| "request header line missing ':' separator".to_owned())?;
        let name = name.trim();
        if name.is_empty() {
            return Err("request header name missing".to_owned());
        }
        headers.insert(name.to_ascii_lowercase(), value.trim().to_owned());
    }
    let mut line_parts = request_line.split_whitespace();
    let method = line_parts
        .next()
        .ok_or_else(|| "request method missing".to_owned())?;
    let path = line_parts
        .next()
        .ok_or_else(|| "request path missing".to_owned())?;
    Ok(ParsedRequest {
        method: method.to_owned(),
        path: path.to_owned(),
        body: request_body.to_owned(),
        headers,
    })
}

fn parse_content_length(header: &str) -> Result<usize, String> {
    let value = header
        .lines()
        .find_map(|line| {
            let (name, raw_value) = line.split_once(':')?;
            if name.eq_ignore_ascii_case("Content-Length") {
                return Some(raw_value.trim());
            }
            None
        })
        .unwrap_or("0");
    value
        .parse::<usize>()
        .map_err(|_| "invalid content-length header".to_owned())
}

fn write_websocket_upgrade_event_response(
    stream: &mut TcpStream,
    snapshot: &ServiceApiSnapshot,
    headers: &BTreeMap<String, String>,
) -> Result<(), String> {
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

    let accept_marker = format!(
        "kamn-{:016x}",
        deterministic_body_tag(websocket_key.as_bytes())
    );
    let handshake = format!(
        "HTTP/1.1 101 Switching Protocols\r\nUpgrade: websocket\r\nConnection: Upgrade\r\nSec-WebSocket-Accept: {accept_marker}\r\nX-KAMN-WebSocket-Contract: v1\r\n\r\n"
    );
    stream
        .write_all(handshake.as_bytes())
        .map_err(|error| format!("service api websocket handshake write failed: {error}"))?;

    let event_payload = format!(
        "{{\"event\":\"state-transition\",\"runtime_mode\":\"{}\",\"role\":\"{}\",\"sequence\":1}}",
        escape_json_string(snapshot.runtime_mode.as_str()),
        escape_json_string(snapshot.role.as_str()),
    );
    write_websocket_text_frame(stream, event_payload.as_bytes())
}

fn write_websocket_text_frame(stream: &mut TcpStream, payload: &[u8]) -> Result<(), String> {
    if payload.len() > 125 {
        return Err("websocket payload exceeds small-frame contract".to_owned());
    }
    let mut frame = Vec::with_capacity(2 + payload.len());
    frame.push(0x81);
    frame.push(payload.len() as u8);
    frame.extend_from_slice(payload);
    stream
        .write_all(frame.as_slice())
        .map_err(|error| format!("service api websocket frame write failed: {error}"))
}

fn write_http_response(
    stream: &mut TcpStream,
    response: &ServiceApiEndpointResponse,
    keep_alive: bool,
) -> Result<(), String> {
    let status_text = match response.status_code {
        200 => "200 OK",
        201 => "201 Created",
        202 => "202 Accepted",
        400 => "400 Bad Request",
        401 => "401 Unauthorized",
        404 => "404 Not Found",
        405 => "405 Method Not Allowed",
        409 => "409 Conflict",
        _ => "500 Internal Server Error",
    };
    let connection_header = if keep_alive { "keep-alive" } else { "close" };
    let payload = format!(
        "HTTP/1.1 {status_text}\r\nContent-Type: {}\r\nContent-Length: {}\r\nConnection: {connection_header}\r\n\r\n{}",
        response.content_type,
        response.body.len(),
        response.body
    );
    stream
        .write_all(payload.as_bytes())
        .map_err(|error| format!("service api write failed: {error}"))
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
