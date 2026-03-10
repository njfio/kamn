use super::support::*;
use super::*;

#[path = "stream_runtime_contract_tests/stream_server_contract_tests.rs"]
mod stream_server_contract_tests;

#[test]
fn functional_observability_endpoint_renders_stream_payload() {
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
    let stream = render_observability_endpoint_response(&snapshot, "/metrics.stream");

    assert_eq!(stream.status_code, 200);
    assert_eq!(stream.content_type, "application/x-ndjson");
    assert!(stream
        .body
        .contains("\"schema_version\":\"kamn.runtime.observability.stream.v1\""));
    assert!(stream.body.ends_with('\n'));
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
    assert!(send_http_get(bind_addr.as_str(), "/metrics.stream").contains("HTTP/1.1 200 OK"));
    assert!(send_http_get(bind_addr.as_str(), "/metrics.stream").contains("HTTP/1.1 200 OK"));
    assert!(send_http_get(bind_addr.as_str(), "/readyz").contains("HTTP/1.1 200 OK"));
    assert!(server
        .join()
        .expect("endpoint thread should complete")
        .is_ok());
}
