use super::*;
use crate::observability_endpoint::RuntimeObservabilitySnapshot;
use std::io::{ErrorKind, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Barrier};
use std::thread;
use std::time::{Duration, Instant};

fn reserve_loopback_addr() -> String {
    let listener = TcpListener::bind("127.0.0.1:0").expect("listener should bind");
    let addr = listener.local_addr().expect("local addr should resolve");
    drop(listener);
    addr.to_string()
}

fn send_http_get(addr: &str, path: &str) -> String {
    let mut stream = TcpStream::connect(addr).expect("endpoint should accept connections");
    stream
        .set_read_timeout(Some(Duration::from_secs(2)))
        .expect("read timeout should be configurable");
    let request = format!("GET {path} HTTP/1.1\r\nHost: {addr}\r\nConnection: close\r\n\r\n");
    stream
        .write_all(request.as_bytes())
        .expect("request should write");
    let mut response = String::new();
    let mut chunk = [0_u8; 1024];
    loop {
        match stream.read(&mut chunk) {
            Ok(0) => break,
            Ok(read_count) => {
                response.push_str(
                    std::str::from_utf8(&chunk[..read_count]).expect("response must be utf-8"),
                );
            }
            Err(error) if matches!(error.kind(), ErrorKind::WouldBlock | ErrorKind::TimedOut) => {
                break;
            }
            Err(error) => panic!("response should be readable: {error}"),
        }
    }
    response
}

fn try_send_http_get(addr: &str, path: &str) -> Result<String, String> {
    let mut stream =
        TcpStream::connect(addr).map_err(|error| format!("connect should succeed: {error}"))?;
    stream
        .set_read_timeout(Some(Duration::from_secs(2)))
        .map_err(|error| format!("read timeout should be configurable: {error}"))?;
    let request = format!("GET {path} HTTP/1.1\r\nHost: {addr}\r\nConnection: close\r\n\r\n");
    stream
        .write_all(request.as_bytes())
        .map_err(|error| format!("request should write: {error}"))?;
    let mut response = String::new();
    let mut chunk = [0_u8; 1024];
    loop {
        match stream.read(&mut chunk) {
            Ok(0) => break,
            Ok(read_count) => {
                response.push_str(
                    std::str::from_utf8(&chunk[..read_count])
                        .map_err(|error| format!("response must be utf-8: {error}"))?,
                );
            }
            Err(error) if matches!(error.kind(), ErrorKind::WouldBlock | ErrorKind::TimedOut) => {
                break;
            }
            Err(error) => return Err(format!("response should be readable: {error}")),
        }
    }
    Ok(response)
}

fn send_raw_http_request(addr: &str, request: &str) -> String {
    let mut stream = TcpStream::connect(addr).expect("endpoint should accept connections");
    stream
        .set_read_timeout(Some(Duration::from_secs(2)))
        .expect("read timeout should be configurable");
    stream
        .write_all(request.as_bytes())
        .expect("request should write");
    let mut response = String::new();
    let mut chunk = [0_u8; 1024];
    loop {
        match stream.read(&mut chunk) {
            Ok(0) => break,
            Ok(read_count) => {
                response.push_str(
                    std::str::from_utf8(&chunk[..read_count]).expect("response must be utf-8"),
                );
            }
            Err(error) if matches!(error.kind(), ErrorKind::WouldBlock | ErrorKind::TimedOut) => {
                break;
            }
            Err(error) => panic!("response should be readable: {error}"),
        }
    }
    response
}

fn wait_for_endpoint_ready(addr: &str) {
    let deadline = Instant::now() + Duration::from_secs(2);
    while Instant::now() < deadline {
        if TcpStream::connect(addr).is_ok() {
            return;
        }
        thread::sleep(Duration::from_millis(10));
    }
    panic!("endpoint did not become ready within timeout");
}

fn sample_observability_snapshot() -> RuntimeObservabilitySnapshot {
    RuntimeObservabilitySnapshot {
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
    }
}

#[test]
fn unit_observability_endpoint_rejects_metrics_path_without_leading_slash() {
    let config = ObservabilityEndpointConfig {
        bind_addr: "127.0.0.1:0".to_owned(),
        metrics_path: "metrics".to_owned(),
        health_path: "/healthz".to_owned(),
        max_requests: 1,
        idle_timeout_ms: 1_000,
    };
    let snapshot = sample_observability_snapshot();

    let error = serve_observability_endpoint(&config, &snapshot)
        .expect_err("metrics path without leading slash must fail");
    assert_eq!(error, "observability metrics path must start with '/'");
}

