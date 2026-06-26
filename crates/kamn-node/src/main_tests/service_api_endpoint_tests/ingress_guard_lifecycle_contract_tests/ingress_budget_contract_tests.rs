use super::super::*;
use super::support::{
    assert_server_ok, build_ingress_snapshot, send_signed_message_request, spawn_ingress_server,
};

#[test]
fn functional_service_api_endpoint_rejects_when_rate_limit_is_exceeded() {
    let _env = acquire_service_api_test_env();
    let snapshot = build_ingress_snapshot("127.0.0.1:34062");
    let server = spawn_ingress_server(
        &snapshot,
        2,
        2_000,
        DEFAULT_SERVICE_API_BODY_LIMIT_BYTES,
        DEFAULT_SERVICE_API_CONCURRENCY_LIMIT,
        1,
    );
    let sender_did = "kamn:did:agent:test-client-rate-limit";
    let message_body = "{\"message\":\"rate-limit-check\"}";

    let first_response = send_signed_message_request(
        &snapshot,
        server.bind_addr.as_str(),
        sender_did,
        101,
        message_body,
    );
    let second_response = send_signed_message_request(
        &snapshot,
        server.bind_addr.as_str(),
        sender_did,
        102,
        message_body,
    );

    assert!(first_response.contains("HTTP/1.1 202 Accepted"));
    assert!(second_response.contains("HTTP/1.1 429 Too Many Requests"));
    let payload = parse_error_envelope_from_http_response(second_response.as_str());
    assert_eq!(payload.error, "too-many-requests");
    assert_eq!(
        payload.reason_code,
        "service_api_ingress_rate_limit_exceeded"
    );
    assert!(payload.message.contains("ingress rate limit exceeded"));
    assert_server_ok(
        server.server,
        "service api endpoint should stop cleanly after configured request budget",
    );
}

#[test]
fn regression_service_api_endpoint_oversized_payload_maps_body_limit_reason_code() {
    let _env = acquire_service_api_test_env();
    let snapshot = build_ingress_snapshot("127.0.0.1:34064");
    let server = spawn_ingress_server(
        &snapshot,
        1,
        2_000,
        256,
        DEFAULT_SERVICE_API_CONCURRENCY_LIMIT,
        1_000,
    );
    let body = format!("{{\"message\":\"{}\"}}", "x".repeat(700));
    let response = send_signed_message_request(
        &snapshot,
        server.bind_addr.as_str(),
        "kamn:did:agent:test-client-oversized",
        303,
        body.as_str(),
    );

    assert!(response.contains("HTTP/1.1 400 Bad Request"));
    let payload = parse_error_envelope_from_http_response(response.as_str());
    assert_eq!(payload.error, "bad-request");
    assert_eq!(
        payload.reason_code,
        "service_api_ingress_body_size_limit_exceeded"
    );
    assert!(payload.message.contains("request body size limit exceeded"));
    assert_server_ok(
        server.server,
        "service api endpoint should stop cleanly after configured request budget",
    );
}

#[test]
fn regression_service_api_endpoint_unauthorized_ingress_consumes_request_budget() {
    let _env = acquire_service_api_test_env();
    let snapshot = build_ingress_snapshot("127.0.0.1:34069");
    let server = spawn_ingress_server(
        &snapshot,
        1,
        80,
        DEFAULT_SERVICE_API_BODY_LIMIT_BYTES,
        DEFAULT_SERVICE_API_CONCURRENCY_LIMIT,
        DEFAULT_SERVICE_API_RATE_LIMIT_PER_SECOND,
    );
    let response = send_http_request(
        server.bind_addr.as_str(),
        "POST",
        "/v1/messages/send",
        "{\"message\":\"unsigned\"}",
    );

    assert!(response.contains("HTTP/1.1 401 Unauthorized"));
    let payload = parse_error_envelope_from_http_response(response.as_str());
    assert_eq!(
        payload.reason_code,
        SERVICE_API_AUTH_MISSING_HEADER_REASON_CODE
    );
    assert_server_ok(
        server.server,
        "unauthorized ingress must still consume request budget for graceful shutdown",
    );
}

#[test]
fn regression_service_api_endpoint_returns_timeout_error_when_no_requests_arrive() {
    let _env = acquire_service_api_test_env();
    let snapshot = build_ingress_snapshot("127.0.0.1:34070");
    let endpoint_config = ServiceApiEndpointConfig {
        bind_addr: reserve_loopback_addr(),
        max_requests: 1,
        idle_timeout_ms: 40,
        body_limit_bytes: DEFAULT_SERVICE_API_BODY_LIMIT_BYTES,
        concurrency_limit: DEFAULT_SERVICE_API_CONCURRENCY_LIMIT,
        rate_limit_per_second: DEFAULT_SERVICE_API_RATE_LIMIT_PER_SECOND,
    };

    let started = Instant::now();
    let result = serve_service_api_endpoint(&endpoint_config, &snapshot);
    assert!(result.is_err());
    let error = result.expect_err("timeout error should be returned");
    assert!(error.contains("service api timed out after 40 ms waiting for requests"));
    assert!(started.elapsed() <= Duration::from_secs(1));
}
