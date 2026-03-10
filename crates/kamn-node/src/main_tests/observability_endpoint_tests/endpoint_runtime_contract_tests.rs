use super::support::*;
use super::*;

#[path = "endpoint_runtime_contract_tests/readiness_contract_tests.rs"]
mod readiness_contract_tests;

#[test]
fn functional_observability_endpoint_renders_metrics_and_health_payloads() {
    let parsed = parse_args_with_clean_daemon_env(vec![
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
    assert!(metrics.body.contains("kamn_observability_latency_p50_ms 1"));
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
    assert!(health
        .body
        .contains("\"schema_version\":\"kamn.runtime.observability.health.v1\""));
    assert!(health.body.contains("\"health\":\"healthy\""));
    assert!(health.body.contains("\"runtime_mode\":\"daemon\""));
    assert!(health.body.contains("\"reason_code\":\"none\""));
    assert!(health.body.contains("\"ready\":true"));
    assert!(health.body.contains("\"readiness_reason_code\":\"none\""));
}

#[test]
fn integration_runtime_observability_endpoint_serves_metrics_and_health_paths() {
    let parsed = parse_args_with_clean_daemon_env(vec![
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

    assert!(send_http_get(bind_addr.as_str(), "/metrics").contains("HTTP/1.1 200 OK"));
    assert!(send_http_get(bind_addr.as_str(), "/healthz").contains("HTTP/1.1 200 OK"));
    assert!(send_http_get(bind_addr.as_str(), "/readyz").contains("HTTP/1.1 200 OK"));

    let server_result = server.join().expect("endpoint thread should complete");
    assert!(
        server_result.is_ok(),
        "endpoint server should stop cleanly after configured request budget"
    );
}
