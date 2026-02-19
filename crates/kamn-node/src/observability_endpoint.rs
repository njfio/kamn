mod endpoint_server;
mod payload_contract;
mod payload_render;
mod tls_mode;

use crate::NodeBootstrapReport;
use axum::{
    body::Body,
    http::{header::CONTENT_TYPE, HeaderValue, StatusCode},
    response::Response,
};
use tokio::runtime::Builder;

pub(crate) const DEFAULT_OBSERVABILITY_ENDPOINT_METRICS_PATH: &str = "/metrics";
pub(crate) const DEFAULT_OBSERVABILITY_ENDPOINT_HEALTH_PATH: &str = "/healthz";
pub(crate) const DEFAULT_OBSERVABILITY_ENDPOINT_READINESS_PATH: &str = "/readyz";
pub(crate) const DEFAULT_OBSERVABILITY_ENDPOINT_STREAM_PATH: &str = "/metrics.stream";
pub(crate) const DEFAULT_OBSERVABILITY_ENDPOINT_MAX_REQUESTS: u64 = 1;
pub(crate) const DEFAULT_OBSERVABILITY_ENDPOINT_IDLE_TIMEOUT_MS: u64 = 5_000;
const OBSERVABILITY_HEALTH_SCHEMA_VERSION: &str = "kamn.runtime.observability.health.v1";
const OBSERVABILITY_READINESS_SCHEMA_VERSION: &str = "kamn.runtime.observability.readiness.v1";
const OBSERVABILITY_STREAM_SCHEMA_VERSION: &str = "kamn.runtime.observability.stream.v1";
const OBSERVABILITY_READINESS_REASON_TAXONOMY_VERSION: &str =
    "kamn.runtime.observability.readiness.reason-taxonomy.v1";
const OBSERVABILITY_ENDPOINT_REASON_TAXONOMY_VERSION: &str =
    "kamn.runtime.observability-endpoint-reason-taxonomy.v1";
const OBSERVABILITY_ENDPOINT_FAIL_CLOSED_SCHEMA_VERSION: &str =
    "kamn.runtime.observability.endpoint-fail-closed.v1";
const OBSERVABILITY_ENDPOINT_REQUIRED_FIELD_MISSING_REASON_PREFIX: &str =
    "runtime_observability_policy_required_field_missing";
const OBSERVABILITY_ENDPOINT_SCHEMA_DRIFT_REASON_PREFIX: &str =
    "runtime_observability_policy_schema_drift";

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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ObservabilityEndpointPayloadSurface {
    Metrics,
    Health,
    Readiness,
    Stream,
}

impl ObservabilityEndpointPayloadSurface {
    fn reason_surface(self) -> &'static str {
        match self {
            Self::Metrics => "metrics",
            Self::Health => "health",
            Self::Readiness => "readiness",
            Self::Stream => "stream",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ObservabilityEndpointResponse {
    pub(crate) status_code: u16,
    pub(crate) content_type: &'static str,
    pub(crate) body: String,
}

#[cfg_attr(not(test), allow(unused_imports))]
pub(crate) use payload_contract::{
    enforce_observability_endpoint_payload_contract,
    validate_observability_endpoint_payload_contract,
};
#[cfg_attr(not(test), allow(unused_imports))]
pub(crate) use tls_mode::{
    set_observability_endpoint_tls_mode_override_for_current_thread_for_tests,
    ObservabilityEndpointTlsModeOverride,
};

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
    runtime.block_on(endpoint_server::serve_observability_endpoint_async(
        config.clone(),
        snapshot.clone(),
    ))
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
        return enforce_observability_endpoint_payload_contract(
            ObservabilityEndpointPayloadSurface::Metrics,
            "text/plain; version=0.0.4",
            payload_render::render_metrics_body(snapshot),
        );
    }
    if path == health_path {
        return enforce_observability_endpoint_payload_contract(
            ObservabilityEndpointPayloadSurface::Health,
            "application/json",
            payload_render::render_health_body(snapshot),
        );
    }
    if path == readiness_path {
        return enforce_observability_endpoint_payload_contract(
            ObservabilityEndpointPayloadSurface::Readiness,
            "application/json",
            payload_render::render_readiness_body(snapshot),
        );
    }
    if path == stream_path {
        return enforce_observability_endpoint_payload_contract(
            ObservabilityEndpointPayloadSurface::Stream,
            "application/x-ndjson",
            payload_render::render_stream_body(snapshot),
        );
    }
    ObservabilityEndpointResponse {
        status_code: 404,
        content_type: "text/plain; charset=utf-8",
        body: "not found\n".to_owned(),
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
