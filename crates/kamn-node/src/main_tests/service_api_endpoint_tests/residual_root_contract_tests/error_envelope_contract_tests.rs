use super::*;
use crate::service_api_endpoint::ServiceApiSnapshot;

#[test]
fn unit_service_api_endpoint_error_envelopes_use_reason_code_and_message_contracts() {
    let snapshot = error_contract_snapshot();
    assert_websocket_upgrade_error(&snapshot);
    assert_method_not_allowed_error(&snapshot);
    assert_not_found_error(&snapshot);

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

fn error_contract_snapshot() -> ServiceApiSnapshot {
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
    build_service_api_snapshot(&report)
}

fn assert_websocket_upgrade_error(snapshot: &ServiceApiSnapshot) {
    let response = render_service_api_endpoint_response(snapshot, "GET", "/v1/events/ws", "");
    assert_eq!(response.status_code, 400);
    let payload = parse_error_envelope(response.body.as_str());
    assert_eq!(payload.error, "bad-request");
    assert_eq!(
        payload.reason_code,
        route_render_contract_tests::websocket_upgrade_required_reason_code()
    );
    assert!(payload.message.contains("websocket upgrade required"));
}

fn assert_method_not_allowed_error(snapshot: &ServiceApiSnapshot) {
    let response =
        render_service_api_endpoint_response(snapshot, "DELETE", "/v1/messages/send", "");
    assert_eq!(response.status_code, 405);
    let payload = parse_error_envelope(response.body.as_str());
    assert_eq!(payload.error, "method-not-allowed");
    assert_eq!(payload.reason_code, "service_api_method_not_allowed");
    assert!(payload.message.contains("method not allowed"));
}

fn assert_not_found_error(snapshot: &ServiceApiSnapshot) {
    let response = render_service_api_endpoint_response(snapshot, "GET", "/v1/nope", "");
    assert_eq!(response.status_code, 404);
    let payload = parse_error_envelope(response.body.as_str());
    assert_eq!(payload.error, "not-found");
    assert_eq!(payload.reason_code, "service_api_route_not_found");
    assert!(payload.message.contains("not found"));
}

fn assert_endpoint_config_rejects_zero_values(
    baseline_config: &ServiceApiEndpointConfig,
    snapshot: &ServiceApiSnapshot,
) {
    assert_endpoint_config_zero_limits(baseline_config, snapshot);
    assert_endpoint_config_zero_timeouts(baseline_config, snapshot);
}

fn assert_endpoint_config_zero_limits(
    baseline_config: &ServiceApiEndpointConfig,
    snapshot: &ServiceApiSnapshot,
) {
    assert_endpoint_config_error(
        baseline_config,
        snapshot,
        |config| config.max_requests = 0,
        "service api max requests must be greater than zero",
        "max_requests=0 must fail closed",
    );
    assert_endpoint_config_error(
        baseline_config,
        snapshot,
        |config| config.concurrency_limit = 0,
        "service api concurrency limit must be greater than zero",
        "concurrency_limit=0 must fail closed",
    );
}

fn assert_endpoint_config_zero_timeouts(
    baseline_config: &ServiceApiEndpointConfig,
    snapshot: &ServiceApiSnapshot,
) {
    assert_endpoint_idle_timeout_zero(baseline_config, snapshot);
    assert_endpoint_body_limit_and_rate_zero(baseline_config, snapshot);
}

fn assert_endpoint_idle_timeout_zero(
    baseline_config: &ServiceApiEndpointConfig,
    snapshot: &ServiceApiSnapshot,
) {
    assert_endpoint_config_error(
        baseline_config,
        snapshot,
        |config| config.idle_timeout_ms = 0,
        "service api idle timeout must be greater than zero",
        "idle_timeout_ms=0 must fail closed",
    );
}

fn assert_endpoint_body_limit_and_rate_zero(
    baseline_config: &ServiceApiEndpointConfig,
    snapshot: &ServiceApiSnapshot,
) {
    assert_endpoint_config_error(
        baseline_config,
        snapshot,
        |config| config.body_limit_bytes = 0,
        "service api body limit bytes must be greater than zero",
        "body_limit_bytes=0 must fail closed",
    );
    assert_endpoint_config_error(
        baseline_config,
        snapshot,
        |config| config.rate_limit_per_second = 0,
        "service api rate limit per second must be greater than zero",
        "rate_limit_per_second=0 must fail closed",
    );
}

fn assert_endpoint_config_error<F>(
    baseline_config: &ServiceApiEndpointConfig,
    snapshot: &ServiceApiSnapshot,
    mutate: F,
    expected_error: &str,
    expected_context: &str,
) where
    F: FnOnce(&mut ServiceApiEndpointConfig),
{
    let mut invalid = baseline_config.clone();
    mutate(&mut invalid);
    let error = serve_service_api_endpoint(&invalid, snapshot).expect_err(expected_context);
    assert_eq!(error, expected_error);
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
