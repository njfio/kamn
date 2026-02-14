use super::*;
use std::io::{ErrorKind, Read, Write};
use std::net::{TcpListener, TcpStream};
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
        .contains("kamn_observability_health{health=\"healthy\"} 1"));

    let health = render_observability_endpoint_response(&snapshot, "/healthz");
    assert_eq!(health.status_code, 200);
    assert_eq!(health.content_type, "application/json");
    assert!(health.body.contains("\"health\":\"healthy\""));
    assert!(health.body.contains("\"runtime_mode\":\"daemon\""));
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
    assert!(stream.body.ends_with('\n'));
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
        max_requests: 3,
        idle_timeout_ms: 2_000,
    };

    let server_snapshot = snapshot.clone();
    let server =
        thread::spawn(move || serve_observability_endpoint(&endpoint_config, &server_snapshot));
    wait_for_endpoint_ready(bind_addr.as_str());

    let metrics_response = send_http_get(bind_addr.as_str(), "/metrics");
    let health_response = send_http_get(bind_addr.as_str(), "/healthz");

    assert!(
        metrics_response.contains("HTTP/1.1 200 OK"),
        "metrics endpoint should return 200 response"
    );
    assert!(metrics_response.contains("kamn_observability_latency_p50_ms 25"));
    assert!(
        health_response.contains("HTTP/1.1 200 OK"),
        "health endpoint should return 200 response"
    );
    assert!(health_response.contains("\"health\":\"healthy\""));

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

    let server_result = server.join().expect("endpoint thread should complete");
    assert!(
        server_result.is_ok(),
        "endpoint server should stop cleanly after configured request budget"
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