#[test]
fn unit_observability_endpoint_rejects_health_path_without_leading_slash() {
    let config = ObservabilityEndpointConfig {
        bind_addr: "127.0.0.1:0".to_owned(),
        metrics_path: "/metrics".to_owned(),
        health_path: "healthz".to_owned(),
        max_requests: 1,
        idle_timeout_ms: 1_000,
    };
    let snapshot = sample_observability_snapshot();

    let error = serve_observability_endpoint(&config, &snapshot)
        .expect_err("health path without leading slash must fail");
    assert_eq!(error, "observability health path must start with '/'");
}

#[test]
fn unit_observability_endpoint_rejects_zero_request_budget() {
    let config = ObservabilityEndpointConfig {
        bind_addr: "127.0.0.1:0".to_owned(),
        metrics_path: "/metrics".to_owned(),
        health_path: "/healthz".to_owned(),
        max_requests: 0,
        idle_timeout_ms: 1_000,
    };
    let snapshot = sample_observability_snapshot();

    let error = serve_observability_endpoint(&config, &snapshot)
        .expect_err("zero request budget must fail");
    assert_eq!(
        error,
        "observability endpoint max requests must be greater than zero"
    );
}

#[test]
fn unit_observability_endpoint_rejects_zero_idle_timeout_budget() {
    let config = ObservabilityEndpointConfig {
        bind_addr: "127.0.0.1:0".to_owned(),
        metrics_path: "/metrics".to_owned(),
        health_path: "/healthz".to_owned(),
        max_requests: 1,
        idle_timeout_ms: 0,
    };
    let snapshot = sample_observability_snapshot();

    let error =
        serve_observability_endpoint(&config, &snapshot).expect_err("zero idle timeout must fail");
    assert_eq!(
        error,
        "observability endpoint idle timeout must be greater than zero"
    );
}

#[test]
fn unit_observability_endpoint_maps_daemon_telemetry_into_snapshot() {
    let parsed = parse_args(vec![
        "kamn-node".to_owned(),
        "--role".to_owned(),
        "processor".to_owned(),
        "--runtime-mode".to_owned(),
        "daemon".to_owned(),
        "--daemon-max-ticks".to_owned(),
        "3".to_owned(),
        "--daemon-tick-interval-ms".to_owned(),
        "25".to_owned(),
    ])
    .expect("daemon args should parse");
    let report = execute(parsed).expect("daemon execution should succeed");

    let snapshot =
        build_runtime_observability_snapshot(&report).expect("daemon report should map snapshot");
    assert_eq!(snapshot.source, "daemon");
    assert_eq!(snapshot.runtime_mode, "daemon");
    assert_eq!(snapshot.latency_p50_ms, 25);
    assert_eq!(snapshot.latency_p99_ms, 50);
    assert_eq!(snapshot.throughput_tps, 2_000);
    assert_eq!(snapshot.error_rate_bps, 50);
    assert_eq!(snapshot.availability_bps, 9_990);
    assert_eq!(snapshot.health, "healthy");
    assert_eq!(snapshot.alert_count, 0);
}

