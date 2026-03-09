use super::super::*;
use super::support::{assert_server_ok, build_transport_snapshot, spawn_transport_server};

#[test]
fn regression_service_api_runtime_observability_projects_live_metrics_under_traffic() {
    let _env = acquire_service_api_test_env();
    let snapshot = build_transport_snapshot("127.0.0.1:34057");
    let server = spawn_transport_server(&snapshot, 4);
    let body = "{\"message\":\"runtime-observability\"}";
    let signature = service_api_request_signature_for_fields("kamn:did:agent:runtime-observability", 1, super::support::state_hash(&snapshot).as_str(), body);
    let send = send_http_request_with_headers(server.bind_addr.as_str(), "POST", "/v1/messages/send", body, &[("X-KAMN-Sender-DID", "kamn:did:agent:runtime-observability"), ("X-KAMN-Request-Nonce", "1"), ("X-KAMN-Request-Signature", signature.as_str()), ("X-KAMN-Authz-Scope", "messages:write")]);
    let unauth = send_http_request(server.bind_addr.as_str(), "POST", "/v1/messages/send", "{\"message\":\"unauth\"}");
    let health = send_http_request(server.bind_addr.as_str(), "GET", "/healthz", "");
    let metrics = send_http_request(server.bind_addr.as_str(), "GET", "/metrics", "");

    assert!(send.contains("HTTP/1.1 202 Accepted"));
    assert!(unauth.contains("HTTP/1.1 401 Unauthorized"));
    assert!(health.contains("HTTP/1.1 200 OK"));
    assert!(metrics.contains("HTTP/1.1 200 OK"));
    let health_payload: ServiceApiHealthBody = parse_service_api_payload(extract_http_response_body(health.as_str())).expect("health payload should deserialize");
    assert_eq!(health_payload.observability_source, "service-api-runtime");
    assert!(matches!(health_payload.observability_health.as_str(), "healthy" | "degraded" | "critical"));
    assert!(parse_scalar_metric_value(metrics.as_str(), "kamn_service_api_observability_throughput_tps").expect("throughput metric should be present") > 0);
    assert!(parse_scalar_metric_value(metrics.as_str(), "kamn_service_api_observability_error_rate_bps").expect("error rate metric should be present") > 0);
    assert!(parse_scalar_metric_value(metrics.as_str(), "kamn_service_api_observability_latency_p50_ms").expect("latency p50 metric should be present") > 0);
    assert!(parse_scalar_metric_value(metrics.as_str(), "kamn_service_api_observability_latency_p99_ms").expect("latency p99 metric should be present") > 0);
    assert!(parse_scalar_metric_value(metrics.as_str(), "kamn_service_api_observability_availability_bps").expect("availability metric should be present") < 10_000);
    assert_server_ok(server.server, "service api endpoint should stop cleanly after configured request budget");
}

#[test]
fn functional_service_api_endpoint_emits_structured_ingress_correlation_markers() {
    let _env = acquire_service_api_test_env();
    let _level_guard = EnvVarGuard::set("KAMN_NODE_LOG_LEVEL", Some("info"));
    let _format_guard = EnvVarGuard::set("KAMN_NODE_LOG_FORMAT", Some("json"));
    let snapshot = build_transport_snapshot("127.0.0.1:34058");
    let bind_addr = reserve_loopback_addr();
    let endpoint_config = ServiceApiEndpointConfig {
        bind_addr: bind_addr.clone(),
        max_requests: 1,
        idle_timeout_ms: 2_000,
        body_limit_bytes: DEFAULT_SERVICE_API_BODY_LIMIT_BYTES,
        concurrency_limit: DEFAULT_SERVICE_API_CONCURRENCY_LIMIT,
        rate_limit_per_second: DEFAULT_SERVICE_API_RATE_LIMIT_PER_SECOND,
    };
    let client = spawn_correlation_client(bind_addr.clone(), super::support::state_hash(&snapshot).as_str());
    let (serve_result, captured_logs) = capture_test_logs(|| serve_service_api_endpoint(&endpoint_config, &snapshot));
    let response = client.join().expect("client request should complete");
    assert!(serve_result.is_ok());
    assert!(response.contains("HTTP/1.1 202 Accepted"));
    assert_correlation_markers(captured_logs.as_slice());
}

#[test]
fn unit_service_api_endpoint_metrics_use_runtime_observability_when_present() {
    let _env = acquire_service_api_test_env();
    let parsed = parse_args(vec!["kamn-node".to_owned(), "--role".to_owned(), "processor".to_owned(), "--runtime-mode".to_owned(), "daemon".to_owned(), "--daemon-max-ticks".to_owned(), "3".to_owned(), "--daemon-tick-interval-ms".to_owned(), "25".to_owned()]).expect("daemon args should parse");
    let report = execute(parsed).expect("daemon execution should succeed");
    let snapshot = build_service_api_snapshot(&report);

    assert_eq!(snapshot.observability_source, "daemon");
    assert_eq!(snapshot.observability_health, "healthy");
    let metrics = route_render_contract_tests::render_metrics_response(&snapshot);
    assert_eq!(metrics.status_code, 200);
    assert!(metrics.body.contains("kamn_service_api_observability_source{source=\"daemon\"} 1"));
    assert!(metrics.body.contains("kamn_service_api_observability_health{health=\"healthy\"} 1"));
    route_render_contract_tests::assert_common_route_metrics(metrics.body.as_str());
}

fn spawn_correlation_client(bind_addr: String, state_hash: &str) -> thread::JoinHandle<String> {
    let signature = service_api_request_signature_for_fields(
        "kamn:did:agent:test-client-correlation",
        41,
        state_hash,
        "{\"message\":\"structured-correlation\"}",
    );
    thread::spawn(move || {
        thread::sleep(Duration::from_millis(50));
        send_http_request_with_headers(
            bind_addr.as_str(),
            "POST",
            "/v1/messages/send",
            "{\"message\":\"structured-correlation\"}",
            &[
                ("X-KAMN-Sender-DID", "kamn:did:agent:test-client-correlation"),
                ("X-KAMN-Request-Nonce", "41"),
                ("X-KAMN-Request-Signature", signature.as_str()),
            ],
        )
    })
}

fn assert_correlation_markers(captured_logs: &[String]) {
    let ingress_line = captured_logs
        .iter()
        .find(|line| line.contains("\"event\":\"service.api.request.received\""))
        .expect("service api ingress should emit received marker");
    let outcome_line = captured_logs
        .iter()
        .find(|line| line.contains("\"event\":\"service.api.request.outcome\""))
        .expect("service api ingress should emit outcome marker");
    assert_eq!(
        extract_json_string_field(ingress_line, "correlation_id"),
        extract_json_string_field(outcome_line, "correlation_id")
    );
    assert_eq!(extract_json_string_field(ingress_line, "method").as_deref(), Some("POST"));
    assert_eq!(extract_json_string_field(ingress_line, "path").as_deref(), Some("/v1/messages/send"));
    assert_eq!(extract_json_string_field(outcome_line, "status_code").as_deref(), Some("202"));
}
