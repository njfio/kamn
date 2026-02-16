use crate::NodeBootstrapReport;
use axum::{
    body::Body,
    extract::State,
    http::{
        header::{CONTENT_LENGTH, CONTENT_TYPE, TRANSFER_ENCODING},
        HeaderMap, HeaderValue, Method, StatusCode, Uri,
    },
    response::Response,
    routing::any,
    Router,
};
#[cfg(test)]
use std::cell::RefCell;
use std::env;
use std::fs;
use std::io::BufReader;
use std::net::SocketAddr;
use std::sync::{
    atomic::{AtomicBool, AtomicU64, Ordering},
    Arc,
};
use std::time::Duration;
use tokio::runtime::Builder;
use tokio::sync::Notify;
use tokio::time::Instant;

pub(crate) const DEFAULT_OBSERVABILITY_ENDPOINT_METRICS_PATH: &str = "/metrics";
pub(crate) const DEFAULT_OBSERVABILITY_ENDPOINT_HEALTH_PATH: &str = "/healthz";
pub(crate) const DEFAULT_OBSERVABILITY_ENDPOINT_READINESS_PATH: &str = "/readyz";
pub(crate) const DEFAULT_OBSERVABILITY_ENDPOINT_STREAM_PATH: &str = "/metrics.stream";
pub(crate) const DEFAULT_OBSERVABILITY_ENDPOINT_MAX_REQUESTS: u64 = 1;
pub(crate) const DEFAULT_OBSERVABILITY_ENDPOINT_IDLE_TIMEOUT_MS: u64 = 5_000;
const OBSERVABILITY_ENDPOINT_TLS_MODE_ENV: &str = "KAMN_OBSERVABILITY_ENDPOINT_TLS_MODE";
const OBSERVABILITY_ENDPOINT_TLS_CERT_FILE_ENV: &str = "KAMN_OBSERVABILITY_ENDPOINT_TLS_CERT_FILE";
const OBSERVABILITY_ENDPOINT_TLS_KEY_FILE_ENV: &str = "KAMN_OBSERVABILITY_ENDPOINT_TLS_KEY_FILE";
const OBSERVABILITY_ENDPOINT_TLS_MODE_DISABLED: &str = "disabled";
const OBSERVABILITY_ENDPOINT_TLS_MODE_REQUIRE: &str = "require";
const OBSERVABILITY_HEALTH_SCHEMA_VERSION: &str = "kamn.runtime.observability.health.v1";
const OBSERVABILITY_READINESS_SCHEMA_VERSION: &str = "kamn.runtime.observability.readiness.v1";
const OBSERVABILITY_STREAM_SCHEMA_VERSION: &str = "kamn.runtime.observability.stream.v1";
const OBSERVABILITY_READINESS_REASON_TAXONOMY_VERSION: &str =
    "kamn.runtime.observability.readiness.reason-taxonomy.v1";

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ObservabilityEndpointConfig {
    pub(crate) bind_addr: String,
    pub(crate) metrics_path: String,
    pub(crate) health_path: String,
    pub(crate) max_requests: u64,
    pub(crate) idle_timeout_ms: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct RuntimeObservabilitySnapshot {
    pub(crate) source: String,
    pub(crate) runtime_mode: String,
    pub(crate) latency_p50_ms: u64,
    pub(crate) latency_p99_ms: u64,
    pub(crate) throughput_tps: u64,
    pub(crate) error_rate_bps: u64,
    pub(crate) availability_bps: u64,
    pub(crate) health: String,
    pub(crate) alert_count: usize,
    pub(crate) reason_code: String,
    pub(crate) transport_checkpoint_failures: u64,
    pub(crate) signer_checkpoint_failures: u64,
    pub(crate) commit_checkpoint_failures: u64,
}

#[derive(Debug)]
struct ObservabilityRequestBudget {
    max_requests: u64,
    served_requests: AtomicU64,
    completion: Notify,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum ObservabilityEndpointTlsMode {
    Disabled,
    Require { cert_file: String, key_file: String },
}

#[cfg(test)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum ObservabilityEndpointTlsModeOverride {
    Require { cert_file: String, key_file: String },
}

#[cfg(test)]
thread_local! {
    static OBSERVABILITY_ENDPOINT_TLS_MODE_OVERRIDE_FOR_TESTS: RefCell<Option<ObservabilityEndpointTlsModeOverride>> =
        const { RefCell::new(None) };
}

#[cfg(test)]
pub(crate) fn set_observability_endpoint_tls_mode_override_for_current_thread_for_tests(
    mode: Option<ObservabilityEndpointTlsModeOverride>,
) {
    OBSERVABILITY_ENDPOINT_TLS_MODE_OVERRIDE_FOR_TESTS.with(|tls_mode_override| {
        tls_mode_override.replace(mode);
    });
}

impl ObservabilityRequestBudget {
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
struct ObservabilityEndpointRuntimeState {
    snapshot: RuntimeObservabilitySnapshot,
    metrics_path: String,
    health_path: String,
    request_budget: Arc<ObservabilityRequestBudget>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ObservabilityEndpointResponse {
    pub(crate) status_code: u16,
    pub(crate) content_type: &'static str,
    pub(crate) body: String,
}

pub(crate) fn build_runtime_observability_snapshot(
    report: &NodeBootstrapReport,
) -> Option<RuntimeObservabilitySnapshot> {
    if let (
        Some(latency_p50_ms),
        Some(latency_p99_ms),
        Some(throughput_tps),
        Some(error_rate_bps),
        Some(availability_bps),
        Some(health),
        Some(alert_count),
        Some(reason_code),
        Some(transport_checkpoint_failures),
        Some(signer_checkpoint_failures),
        Some(commit_checkpoint_failures),
    ) = (
        report.daemon_observability_latency_p50_ms,
        report.daemon_observability_latency_p99_ms,
        report.daemon_observability_throughput_tps,
        report.daemon_observability_error_rate_bps,
        report.daemon_observability_availability_bps,
        report.daemon_observability_health.as_deref(),
        report.daemon_observability_alert_count,
        report.daemon_observability_reason_code.as_deref(),
        report.daemon_observability_transport_checkpoint_failures,
        report.daemon_observability_signer_checkpoint_failures,
        report.daemon_observability_commit_checkpoint_failures,
    ) {
        return Some(RuntimeObservabilitySnapshot {
            source: "daemon".to_owned(),
            runtime_mode: report.runtime_mode.clone(),
            latency_p50_ms,
            latency_p99_ms,
            throughput_tps,
            error_rate_bps,
            availability_bps,
            health: health.to_owned(),
            alert_count,
            reason_code: reason_code.to_owned(),
            transport_checkpoint_failures,
            signer_checkpoint_failures,
            commit_checkpoint_failures,
        });
    }

    if let (
        Some(latency_p50_ms),
        Some(latency_p99_ms),
        Some(throughput_tps),
        Some(error_rate_bps),
        Some(availability_bps),
        Some(health),
        Some(alert_count),
        Some(reason_code),
        Some(transport_checkpoint_failures),
        Some(signer_checkpoint_failures),
        Some(commit_checkpoint_failures),
    ) = (
        report.kolme_live_observability_latency_p50_ms,
        report.kolme_live_observability_latency_p99_ms,
        report.kolme_live_observability_throughput_tps,
        report.kolme_live_observability_error_rate_bps,
        report.kolme_live_observability_availability_bps,
        report.kolme_live_observability_health.as_deref(),
        report.kolme_live_observability_alert_count,
        report.kolme_live_observability_reason_code.as_deref(),
        report.kolme_live_observability_transport_checkpoint_failures,
        report.kolme_live_observability_signer_checkpoint_failures,
        report.kolme_live_observability_commit_checkpoint_failures,
    ) {
        return Some(RuntimeObservabilitySnapshot {
            source: "kolme-live".to_owned(),
            runtime_mode: report.runtime_mode.clone(),
            latency_p50_ms,
            latency_p99_ms,
            throughput_tps,
            error_rate_bps,
            availability_bps,
            health: health.to_owned(),
            alert_count,
            reason_code: reason_code.to_owned(),
            transport_checkpoint_failures,
            signer_checkpoint_failures,
            commit_checkpoint_failures,
        });
    }

    None
}

#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn render_observability_endpoint_response(
    snapshot: &RuntimeObservabilitySnapshot,
    path: &str,
) -> ObservabilityEndpointResponse {
    render_observability_endpoint_response_with_paths(
        snapshot,
        path,
        DEFAULT_OBSERVABILITY_ENDPOINT_METRICS_PATH,
        DEFAULT_OBSERVABILITY_ENDPOINT_HEALTH_PATH,
        DEFAULT_OBSERVABILITY_ENDPOINT_READINESS_PATH,
        DEFAULT_OBSERVABILITY_ENDPOINT_STREAM_PATH,
    )
}

pub(crate) fn serve_observability_endpoint(
    config: &ObservabilityEndpointConfig,
    snapshot: &RuntimeObservabilitySnapshot,
) -> Result<(), String> {
    if !config.metrics_path.starts_with('/') {
        return Err("observability metrics path must start with '/'".to_owned());
    }
    if !config.health_path.starts_with('/') {
        return Err("observability health path must start with '/'".to_owned());
    }
    if config.max_requests == 0 {
        return Err("observability endpoint max requests must be greater than zero".to_owned());
    }
    if config.idle_timeout_ms == 0 {
        return Err("observability endpoint idle timeout must be greater than zero".to_owned());
    }

    let runtime = Builder::new_current_thread()
        .enable_io()
        .enable_time()
        .build()
        .map_err(|error| format!("observability endpoint runtime init failed: {error}"))?;
    runtime.block_on(serve_observability_endpoint_async(
        config.clone(),
        snapshot.clone(),
    ))
}

async fn serve_observability_endpoint_async(
    config: ObservabilityEndpointConfig,
    snapshot: RuntimeObservabilitySnapshot,
) -> Result<(), String> {
    let tls_mode = resolve_observability_endpoint_tls_mode()?;
    let request_budget = Arc::new(ObservabilityRequestBudget::new(config.max_requests));
    let runtime_state = Arc::new(ObservabilityEndpointRuntimeState {
        snapshot,
        metrics_path: config.metrics_path,
        health_path: config.health_path,
        request_budget: request_budget.clone(),
    });
    let timeout_reached = Arc::new(AtomicBool::new(false));
    let timeout_flag = timeout_reached.clone();
    let deadline = Instant::now() + Duration::from_millis(config.idle_timeout_ms);
    let app = build_observability_endpoint_router(runtime_state);

    match tls_mode {
        ObservabilityEndpointTlsMode::Disabled => {
            let listener = tokio::net::TcpListener::bind(config.bind_addr.as_str())
                .await
                .map_err(|error| format!("observability endpoint bind failed: {error}"))?;
            axum::serve(listener, app.clone())
                .with_graceful_shutdown(async move {
                    let wait_for_budget = request_budget.wait_until_complete();
                    tokio::pin!(wait_for_budget);
                    let idle_timeout = tokio::time::sleep_until(deadline);
                    tokio::pin!(idle_timeout);
                    tokio::select! {
                        _ = &mut wait_for_budget => {}
                        _ = &mut idle_timeout => {
                            timeout_flag.store(true, Ordering::SeqCst);
                        }
                    }
                })
                .await
                .map_err(|error| format!("observability endpoint serve failed: {error}"))?;
        }
        ObservabilityEndpointTlsMode::Require {
            cert_file,
            key_file,
        } => {
            let bind_addr = config.bind_addr.parse::<SocketAddr>().map_err(|error| {
                format!(
                    "observability endpoint tls bind address parse failed: {}: {error}",
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
                    "observability endpoint tls config load failed: cert_file={cert_file}, key_file={key_file}: {error}"
                )
            })?;

            let handle = axum_server::Handle::new();
            let shutdown_handle = handle.clone();
            tokio::spawn(async move {
                let wait_for_budget = request_budget.wait_until_complete();
                tokio::pin!(wait_for_budget);
                let idle_timeout = tokio::time::sleep_until(deadline);
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
                .map_err(|error| format!("observability endpoint tls serve failed: {error}"))?;
        }
    }

    if timeout_reached.load(Ordering::SeqCst) {
        return Err(format!(
            "observability endpoint timed out after {} ms waiting for requests",
            config.idle_timeout_ms
        ));
    }

    Ok(())
}

fn resolve_observability_endpoint_tls_mode() -> Result<ObservabilityEndpointTlsMode, String> {
    #[cfg(test)]
    if let Some(tls_mode_override) = OBSERVABILITY_ENDPOINT_TLS_MODE_OVERRIDE_FOR_TESTS
        .with(|tls_mode_override| tls_mode_override.borrow().clone())
    {
        return match tls_mode_override {
            ObservabilityEndpointTlsModeOverride::Require {
                cert_file,
                key_file,
            } => {
                if cert_file.trim().is_empty() {
                    return Err(
                        "observability endpoint tls cert override must not be empty".to_owned()
                    );
                }
                if key_file.trim().is_empty() {
                    return Err(
                        "observability endpoint tls key override must not be empty".to_owned()
                    );
                }
                validate_observability_endpoint_tls_materials(
                    cert_file.as_str(),
                    key_file.as_str(),
                )?;
                Ok(ObservabilityEndpointTlsMode::Require {
                    cert_file,
                    key_file,
                })
            }
        };
    }

    match env::var(OBSERVABILITY_ENDPOINT_TLS_MODE_ENV) {
        Ok(value) => {
            let mode = value.trim().to_ascii_lowercase();
            if mode.is_empty() {
                return Err(format!(
                    "observability endpoint tls mode env must not be empty: {OBSERVABILITY_ENDPOINT_TLS_MODE_ENV}"
                ));
            }
            match mode.as_str() {
                OBSERVABILITY_ENDPOINT_TLS_MODE_DISABLED => {
                    Ok(ObservabilityEndpointTlsMode::Disabled)
                }
                OBSERVABILITY_ENDPOINT_TLS_MODE_REQUIRE => {
                    let cert_file = env::var(OBSERVABILITY_ENDPOINT_TLS_CERT_FILE_ENV)
                        .map_err(|_| {
                            format!(
                                "observability endpoint tls mode requires env: {OBSERVABILITY_ENDPOINT_TLS_CERT_FILE_ENV}"
                            )
                        })?
                        .trim()
                        .to_owned();
                    if cert_file.is_empty() {
                        return Err(format!(
                            "observability endpoint tls cert env must not be empty: {OBSERVABILITY_ENDPOINT_TLS_CERT_FILE_ENV}"
                        ));
                    }
                    let key_file = env::var(OBSERVABILITY_ENDPOINT_TLS_KEY_FILE_ENV)
                        .map_err(|_| {
                            format!(
                                "observability endpoint tls mode requires env: {OBSERVABILITY_ENDPOINT_TLS_KEY_FILE_ENV}"
                            )
                        })?
                        .trim()
                        .to_owned();
                    if key_file.is_empty() {
                        return Err(format!(
                            "observability endpoint tls key env must not be empty: {OBSERVABILITY_ENDPOINT_TLS_KEY_FILE_ENV}"
                        ));
                    }
                    validate_observability_endpoint_tls_materials(
                        cert_file.as_str(),
                        key_file.as_str(),
                    )?;
                    Ok(ObservabilityEndpointTlsMode::Require {
                        cert_file,
                        key_file,
                    })
                }
                other => Err(format!(
                    "observability endpoint tls mode is invalid: {other} (supported: {OBSERVABILITY_ENDPOINT_TLS_MODE_DISABLED},{OBSERVABILITY_ENDPOINT_TLS_MODE_REQUIRE})"
                )),
            }
        }
        Err(env::VarError::NotPresent) => Ok(ObservabilityEndpointTlsMode::Disabled),
        Err(env::VarError::NotUnicode(_)) => Err(format!(
            "observability endpoint tls mode env must be utf-8: {OBSERVABILITY_ENDPOINT_TLS_MODE_ENV}"
        )),
    }
}

fn validate_observability_endpoint_tls_materials(
    cert_file: &str,
    key_file: &str,
) -> Result<(), String> {
    let cert_bytes = fs::read(cert_file).map_err(|error| {
        format!("observability endpoint tls certificate file read failed: {cert_file}: {error}")
    })?;
    let key_bytes = fs::read(key_file).map_err(|error| {
        format!("observability endpoint tls key file read failed: {key_file}: {error}")
    })?;

    let mut cert_reader = BufReader::new(cert_bytes.as_slice());
    let certs = rustls_pemfile::certs(&mut cert_reader)
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| {
            format!(
                "observability endpoint tls certificate file parse failed: {cert_file}: {error}"
            )
        })?;
    if certs.is_empty() {
        return Err(format!(
            "observability endpoint tls certificate file parse failed: {cert_file}: no certificates found"
        ));
    }

    let mut key_reader = BufReader::new(key_bytes.as_slice());
    let private_key = rustls_pemfile::private_key(&mut key_reader).map_err(|error| {
        format!("observability endpoint tls key file parse failed: {key_file}: {error}")
    })?;
    if private_key.is_none() {
        return Err(format!(
            "observability endpoint tls key file parse failed: {key_file}: no private key found"
        ));
    }
    Ok(())
}

fn build_observability_endpoint_router(state: Arc<ObservabilityEndpointRuntimeState>) -> Router {
    Router::new()
        .route("/", any(handle_observability_http_route))
        .route("/{*path}", any(handle_observability_http_route))
        .with_state(state)
}

async fn handle_observability_http_route(
    State(state): State<Arc<ObservabilityEndpointRuntimeState>>,
    method: Method,
    uri: Uri,
    headers: HeaderMap,
) -> Response {
    let response = if method != Method::GET || request_has_non_empty_body(&headers) {
        handle_observability_not_found_path().await
    } else {
        dispatch_observability_endpoint_request(
            &state.snapshot,
            uri.path(),
            state.metrics_path.as_str(),
            state.health_path.as_str(),
            DEFAULT_OBSERVABILITY_ENDPOINT_READINESS_PATH,
            DEFAULT_OBSERVABILITY_ENDPOINT_STREAM_PATH,
        )
        .await
    };
    state.request_budget.record_request();
    render_observability_http_response(response)
}

fn request_has_non_empty_body(headers: &HeaderMap) -> bool {
    if headers.contains_key(TRANSFER_ENCODING) {
        return true;
    }

    match headers.get(CONTENT_LENGTH) {
        Some(content_length) => match content_length.to_str() {
            Ok(value) => value.parse::<u64>().map_or(true, |parsed| parsed > 0),
            Err(_) => true,
        },
        None => false,
    }
}

async fn dispatch_observability_endpoint_request(
    snapshot: &RuntimeObservabilitySnapshot,
    request_path: &str,
    metrics_path: &str,
    health_path: &str,
    readiness_path: &str,
    stream_path: &str,
) -> ObservabilityEndpointResponse {
    if request_path == metrics_path {
        return handle_observability_metrics_path(snapshot).await;
    }
    if request_path == health_path {
        return handle_observability_health_path(snapshot).await;
    }
    if request_path == readiness_path {
        return handle_observability_readiness_path(snapshot).await;
    }
    if request_path == stream_path {
        return handle_observability_stream_path(snapshot).await;
    }
    handle_observability_not_found_path().await
}

async fn handle_observability_metrics_path(
    snapshot: &RuntimeObservabilitySnapshot,
) -> ObservabilityEndpointResponse {
    ObservabilityEndpointResponse {
        status_code: 200,
        content_type: "text/plain; version=0.0.4",
        body: render_metrics_body(snapshot),
    }
}

async fn handle_observability_health_path(
    snapshot: &RuntimeObservabilitySnapshot,
) -> ObservabilityEndpointResponse {
    ObservabilityEndpointResponse {
        status_code: 200,
        content_type: "application/json",
        body: render_health_body(snapshot),
    }
}

async fn handle_observability_readiness_path(
    snapshot: &RuntimeObservabilitySnapshot,
) -> ObservabilityEndpointResponse {
    ObservabilityEndpointResponse {
        status_code: 200,
        content_type: "application/json",
        body: render_readiness_body(snapshot),
    }
}

async fn handle_observability_stream_path(
    snapshot: &RuntimeObservabilitySnapshot,
) -> ObservabilityEndpointResponse {
    ObservabilityEndpointResponse {
        status_code: 200,
        content_type: "application/x-ndjson",
        body: render_stream_body(snapshot),
    }
}

async fn handle_observability_not_found_path() -> ObservabilityEndpointResponse {
    ObservabilityEndpointResponse {
        status_code: 404,
        content_type: "text/plain; charset=utf-8",
        body: "not found\n".to_owned(),
    }
}

fn render_observability_endpoint_response_with_paths(
    snapshot: &RuntimeObservabilitySnapshot,
    path: &str,
    metrics_path: &str,
    health_path: &str,
    readiness_path: &str,
    stream_path: &str,
) -> ObservabilityEndpointResponse {
    if path == metrics_path {
        return ObservabilityEndpointResponse {
            status_code: 200,
            content_type: "text/plain; version=0.0.4",
            body: render_metrics_body(snapshot),
        };
    }
    if path == health_path {
        return ObservabilityEndpointResponse {
            status_code: 200,
            content_type: "application/json",
            body: render_health_body(snapshot),
        };
    }
    if path == readiness_path {
        return ObservabilityEndpointResponse {
            status_code: 200,
            content_type: "application/json",
            body: render_readiness_body(snapshot),
        };
    }
    if path == stream_path {
        return ObservabilityEndpointResponse {
            status_code: 200,
            content_type: "application/x-ndjson",
            body: render_stream_body(snapshot),
        };
    }
    ObservabilityEndpointResponse {
        status_code: 404,
        content_type: "text/plain; charset=utf-8",
        body: "not found\n".to_owned(),
    }
}

fn render_metrics_body(snapshot: &RuntimeObservabilitySnapshot) -> String {
    let health_value = if snapshot.health == "healthy" { 1 } else { 0 };
    let ready_value = if is_runtime_ready(snapshot) { 1 } else { 0 };
    let readiness_reason_code = readiness_reason_code(snapshot);
    let transport_status = transport_dependency_status(snapshot);
    let signer_status = signer_dependency_status(snapshot);
    let commit_status = commit_dependency_status(snapshot);
    format!(
        "kamn_observability_latency_p50_ms {}\nkamn_observability_latency_p99_ms {}\nkamn_observability_throughput_tps {}\nkamn_observability_error_rate_bps {}\nkamn_observability_availability_bps {}\nkamn_observability_alert_count {}\nkamn_observability_transport_checkpoint_failures {}\nkamn_observability_signer_checkpoint_failures {}\nkamn_observability_commit_checkpoint_failures {}\nkamn_observability_ready {}\nkamn_observability_source{{source=\"{}\"}} 1\nkamn_observability_runtime_mode{{runtime_mode=\"{}\"}} 1\nkamn_observability_reason_code{{reason_code=\"{}\"}} 1\nkamn_observability_readiness_reason_code{{readiness_reason_code=\"{}\"}} 1\nkamn_observability_transport_dependency_status{{status=\"{}\"}} 1\nkamn_observability_signer_dependency_status{{status=\"{}\"}} 1\nkamn_observability_commit_dependency_status{{status=\"{}\"}} 1\nkamn_observability_health{{health=\"{}\"}} {}\n",
        snapshot.latency_p50_ms,
        snapshot.latency_p99_ms,
        snapshot.throughput_tps,
        snapshot.error_rate_bps,
        snapshot.availability_bps,
        snapshot.alert_count,
        snapshot.transport_checkpoint_failures,
        snapshot.signer_checkpoint_failures,
        snapshot.commit_checkpoint_failures,
        ready_value,
        escape_metrics_label(snapshot.source.as_str()),
        escape_metrics_label(snapshot.runtime_mode.as_str()),
        escape_metrics_label(snapshot.reason_code.as_str()),
        escape_metrics_label(readiness_reason_code),
        escape_metrics_label(transport_status),
        escape_metrics_label(signer_status),
        escape_metrics_label(commit_status),
        escape_metrics_label(snapshot.health.as_str()),
        health_value
    )
}

fn render_health_body(snapshot: &RuntimeObservabilitySnapshot) -> String {
    let readiness_reason_code = readiness_reason_code(snapshot);
    format!(
        "{{\"schema_version\":\"{}\",\"source\":\"{}\",\"runtime_mode\":\"{}\",\"health\":\"{}\",\"alert_count\":{},\"reason_code\":\"{}\",\"ready\":{},\"readiness_reason_code\":\"{}\",\"readiness_reason_taxonomy_version\":\"{}\",\"transport_dependency_status\":\"{}\",\"signer_dependency_status\":\"{}\",\"commit_dependency_status\":\"{}\",\"transport_checkpoint_failures\":{},\"signer_checkpoint_failures\":{},\"commit_checkpoint_failures\":{},\"latency_p50_ms\":{},\"latency_p99_ms\":{},\"throughput_tps\":{},\"error_rate_bps\":{},\"availability_bps\":{}}}",
        OBSERVABILITY_HEALTH_SCHEMA_VERSION,
        escape_json_string(snapshot.source.as_str()),
        escape_json_string(snapshot.runtime_mode.as_str()),
        escape_json_string(snapshot.health.as_str()),
        snapshot.alert_count,
        escape_json_string(snapshot.reason_code.as_str()),
        is_runtime_ready(snapshot),
        escape_json_string(readiness_reason_code),
        OBSERVABILITY_READINESS_REASON_TAXONOMY_VERSION,
        transport_dependency_status(snapshot),
        signer_dependency_status(snapshot),
        commit_dependency_status(snapshot),
        snapshot.transport_checkpoint_failures,
        snapshot.signer_checkpoint_failures,
        snapshot.commit_checkpoint_failures,
        snapshot.latency_p50_ms,
        snapshot.latency_p99_ms,
        snapshot.throughput_tps,
        snapshot.error_rate_bps,
        snapshot.availability_bps
    )
}

fn render_stream_body(snapshot: &RuntimeObservabilitySnapshot) -> String {
    let readiness_reason_code = readiness_reason_code(snapshot);
    format!(
        "{{\"schema_version\":\"{}\",\"source\":\"{}\",\"runtime_mode\":\"{}\",\"health\":\"{}\",\"alert_count\":{},\"reason_code\":\"{}\",\"ready\":{},\"readiness_reason_code\":\"{}\",\"transport_dependency_status\":\"{}\",\"signer_dependency_status\":\"{}\",\"commit_dependency_status\":\"{}\",\"transport_checkpoint_failures\":{},\"signer_checkpoint_failures\":{},\"commit_checkpoint_failures\":{},\"latency_p50_ms\":{},\"latency_p99_ms\":{},\"throughput_tps\":{},\"error_rate_bps\":{},\"availability_bps\":{}}}\n",
        OBSERVABILITY_STREAM_SCHEMA_VERSION,
        escape_json_string(snapshot.source.as_str()),
        escape_json_string(snapshot.runtime_mode.as_str()),
        escape_json_string(snapshot.health.as_str()),
        snapshot.alert_count,
        escape_json_string(snapshot.reason_code.as_str()),
        is_runtime_ready(snapshot),
        escape_json_string(readiness_reason_code),
        transport_dependency_status(snapshot),
        signer_dependency_status(snapshot),
        commit_dependency_status(snapshot),
        snapshot.transport_checkpoint_failures,
        snapshot.signer_checkpoint_failures,
        snapshot.commit_checkpoint_failures,
        snapshot.latency_p50_ms,
        snapshot.latency_p99_ms,
        snapshot.throughput_tps,
        snapshot.error_rate_bps,
        snapshot.availability_bps
    )
}

fn render_readiness_body(snapshot: &RuntimeObservabilitySnapshot) -> String {
    let readiness_reason_code = readiness_reason_code(snapshot);
    format!(
        "{{\"schema_version\":\"{}\",\"source\":\"{}\",\"runtime_mode\":\"{}\",\"ready\":{},\"health\":\"{}\",\"reason_code\":\"{}\",\"readiness_reason_code\":\"{}\",\"readiness_reason_taxonomy_version\":\"{}\",\"transport_dependency_status\":\"{}\",\"signer_dependency_status\":\"{}\",\"commit_dependency_status\":\"{}\",\"transport_checkpoint_failures\":{},\"signer_checkpoint_failures\":{},\"commit_checkpoint_failures\":{}}}",
        OBSERVABILITY_READINESS_SCHEMA_VERSION,
        escape_json_string(snapshot.source.as_str()),
        escape_json_string(snapshot.runtime_mode.as_str()),
        is_runtime_ready(snapshot),
        escape_json_string(snapshot.health.as_str()),
        escape_json_string(snapshot.reason_code.as_str()),
        escape_json_string(readiness_reason_code),
        OBSERVABILITY_READINESS_REASON_TAXONOMY_VERSION,
        transport_dependency_status(snapshot),
        signer_dependency_status(snapshot),
        commit_dependency_status(snapshot),
        snapshot.transport_checkpoint_failures,
        snapshot.signer_checkpoint_failures,
        snapshot.commit_checkpoint_failures
    )
}

fn is_runtime_ready(snapshot: &RuntimeObservabilitySnapshot) -> bool {
    snapshot.transport_checkpoint_failures == 0
        && snapshot.signer_checkpoint_failures == 0
        && snapshot.commit_checkpoint_failures == 0
        && snapshot.health == "healthy"
}

fn readiness_reason_code(snapshot: &RuntimeObservabilitySnapshot) -> &'static str {
    if snapshot.transport_checkpoint_failures > 0 {
        "readiness_transport_dependency_unhealthy"
    } else if snapshot.signer_checkpoint_failures > 0 {
        "readiness_signer_dependency_unhealthy"
    } else if snapshot.commit_checkpoint_failures > 0 {
        "readiness_commit_dependency_unhealthy"
    } else if snapshot.health != "healthy" {
        "readiness_runtime_health_degraded"
    } else {
        "none"
    }
}

fn transport_dependency_status(snapshot: &RuntimeObservabilitySnapshot) -> &'static str {
    if snapshot.transport_checkpoint_failures > 0 {
        "degraded"
    } else {
        "ready"
    }
}

fn signer_dependency_status(snapshot: &RuntimeObservabilitySnapshot) -> &'static str {
    if snapshot.signer_checkpoint_failures > 0 {
        "degraded"
    } else {
        "ready"
    }
}