#[test]
fn functional_observability_endpoint_renders_metrics_and_health_payloads() {
    let parsed = parse_args(vec![
        "kamn-node".to_owned(),
        "--role".to_owned(),
        "processor".to_owned(),
        "--runtime-mode".to_owned(),
        "daemon".to_owned(),
        "--daemon-max-ticks".to_owned(),
        "3".to_owned(),
        "--daemon-tick-interval-ms".to_owned(),
        "25".to_owned(),
    ])
    .expect("daemon args should parse");
    let report = execute(parsed).expect("daemon execution should succeed");
    let snapshot =
        build_runtime_observability_snapshot(&report).expect("daemon report should map snapshot");

    let metrics = render_observability_endpoint_response(&snapshot, "/metrics");
    assert_eq!(metrics.status_code, 200);
    assert_eq!(metrics.content_type, "text/plain; version=0.0.4");
    assert!(metrics
        .body
        .contains("kamn_observability_latency_p50_ms 25"));
    assert!(metrics
        .body
        .contains("kamn_observability_reason_code{reason_code=\"none\"} 1"));
    assert!(metrics
        .body
        .contains("kamn_observability_readiness_reason_code{readiness_reason_code=\"none\"} 1"));
    assert!(metrics.body.contains("kamn_observability_ready 1"));
    assert!(metrics
        .body
        .contains("kamn_observability_health{health=\"healthy\"} 1"));

    let health = render_observability_endpoint_response(&snapshot, "/healthz");
    assert_eq!(health.status_code, 200);
    assert_eq!(health.content_type, "application/json");
    assert!(health.body.contains("\"health\":\"healthy\""));
    assert!(health.body.contains("\"runtime_mode\":\"daemon\""));
    assert!(health.body.contains("\"reason_code\":\"none\""));
    assert!(health.body.contains("\"ready\":true"));
    assert!(health.body.contains("\"readiness_reason_code\":\"none\""));
    assert!(health
        .body
        .contains("\"transport_dependency_status\":\"ready\""));
    assert!(health
        .body
        .contains("\"signer_dependency_status\":\"ready\""));
    assert!(health
        .body
        .contains("\"commit_dependency_status\":\"ready\""));
    assert!(health.body.contains("\"transport_checkpoint_failures\":0"));
    assert!(health.body.contains("\"signer_checkpoint_failures\":0"));
    assert!(health.body.contains("\"commit_checkpoint_failures\":0"));

    let readiness = render_observability_endpoint_response(&snapshot, "/readyz");
    assert_eq!(readiness.status_code, 200);
    assert_eq!(readiness.content_type, "application/json");
    assert!(readiness.body.contains("\"ready\":true"));
    assert!(readiness
        .body
        .contains("\"readiness_reason_code\":\"none\""));
    assert!(readiness
        .body
        .contains("\"transport_dependency_status\":\"ready\""));
    assert!(readiness
        .body
        .contains("\"signer_dependency_status\":\"ready\""));
    assert!(readiness
        .body
        .contains("\"commit_dependency_status\":\"ready\""));
}

#[test]
fn functional_observability_endpoint_renders_stream_payload() {
    let parsed = parse_args(vec![
        "kamn-node".to_owned(),
        "--role".to_owned(),
        "processor".to_owned(),
        "--runtime-mode".to_owned(),
        "daemon".to_owned(),
        "--daemon-max-ticks".to_owned(),
        "3".to_owned(),
        "--daemon-tick-interval-ms".to_owned(),
        "25".to_owned(),
    ])
    .expect("daemon args should parse");
    let report = execute(parsed).expect("daemon execution should succeed");
    let snapshot =
        build_runtime_observability_snapshot(&report).expect("daemon report should map snapshot");

    let stream = render_observability_endpoint_response(&snapshot, "/metrics.stream");
    assert_eq!(stream.status_code, 200);
    assert_eq!(stream.content_type, "application/x-ndjson");
    assert!(stream
        .body
        .contains("\"schema_version\":\"kamn.runtime.observability.stream.v1\""));
    assert!(stream.body.contains("\"reason_code\":\"none\""));
    assert!(stream.body.contains("\"ready\":true"));
    assert!(stream.body.contains("\"readiness_reason_code\":\"none\""));
    assert!(stream
        .body
        .contains("\"transport_dependency_status\":\"ready\""));
    assert!(stream
        .body
        .contains("\"signer_dependency_status\":\"ready\""));
    assert!(stream
        .body
        .contains("\"commit_dependency_status\":\"ready\""));
    assert!(stream.body.contains("\"transport_checkpoint_failures\":0"));
    assert!(stream.body.contains("\"signer_checkpoint_failures\":0"));
    assert!(stream.body.contains("\"commit_checkpoint_failures\":0"));
    assert!(stream.body.ends_with('\n'));
}

#[test]
fn functional_observability_endpoint_readiness_reports_degraded_timeout_reason_codes() {
    let parsed = parse_args(vec![
        "kamn-node".to_owned(),
        "--role".to_owned(),
        "processor".to_owned(),
        "--runtime-mode".to_owned(),
        "daemon".to_owned(),
        "--daemon-max-ticks".to_owned(),
        "100".to_owned(),
        "--daemon-tick-interval-ms".to_owned(),
        "1".to_owned(),
        "--daemon-shutdown-signal-tick".to_owned(),
        "7".to_owned(),
        "--daemon-shutdown-drain-ticks".to_owned(),
        "4".to_owned(),
        "--daemon-shutdown-timeout-ticks".to_owned(),
        "2".to_owned(),
    ])
    .expect("daemon timeout args should parse");
    let report = execute(parsed).expect("daemon timeout execution should succeed");
    let snapshot =
        build_runtime_observability_snapshot(&report).expect("timeout report should map snapshot");

    let readiness = render_observability_endpoint_response(&snapshot, "/readyz");
    assert_eq!(readiness.status_code, 200);
    assert_eq!(readiness.content_type, "application/json");
    assert!(readiness.body.contains("\"ready\":false"));
    assert!(readiness.body.contains("\"health\":\"critical\""));
    assert!(readiness
        .body
        .contains("\"reason_code\":\"daemon_shutdown_timeout\""));
    assert!(readiness
        .body
        .contains("\"readiness_reason_code\":\"readiness_commit_dependency_unhealthy\""));
    assert!(readiness
        .body
        .contains("\"transport_dependency_status\":\"ready\""));
    assert!(readiness
        .body
        .contains("\"signer_dependency_status\":\"ready\""));
    assert!(readiness
        .body
        .contains("\"commit_dependency_status\":\"degraded\""));
}

