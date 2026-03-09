use super::*;

#[test]
fn unit_service_api_endpoint_error_envelopes_use_reason_code_and_message_contracts() {
    let _env = acquire_service_api_test_env();
    let parsed = parse_args(vec![
        "kamn-node".to_owned(),
        "--role".to_owned(),
        "processor".to_owned(),
        "--runtime-mode".to_owned(),
        "api".to_owned(),
        "--api-bind".to_owned(),
        "127.0.0.1:34061".to_owned(),
    ])
    .expect("api args should parse");
    let report = execute(parsed).expect("api execution should succeed");
    let snapshot = build_service_api_snapshot(&report);

    let websocket_required = render_service_api_endpoint_response(&snapshot, "GET", "/v1/events/ws", "");
    assert_eq!(websocket_required.status_code, 400);
    let websocket_required_payload = parse_error_envelope(websocket_required.body.as_str());
    assert_eq!(websocket_required_payload.error, "bad-request");
    assert_eq!(
        websocket_required_payload.reason_code,
        route_render_contract_tests::websocket_upgrade_required_reason_code()
    );
    assert!(websocket_required_payload.message.contains("websocket upgrade required"));

    let method_not_allowed = render_service_api_endpoint_response(&snapshot, "DELETE", "/v1/messages/send", "");
    assert_eq!(method_not_allowed.status_code, 405);
    let method_not_allowed_payload = parse_error_envelope(method_not_allowed.body.as_str());
    assert_eq!(method_not_allowed_payload.error, "method-not-allowed");
    assert_eq!(method_not_allowed_payload.reason_code, "service_api_method_not_allowed");
    assert!(method_not_allowed_payload.message.contains("method not allowed"));

    let not_found = render_service_api_endpoint_response(&snapshot, "GET", "/v1/nope", "");
    assert_eq!(not_found.status_code, 404);
    let not_found_payload = parse_error_envelope(not_found.body.as_str());
    assert_eq!(not_found_payload.error, "not-found");
    assert_eq!(not_found_payload.reason_code, "service_api_route_not_found");
    assert!(not_found_payload.message.contains("not found"));

    let baseline_config = ServiceApiEndpointConfig {
        bind_addr: "127.0.0.1:0".to_owned(),
        max_requests: 1,
        idle_timeout_ms: 1,
        body_limit_bytes: 1,
        concurrency_limit: 1,
        rate_limit_per_second: 1,
    };

    assert_endpoint_config_rejects_zero_values(&baseline_config, &snapshot);
    assert_daemon_relay_upsert_marks_message_relayed();
}

fn assert_endpoint_config_rejects_zero_values(
    baseline_config: &ServiceApiEndpointConfig,
    snapshot: &crate::service_api_endpoint::ServiceApiSnapshot,
) {
    let mut max_requests_zero = baseline_config.clone();
    max_requests_zero.max_requests = 0;
    let max_requests_error = serve_service_api_endpoint(&max_requests_zero, snapshot)
        .expect_err("max_requests=0 must fail closed");
    assert_eq!(max_requests_error, "service api max requests must be greater than zero");

    let mut idle_timeout_zero = baseline_config.clone();
    idle_timeout_zero.idle_timeout_ms = 0;
    let idle_timeout_error = serve_service_api_endpoint(&idle_timeout_zero, snapshot)
        .expect_err("idle_timeout_ms=0 must fail closed");
    assert_eq!(idle_timeout_error, "service api idle timeout must be greater than zero");

    let mut body_limit_zero = baseline_config.clone();
    body_limit_zero.body_limit_bytes = 0;
    let body_limit_error = serve_service_api_endpoint(&body_limit_zero, snapshot)
        .expect_err("body_limit_bytes=0 must fail closed");
    assert_eq!(body_limit_error, "service api body limit bytes must be greater than zero");

    let mut concurrency_limit_zero = baseline_config.clone();
    concurrency_limit_zero.concurrency_limit = 0;
    let concurrency_limit_error = serve_service_api_endpoint(&concurrency_limit_zero, snapshot)
        .expect_err("concurrency_limit=0 must fail closed");
    assert_eq!(concurrency_limit_error, "service api concurrency limit must be greater than zero");

    let mut rate_limit_zero = baseline_config.clone();
    rate_limit_zero.rate_limit_per_second = 0;
    let rate_limit_error = serve_service_api_endpoint(&rate_limit_zero, snapshot)
        .expect_err("rate_limit_per_second=0 must fail closed");
    assert_eq!(rate_limit_error, "service api rate limit per second must be greater than zero");
}

fn assert_daemon_relay_upsert_marks_message_relayed() {
    let relay_entry = ServiceApiRelaySpoolEntry {
        message_id: "msg-test-relay".to_owned(),
        sender_did: Some("kamn:did:agent:sender".to_owned()),
        recipient_did: "kamn:did:agent:recipient".to_owned(),
        body: "{\"message\":\"relay\"}".to_owned(),
        queued_at_unix: 1,
    };
    let relayed = upsert_service_api_relayed_message_from_daemon(None, &relay_entry)
        .expect("daemon relay upsert should succeed without a state file");
    assert_eq!(relayed.message_id, "msg-test-relay");
    assert_eq!(relayed.status, "relayed");
}
