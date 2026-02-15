use crate::NodeBootstrapReport;
use std::io::{ErrorKind, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::thread;
use std::time::{Duration, Instant};

pub(crate) const DEFAULT_OBSERVABILITY_ENDPOINT_METRICS_PATH: &str = "/metrics";
pub(crate) const DEFAULT_OBSERVABILITY_ENDPOINT_HEALTH_PATH: &str = "/healthz";
pub(crate) const DEFAULT_OBSERVABILITY_ENDPOINT_READINESS_PATH: &str = "/readyz";
pub(crate) const DEFAULT_OBSERVABILITY_ENDPOINT_STREAM_PATH: &str = "/metrics.stream";
pub(crate) const DEFAULT_OBSERVABILITY_ENDPOINT_MAX_REQUESTS: u64 = 1;
pub(crate) const DEFAULT_OBSERVABILITY_ENDPOINT_IDLE_TIMEOUT_MS: u64 = 5_000;

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

    let listener = TcpListener::bind(config.bind_addr.as_str())
        .map_err(|error| format!("observability endpoint bind failed: {error}"))?;
    listener
        .set_nonblocking(true)
        .map_err(|error| format!("observability endpoint nonblocking mode failed: {error}"))?;

    let deadline = Instant::now() + Duration::from_millis(config.idle_timeout_ms);
    let mut served_requests = 0_u64;
    while served_requests < config.max_requests {
        if Instant::now() >= deadline {
            return Err(format!(
                "observability endpoint timed out after {} ms waiting for requests",
                config.idle_timeout_ms
            ));
        }
        match listener.accept() {
            Ok((mut stream, _)) => {
                let path = read_http_request_path(&mut stream).unwrap_or_else(|_| "/".to_owned());
                let response = render_observability_endpoint_response_with_paths(
                    snapshot,
                    path.as_str(),
                    config.metrics_path.as_str(),
                    config.health_path.as_str(),
                    DEFAULT_OBSERVABILITY_ENDPOINT_READINESS_PATH,
                    DEFAULT_OBSERVABILITY_ENDPOINT_STREAM_PATH,
                );
                write_http_response(&mut stream, &response)?;
                served_requests = served_requests.saturating_add(1);
            }
            Err(error) if error.kind() == ErrorKind::WouldBlock => {
                thread::sleep(Duration::from_millis(5));
            }
            Err(error) => {
                return Err(format!("observability endpoint accept failed: {error}"));
            }
        }
    }
    Ok(())
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
        "{{\"source\":\"{}\",\"runtime_mode\":\"{}\",\"health\":\"{}\",\"alert_count\":{},\"reason_code\":\"{}\",\"ready\":{},\"readiness_reason_code\":\"{}\",\"transport_dependency_status\":\"{}\",\"signer_dependency_status\":\"{}\",\"commit_dependency_status\":\"{}\",\"transport_checkpoint_failures\":{},\"signer_checkpoint_failures\":{},\"commit_checkpoint_failures\":{},\"latency_p50_ms\":{},\"latency_p99_ms\":{},\"throughput_tps\":{},\"error_rate_bps\":{},\"availability_bps\":{}}}",
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

fn render_stream_body(snapshot: &RuntimeObservabilitySnapshot) -> String {
    let readiness_reason_code = readiness_reason_code(snapshot);
    format!(
        "{{\"schema_version\":\"kamn.runtime.observability.stream.v1\",\"source\":\"{}\",\"runtime_mode\":\"{}\",\"health\":\"{}\",\"alert_count\":{},\"reason_code\":\"{}\",\"ready\":{},\"readiness_reason_code\":\"{}\",\"transport_dependency_status\":\"{}\",\"signer_dependency_status\":\"{}\",\"commit_dependency_status\":\"{}\",\"transport_checkpoint_failures\":{},\"signer_checkpoint_failures\":{},\"commit_checkpoint_failures\":{},\"latency_p50_ms\":{},\"latency_p99_ms\":{},\"throughput_tps\":{},\"error_rate_bps\":{},\"availability_bps\":{}}}\n",
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
        "{{\"source\":\"{}\",\"runtime_mode\":\"{}\",\"ready\":{},\"health\":\"{}\",\"reason_code\":\"{}\",\"readiness_reason_code\":\"{}\",\"transport_dependency_status\":\"{}\",\"signer_dependency_status\":\"{}\",\"commit_dependency_status\":\"{}\",\"transport_checkpoint_failures\":{},\"signer_checkpoint_failures\":{},\"commit_checkpoint_failures\":{}}}",
        escape_json_string(snapshot.source.as_str()),
        escape_json_string(snapshot.runtime_mode.as_str()),
        is_runtime_ready(snapshot),
        escape_json_string(snapshot.health.as_str()),
        escape_json_string(snapshot.reason_code.as_str()),
        escape_json_string(readiness_reason_code),
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

fn write_http_response(
    stream: &mut TcpStream,
    response: &ObservabilityEndpointResponse,
) -> Result<(), String> {
    let status_text = match response.status_code {
        200 => "200 OK",
        404 => "404 Not Found",
        _ => "500 Internal Server Error",
    };
    let payload = format!(
        "HTTP/1.1 {status_text}\r\nContent-Type: {}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
        response.content_type,
        response.body.len(),
        response.body
    );
    stream
        .write_all(payload.as_bytes())
        .map_err(|error| format!("observability endpoint write failed: {error}"))
}

fn read_http_request_path(stream: &mut TcpStream) -> Result<String, String> {
    let mut request = Vec::new();
    let mut chunk = [0_u8; 256];
    stream
        .set_read_timeout(Some(Duration::from_secs(1)))
        .map_err(|error| format!("observability endpoint read-timeout failed: {error}"))?;
    loop {
        match stream.read(&mut chunk) {
            Ok(0) => break,
            Ok(read_count) => {
                request.extend_from_slice(&chunk[..read_count]);
                if request.windows(2).any(|window| window == b"\r\n") {
                    break;
                }
                if request.len() > 8 * 1024 {
                    return Err("observability endpoint request header too large".to_owned());
                }
            }
            Err(error) if matches!(error.kind(), ErrorKind::WouldBlock | ErrorKind::TimedOut) => {
                break;
            }
            Err(error) => {
                return Err(format!("observability endpoint read failed: {error}"));
            }
        }
    }
    let request = String::from_utf8(request)
        .map_err(|_| "observability endpoint request was not valid utf-8".to_owned())?;
    let request_line = request
        .lines()
        .next()
        .ok_or_else(|| "observability endpoint request line missing".to_owned())?;
    let mut parts = request_line.split_whitespace();
    let method = parts
        .next()
        .ok_or_else(|| "observability endpoint request method missing".to_owned())?;
    let path = parts
        .next()
        .ok_or_else(|| "observability endpoint request path missing".to_owned())?;
    if method != "GET" {
        return Err("observability endpoint only supports GET".to_owned());
    }
    Ok(path.to_owned())
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