#[test]
fn functional_observability_endpoint_readiness_reason_taxonomy_covers_dependency_probe_matrix() {
    let mut transport_degraded = sample_observability_snapshot();
    transport_degraded.health = "critical".to_owned();
    transport_degraded.reason_code = "transport_finality_retry_unavailable".to_owned();
    transport_degraded.transport_checkpoint_failures = 2;
    let transport_readiness =
        render_observability_endpoint_response(&transport_degraded, "/readyz");
    assert!(transport_readiness.body.contains("\"ready\":false"));
    assert!(transport_readiness
        .body
        .contains("\"readiness_reason_code\":\"readiness_transport_dependency_unhealthy\""));
    assert!(transport_readiness
        .body
        .contains("\"transport_dependency_status\":\"degraded\""));
    assert!(transport_readiness
        .body
        .contains("\"signer_dependency_status\":\"ready\""));
    assert!(transport_readiness
        .body
        .contains("\"commit_dependency_status\":\"ready\""));

    let mut signer_degraded = sample_observability_snapshot();
    signer_degraded.health = "critical".to_owned();
    signer_degraded.reason_code = "signer_rotation_stale".to_owned();
    signer_degraded.signer_checkpoint_failures = 1;
    let signer_readiness = render_observability_endpoint_response(&signer_degraded, "/readyz");
    assert!(signer_readiness
        .body
        .contains("\"readiness_reason_code\":\"readiness_signer_dependency_unhealthy\""));
    assert!(signer_readiness
        .body
        .contains("\"transport_dependency_status\":\"ready\""));
    assert!(signer_readiness
        .body
        .contains("\"signer_dependency_status\":\"degraded\""));
    assert!(signer_readiness
        .body
        .contains("\"commit_dependency_status\":\"ready\""));

    let mut commit_degraded = sample_observability_snapshot();
    commit_degraded.health = "critical".to_owned();
    commit_degraded.reason_code = "daemon_shutdown_timeout".to_owned();
    commit_degraded.commit_checkpoint_failures = 1;
    let commit_readiness = render_observability_endpoint_response(&commit_degraded, "/readyz");
    assert!(commit_readiness
        .body
        .contains("\"readiness_reason_code\":\"readiness_commit_dependency_unhealthy\""));
    assert!(commit_readiness
        .body
        .contains("\"transport_dependency_status\":\"ready\""));
    assert!(commit_readiness
        .body
        .contains("\"signer_dependency_status\":\"ready\""));
    assert!(commit_readiness
        .body
        .contains("\"commit_dependency_status\":\"degraded\""));

    let mut runtime_health_degraded = sample_observability_snapshot();
    runtime_health_degraded.health = "degraded".to_owned();
    runtime_health_degraded.reason_code = "daemon_slo_alert".to_owned();
    let runtime_health_readiness =
        render_observability_endpoint_response(&runtime_health_degraded, "/readyz");
    assert!(runtime_health_readiness
        .body
        .contains("\"readiness_reason_code\":\"readiness_runtime_health_degraded\""));
    assert!(runtime_health_readiness
        .body
        .contains("\"transport_dependency_status\":\"ready\""));
    assert!(runtime_health_readiness
        .body
        .contains("\"signer_dependency_status\":\"ready\""));
    assert!(runtime_health_readiness
        .body
        .contains("\"commit_dependency_status\":\"ready\""));
}