fn commit_dependency_status(snapshot: &RuntimeObservabilitySnapshot) -> &'static str {
    if snapshot.commit_checkpoint_failures > 0 {
        "degraded"
    } else {
        "ready"
    }
}

fn render_observability_http_response(response: ObservabilityEndpointResponse) -> Response {
    let status_code =
        StatusCode::from_u16(response.status_code).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);
    let mut payload = Response::new(Body::from(response.body));
    *payload.status_mut() = status_code;
    payload.headers_mut().insert(
        CONTENT_TYPE,
        HeaderValue::from_static(response.content_type),
    );
    payload
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
        render_observability_endpoint_response, serve_observability_endpoint,
        ObservabilityEndpointConfig, RuntimeObservabilitySnapshot,
    };
    use std::io::{Read, Write};
    use std::net::{TcpListener, TcpStream};
    use std::thread;
    use std::time::{Duration, Instant};

    fn reserve_loopback_addr() -> String {
        let listener = TcpListener::bind("127.0.0.1:0").expect("listener should bind");
        let addr = listener.local_addr().expect("local addr should resolve");
        drop(listener);
        addr.to_string()
    }

    fn connect_with_retry(addr: &str) -> TcpStream {
        let deadline = Instant::now() + Duration::from_secs(2);
        while Instant::now() < deadline {
            if let Ok(stream) = TcpStream::connect(addr) {
                return stream;
            }
            thread::sleep(Duration::from_millis(10));
        }
        panic!("server should accept");
    }

    #[test]
    fn unit_observability_endpoint_response_returns_not_found_for_unknown_path() {
        let snapshot = RuntimeObservabilitySnapshot {
            source: "daemon".to_owned(),
            runtime_mode: "daemon".to_owned(),
            latency_p50_ms: 25,
            latency_p99_ms: 50,
            throughput_tps: 2_000,
            error_rate_bps: 50,
            availability_bps: 9_990,
            health: "healthy".to_owned(),
            alert_count: 0,
            reason_code: "none".to_owned(),
            transport_checkpoint_failures: 0,
            signer_checkpoint_failures: 0,
            commit_checkpoint_failures: 0,
        };
        let response = render_observability_endpoint_response(&snapshot, "/unknown");
        assert_eq!(response.status_code, 404);
        assert_eq!(response.content_type, "text/plain; charset=utf-8");
    }

    #[test]
    fn unit_observability_endpoint_readiness_reports_ready_with_none_reason_code() {
        let snapshot = RuntimeObservabilitySnapshot {
            source: "daemon".to_owned(),
            runtime_mode: "daemon".to_owned(),
            latency_p50_ms: 25,
            latency_p99_ms: 50,
            throughput_tps: 2_000,
            error_rate_bps: 50,
            availability_bps: 9_990,
            health: "healthy".to_owned(),
            alert_count: 0,
            reason_code: "none".to_owned(),
            transport_checkpoint_failures: 0,
            signer_checkpoint_failures: 0,
            commit_checkpoint_failures: 0,
        };
        let response = render_observability_endpoint_response(&snapshot, "/readyz");
        assert_eq!(response.status_code, 200);
        assert_eq!(response.content_type, "application/json");
        assert!(response.body.contains("\"ready\":true"));
        assert!(response.body.contains("\"readiness_reason_code\":\"none\""));
    }

    #[test]
    fn unit_observability_endpoint_readiness_reports_commit_degraded_reason_code() {
        let snapshot = RuntimeObservabilitySnapshot {
            source: "daemon".to_owned(),
            runtime_mode: "daemon".to_owned(),
            latency_p50_ms: 145,
            latency_p99_ms: 425,
            throughput_tps: 900,
            error_rate_bps: 250,
            availability_bps: 9_800,
            health: "critical".to_owned(),
            alert_count: 4,
            reason_code: "daemon_shutdown_timeout".to_owned(),
            transport_checkpoint_failures: 0,
            signer_checkpoint_failures: 0,
            commit_checkpoint_failures: 1,
        };
        let response = render_observability_endpoint_response(&snapshot, "/readyz");
        assert_eq!(response.status_code, 200);
        assert_eq!(response.content_type, "application/json");
        assert!(response.body.contains("\"ready\":false"));
        assert!(response
            .body
            .contains("\"readiness_reason_code\":\"readiness_commit_dependency_unhealthy\""));
        assert!(response
            .body
            .contains("\"commit_dependency_status\":\"degraded\""));
    }

    #[test]
    fn integration_observability_endpoint_serves_request_budget() {
        let snapshot = RuntimeObservabilitySnapshot {
            source: "daemon".to_owned(),
            runtime_mode: "daemon".to_owned(),
            latency_p50_ms: 25,
            latency_p99_ms: 50,
            throughput_tps: 2_000,
            error_rate_bps: 50,
            availability_bps: 9_990,
            health: "healthy".to_owned(),
            alert_count: 0,
            reason_code: "none".to_owned(),
            transport_checkpoint_failures: 0,
            signer_checkpoint_failures: 0,
            commit_checkpoint_failures: 0,
        };
        let bind_addr = reserve_loopback_addr();
        let config = ObservabilityEndpointConfig {
            bind_addr: bind_addr.clone(),
            metrics_path: "/metrics".to_owned(),
            health_path: "/healthz".to_owned(),
            max_requests: 1,
            idle_timeout_ms: 2_000,
        };
        let server = thread::spawn(move || {
            serve_observability_endpoint(&config, &snapshot).expect("server")
        });
        let mut stream = connect_with_retry(bind_addr.as_str());
        stream
            .write_all(b"GET /metrics HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n")
            .expect("request should write");
        let mut response = String::new();
        stream
            .read_to_string(&mut response)
            .expect("response should be readable");
        assert!(response.contains("HTTP/1.1 200 OK"));
        server.join().expect("server thread should join");
    }
}