#[test]
fn functional_observability_endpoint_projects_readiness_reason_code_parity_across_endpoint_surfaces(
) {
    let assert_projection_parity = |snapshot: &RuntimeObservabilitySnapshot,
                                    expected_reason_code: &str| {
        let metrics = render_observability_endpoint_response(snapshot, "/metrics");
        let health = render_observability_endpoint_response(snapshot, "/healthz");
        let readiness = render_observability_endpoint_response(snapshot, "/readyz");
        let stream = render_observability_endpoint_response(snapshot, "/metrics.stream");

        let metrics_marker = format!(
            "kamn_observability_readiness_reason_code{{readiness_reason_code=\"{}\"}} 1",
            expected_reason_code
        );
        let json_marker = format!("\"readiness_reason_code\":\"{}\"", expected_reason_code);

        assert!(metrics.body.contains(metrics_marker.as_str()));
        assert!(health.body.contains(json_marker.as_str()));
        assert!(readiness.body.contains(json_marker.as_str()));
        assert!(stream.body.contains(json_marker.as_str()));
    };

    let healthy = sample_observability_snapshot();
    assert_projection_parity(&healthy, "none");

    let mut transport_degraded = sample_observability_snapshot();
    transport_degraded.health = "critical".to_owned();
    transport_degraded.reason_code = "transport_finality_retry_unavailable".to_owned();
    transport_degraded.transport_checkpoint_failures = 1;
    assert_projection_parity(
        &transport_degraded,
        "readiness_transport_dependency_unhealthy",
    );

    let mut signer_degraded = sample_observability_snapshot();
    signer_degraded.health = "critical".to_owned();
    signer_degraded.reason_code = "signer_rotation_stale".to_owned();
    signer_degraded.signer_checkpoint_failures = 1;
    assert_projection_parity(&signer_degraded, "readiness_signer_dependency_unhealthy");

    let mut commit_degraded = sample_observability_snapshot();
    commit_degraded.health = "critical".to_owned();
    commit_degraded.reason_code = "daemon_shutdown_timeout".to_owned();
    commit_degraded.commit_checkpoint_failures = 1;
    assert_projection_parity(&commit_degraded, "readiness_commit_dependency_unhealthy");

    let mut runtime_health_degraded = sample_observability_snapshot();
    runtime_health_degraded.health = "degraded".to_owned();
    runtime_health_degraded.reason_code = "daemon_slo_alert".to_owned();
    assert_projection_parity(
        &runtime_health_degraded,
        "readiness_runtime_health_degraded",
    );
}

#[test]
fn integration_runtime_observability_endpoint_serves_metrics_and_health_paths() {
    let parsed = parse_args(vec![
        "kamn-node".to_owned(),
        "--role".to_owned(),
        "processor".to_owned(),
        "--runtime-mode".to_owned(),
        "daemon".to_owned(),
        "--daemon-max-ticks".to_owned(),
        "3".to_owned(),
        "--daemon-tick-interval-ms".to_owned(),
        "25".to_owned(),
    ])
    .expect("daemon args should parse");
    let report = execute(parsed).expect("daemon execution should succeed");
    let snapshot =
        build_runtime_observability_snapshot(&report).expect("daemon report should map snapshot");
    let bind_addr = reserve_loopback_addr();
    let endpoint_config = ObservabilityEndpointConfig {
        bind_addr: bind_addr.clone(),
        metrics_path: "/metrics".to_owned(),
        health_path: "/healthz".to_owned(),
        max_requests: 4,
        idle_timeout_ms: 2_000,
    };

    let server_snapshot = snapshot.clone();
    let server =
        thread::spawn(move || serve_observability_endpoint(&endpoint_config, &server_snapshot));
    wait_for_endpoint_ready(bind_addr.as_str());

    let metrics_response = send_http_get(bind_addr.as_str(), "/metrics");
    let health_response = send_http_get(bind_addr.as_str(), "/healthz");
    let readiness_response = send_http_get(bind_addr.as_str(), "/readyz");

    assert!(
        metrics_response.contains("HTTP/1.1 200 OK"),
        "metrics endpoint should return 200 response"
    );
    assert!(metrics_response.contains("kamn_observability_latency_p50_ms 25"));
    assert!(metrics_response.contains("kamn_observability_reason_code{reason_code=\"none\"} 1"));
    assert!(metrics_response
        .contains("kamn_observability_readiness_reason_code{readiness_reason_code=\"none\"} 1"));
    assert!(metrics_response.contains("kamn_observability_ready 1"));
    assert!(
        health_response.contains("HTTP/1.1 200 OK"),
        "health endpoint should return 200 response"
    );
    assert!(health_response.contains("\"health\":\"healthy\""));
    assert!(health_response.contains("\"reason_code\":\"none\""));
    assert!(health_response.contains("\"ready\":true"));
    assert!(health_response.contains("\"readiness_reason_code\":\"none\""));
    assert!(readiness_response.contains("HTTP/1.1 200 OK"));
    assert!(readiness_response.contains("\"ready\":true"));
    assert!(readiness_response.contains("\"readiness_reason_code\":\"none\""));

    let server_result = server.join().expect("endpoint thread should complete");
    assert!(
        server_result.is_ok(),
        "endpoint server should stop cleanly after configured request budget"
    );
}

#[test]
fn integration_runtime_observability_endpoint_serves_stream_path() {
    let parsed = parse_args(vec![
        "kamn-node".to_owned(),
        "--role".to_owned(),
        "processor".to_owned(),
        "--runtime-mode".to_owned(),
        "daemon".to_owned(),
        "--daemon-max-ticks".to_owned(),
        "3".to_owned(),
        "--daemon-tick-interval-ms".to_owned(),
        "25".to_owned(),
    ])
    .expect("daemon args should parse");
    let report = execute(parsed).expect("daemon execution should succeed");
    let snapshot =
        build_runtime_observability_snapshot(&report).expect("daemon report should map snapshot");
    let bind_addr = reserve_loopback_addr();
    let endpoint_config = ObservabilityEndpointConfig {
        bind_addr: bind_addr.clone(),
        metrics_path: "/metrics".to_owned(),
        health_path: "/healthz".to_owned(),
        max_requests: 2,
        idle_timeout_ms: 2_000,
    };

    let server_snapshot = snapshot.clone();
    let server =
        thread::spawn(move || serve_observability_endpoint(&endpoint_config, &server_snapshot));
    wait_for_endpoint_ready(bind_addr.as_str());

    let stream_response = send_http_get(bind_addr.as_str(), "/metrics.stream");
    assert!(
        stream_response.contains("HTTP/1.1 200 OK"),
        "stream endpoint should return 200 response"
    );
    assert!(stream_response.contains("application/x-ndjson"));
    assert!(stream_response.contains("kamn.runtime.observability.stream.v1"));
    assert!(stream_response.contains("\"reason_code\":\"none\""));
    assert!(stream_response.contains("\"ready\":true"));
    assert!(stream_response.contains("\"readiness_reason_code\":\"none\""));

    let server_result = server.join().expect("endpoint thread should complete");
    assert!(
        server_result.is_ok(),
        "endpoint server should stop cleanly after configured request budget"
    );
}

#[test]
fn integration_runtime_observability_endpoint_handles_concurrent_metrics_and_stream_requests() {
    let parsed = parse_args(vec![
        "kamn-node".to_owned(),
        "--role".to_owned(),
        "processor".to_owned(),
        "--runtime-mode".to_owned(),
        "daemon".to_owned(),
        "--daemon-max-ticks".to_owned(),
        "3".to_owned(),
        "--daemon-tick-interval-ms".to_owned(),
        "25".to_owned(),
    ])
    .expect("daemon args should parse");
    let report = execute(parsed).expect("daemon execution should succeed");
    let snapshot =
        build_runtime_observability_snapshot(&report).expect("daemon report should map snapshot");
    let bind_addr = reserve_loopback_addr();
    let endpoint_config = ObservabilityEndpointConfig {
        bind_addr: bind_addr.clone(),
        metrics_path: "/metrics".to_owned(),
        health_path: "/healthz".to_owned(),
        max_requests: 5,
        idle_timeout_ms: 2_000,
    };

    let server_snapshot = snapshot.clone();
    let server =
        thread::spawn(move || serve_observability_endpoint(&endpoint_config, &server_snapshot));
    wait_for_endpoint_ready(bind_addr.as_str());

    let barrier = Arc::new(Barrier::new(3));
    let metrics_barrier = Arc::clone(&barrier);
    let metrics_addr = bind_addr.clone();
    let metrics_worker = thread::spawn(move || {
        metrics_barrier.wait();
        send_http_get(metrics_addr.as_str(), "/metrics")
    });

    let stream_barrier = Arc::clone(&barrier);
    let stream_addr = bind_addr.clone();
    let stream_worker = thread::spawn(move || {
        stream_barrier.wait();
        send_http_get(stream_addr.as_str(), "/metrics.stream")
    });

    barrier.wait();
    let metrics_response = metrics_worker
        .join()
        .expect("metrics worker thread should complete");
    let stream_response = stream_worker
        .join()
        .expect("stream worker thread should complete");
    assert!(metrics_response.contains("HTTP/1.1 200 OK"));
    assert!(metrics_response.contains("kamn_observability_ready 1"));
    assert!(stream_response.contains("HTTP/1.1 200 OK"));
    assert!(stream_response.contains("kamn.runtime.observability.stream.v1"));

    let health_response = send_http_get(bind_addr.as_str(), "/healthz");
    let readiness_response = send_http_get(bind_addr.as_str(), "/readyz");
    assert!(health_response.contains("HTTP/1.1 200 OK"));
    assert!(readiness_response.contains("HTTP/1.1 200 OK"));
    assert!(readiness_response.contains("\"ready\":true"));

    let server_result = server.join().expect("endpoint thread should complete");
    assert!(
        server_result.is_ok(),
        "endpoint server should stop cleanly after configured request budget"
    );
}

#[test]
fn integration_runtime_observability_endpoint_supports_stream_reconnect_churn_sequence() {
    let snapshot = sample_observability_snapshot();
    let bind_addr = reserve_loopback_addr();
    let endpoint_config = ObservabilityEndpointConfig {
        bind_addr: bind_addr.clone(),
        metrics_path: "/metrics".to_owned(),
        health_path: "/healthz".to_owned(),
        max_requests: 4,
        idle_timeout_ms: 2_000,
    };

    let server_snapshot = snapshot.clone();
    let server =
        thread::spawn(move || serve_observability_endpoint(&endpoint_config, &server_snapshot));
    wait_for_endpoint_ready(bind_addr.as_str());

    let first_stream_response = send_http_get(bind_addr.as_str(), "/metrics.stream");
    let second_stream_response = send_http_get(bind_addr.as_str(), "/metrics.stream");
    let readiness_response = send_http_get(bind_addr.as_str(), "/readyz");
    assert!(first_stream_response.contains("HTTP/1.1 200 OK"));
    assert!(first_stream_response.contains("kamn.runtime.observability.stream.v1"));
    assert!(second_stream_response.contains("HTTP/1.1 200 OK"));
    assert!(second_stream_response.contains("kamn.runtime.observability.stream.v1"));
    assert!(readiness_response.contains("HTTP/1.1 200 OK"));
    assert!(readiness_response.contains("\"readiness_reason_code\":\"none\""));

    let server_result = server.join().expect("endpoint thread should complete");
    assert!(
        server_result.is_ok(),
        "endpoint server should stop cleanly after configured request budget"
    );
}

#[test]
fn integration_runtime_observability_endpoint_enforces_queue_bound_request_budget() {
    let snapshot = sample_observability_snapshot();
    let bind_addr = reserve_loopback_addr();
    let endpoint_config = ObservabilityEndpointConfig {
        bind_addr: bind_addr.clone(),
        metrics_path: "/metrics".to_owned(),
        health_path: "/healthz".to_owned(),
        max_requests: 2,
        idle_timeout_ms: 2_000,
    };

    let server_snapshot = snapshot.clone();
    let server =
        thread::spawn(move || serve_observability_endpoint(&endpoint_config, &server_snapshot));
    wait_for_endpoint_ready(bind_addr.as_str());

    let first_request_response = send_http_get(bind_addr.as_str(), "/metrics");
    assert!(first_request_response.contains("HTTP/1.1 200 OK"));
    assert!(first_request_response.contains("kamn_observability_ready 1"));

    let server_result = server.join().expect("endpoint thread should complete");
    assert!(
        server_result.is_ok(),
        "endpoint server should stop cleanly once bounded request budget is exhausted"
    );

    let second_request_result = try_send_http_get(bind_addr.as_str(), "/metrics");
    assert!(
        second_request_result.is_err(),
        "request budget exhaustion should close listener and reject additional requests"
    );
}

#[test]
fn integration_runtime_observability_endpoint_returns_not_found_for_unknown_path() {
    let snapshot = sample_observability_snapshot();
    let bind_addr = reserve_loopback_addr();
    let endpoint_config = ObservabilityEndpointConfig {
        bind_addr: bind_addr.clone(),
        metrics_path: "/metrics".to_owned(),
        health_path: "/healthz".to_owned(),
        max_requests: 2,
        idle_timeout_ms: 2_000,
    };

    let server_snapshot = snapshot.clone();
    let server =
        thread::spawn(move || serve_observability_endpoint(&endpoint_config, &server_snapshot));
    wait_for_endpoint_ready(bind_addr.as_str());

    let unknown_response = send_http_get(bind_addr.as_str(), "/unknown");
    assert!(unknown_response.contains("HTTP/1.1 404 Not Found"));
    assert!(unknown_response.contains("not found"));

    let server_result = server.join().expect("endpoint thread should complete");
    assert!(
        server_result.is_ok(),
        "endpoint server should stop cleanly after configured request budget"
    );
}

#[test]
fn integration_runtime_observability_endpoint_returns_not_found_for_malformed_request_method() {
    let snapshot = sample_observability_snapshot();
    let bind_addr = reserve_loopback_addr();
    let endpoint_config = ObservabilityEndpointConfig {
        bind_addr: bind_addr.clone(),
        metrics_path: "/metrics".to_owned(),
        health_path: "/healthz".to_owned(),
        max_requests: 2,
        idle_timeout_ms: 2_000,
    };

    let server_snapshot = snapshot.clone();
    let server =
        thread::spawn(move || serve_observability_endpoint(&endpoint_config, &server_snapshot));
    wait_for_endpoint_ready(bind_addr.as_str());

    let malformed_response = send_raw_http_request(
        bind_addr.as_str(),
        "POST /metrics HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n",
    );
    assert!(malformed_response.contains("HTTP/1.1 404 Not Found"));
    assert!(malformed_response.contains("not found"));

    let server_result = server.join().expect("endpoint thread should complete");
    assert!(
        server_result.is_ok(),
        "endpoint server should stop cleanly after configured request budget"
    );
}

#[test]
fn integration_runtime_observability_endpoint_fails_closed_on_idle_timeout() {
    let snapshot = sample_observability_snapshot();
    let bind_addr = reserve_loopback_addr();
    let timeout_ms = 200_u64;
    let endpoint_config = ObservabilityEndpointConfig {
        bind_addr,
        metrics_path: "/metrics".to_owned(),
        health_path: "/healthz".to_owned(),
        max_requests: 1,
        idle_timeout_ms: timeout_ms,
    };

    let server = thread::spawn(move || serve_observability_endpoint(&endpoint_config, &snapshot));
    let server_result = server.join().expect("endpoint thread should complete");
    let error = server_result.expect_err("idle timeout should fail closed when no requests arrive");
    assert_eq!(
        error,
        "observability endpoint timed out after 200 ms waiting for requests"
    );
}

#[test]
fn regression_observability_endpoint_export_keeps_bootstrap_report_rendering_unchanged() {
    // Regression: #2830
    let parsed = parse_args(vec![
        "kamn-node".to_owned(),
        "--role".to_owned(),
        "processor".to_owned(),
        "--runtime-mode".to_owned(),
        "daemon".to_owned(),
        "--daemon-max-ticks".to_owned(),
        "3".to_owned(),
        "--daemon-tick-interval-ms".to_owned(),
        "25".to_owned(),
    ])
    .expect("daemon args should parse");
    let report = execute(parsed).expect("daemon execution should succeed");
    let before = render_bootstrap_report(&report, OutputMode::json());

    let snapshot =
        build_runtime_observability_snapshot(&report).expect("daemon report should map snapshot");
    let _metrics = render_observability_endpoint_response(&snapshot, "/metrics");
    let _health = render_observability_endpoint_response(&snapshot, "/healthz");

    let after = render_bootstrap_report(&report, OutputMode::json());
    assert_eq!(before, after);
}

#[test]
fn regression_observability_endpoint_uses_async_listener_serving_path() {
    // Regression: #3511
    let source = include_str!("../observability_endpoint.rs");
    assert!(
        source.contains("tokio::net::TcpListener::bind("),
        "observability endpoint must use tokio listener serving path"
    );
    assert!(
        source.contains("async fn serve_observability_endpoint_async("),
        "observability endpoint should expose async serving function"
    );
    assert!(
        source.contains("runtime.block_on(serve_observability_endpoint_async("),
        "sync wrapper must drive async observability serving via runtime block_on"
    );
}

#[test]
fn regression_observability_endpoint_uses_async_metrics_health_stream_adapters() {
    // Regression: #3512
    let source = include_str!("../observability_endpoint.rs");
    assert!(
        source.contains("async fn dispatch_observability_endpoint_request("),
        "observability endpoint should dispatch requests through async adapter routing"
    );
    assert!(
        source.contains("async fn handle_observability_metrics_path("),
        "observability endpoint should expose async metrics handler adapter"
    );
    assert!(
        source.contains("async fn handle_observability_health_path("),
        "observability endpoint should expose async health handler adapter"
    );
    assert!(
        source.contains("async fn handle_observability_stream_path("),
        "observability endpoint should expose async stream handler adapter"
    );
}

#[test]
fn regression_observability_endpoint_keeps_async_negative_matrix_contracts() {
    // Regression: #3514
    let source = include_str!("../observability_endpoint.rs");
    assert!(
        source.contains("handle_observability_not_found_path().await"),
        "async dispatch must route unsupported paths through deterministic not-found handler"
    );
    assert!(
        source.contains("if method != \"GET\""),
        "request parser must fail closed on malformed method contracts"
    );
    assert!(
        source.contains("observability endpoint timed out after {} ms waiting for requests"),
        "async serving loop must preserve deterministic idle-timeout failure contract"
    );
}
