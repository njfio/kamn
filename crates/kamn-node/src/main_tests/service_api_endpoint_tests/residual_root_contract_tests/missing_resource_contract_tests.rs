use super::*;

#[test]
fn regression_service_api_endpoint_rejects_unknown_task_and_escrow_resource_transitions() {
    let _env = acquire_service_api_test_env();
    let parsed = parse_args(vec![
        "kamn-node".to_owned(),
        "--role".to_owned(),
        "processor".to_owned(),
        "--runtime-mode".to_owned(),
        "api".to_owned(),
        "--api-bind".to_owned(),
        "127.0.0.1:34112".to_owned(),
    ])
    .expect("api args should parse");
    let report = execute(parsed).expect("api execution should succeed");
    let snapshot = build_service_api_snapshot(&report);
    let state_hash = format!(
        "service-api:{}:{}",
        snapshot.chain_id.as_str(),
        snapshot.chain_version.as_str()
    );
    let caller_did = "kamn:did:agent:test-client-missing-resource";

    let bind_addr = reserve_loopback_addr();
    let endpoint_config = ServiceApiEndpointConfig {
        bind_addr: bind_addr.clone(),
        max_requests: 3,
        idle_timeout_ms: 2_000,
        body_limit_bytes: DEFAULT_SERVICE_API_BODY_LIMIT_BYTES,
        concurrency_limit: DEFAULT_SERVICE_API_CONCURRENCY_LIMIT,
        rate_limit_per_second: DEFAULT_SERVICE_API_RATE_LIMIT_PER_SECOND,
    };
    let server_snapshot = snapshot.clone();
    let server = thread::spawn(move || serve_service_api_endpoint(&endpoint_config, &server_snapshot));
    wait_for_endpoint_ready(bind_addr.as_str());

    assert_missing_task_and_escrow_paths(bind_addr.as_str(), caller_did, state_hash.as_str());

    let server_result = server.join().expect("endpoint thread should complete");
    assert!(
        server_result.is_ok(),
        "service api endpoint should stop cleanly after missing-resource regression flow"
    );
}

fn assert_missing_task_and_escrow_paths(bind_addr: &str, caller_did: &str, state_hash: &str) {
    assert_missing_route(
        bind_addr,
        caller_did,
        71,
        "POST",
        "/v1/tasks/task-missing-71/accept",
        state_hash,
    );
    assert_missing_route(
        bind_addr,
        caller_did,
        72,
        "GET",
        "/v1/tasks/task-missing-71",
        state_hash,
    );
    assert_missing_route(
        bind_addr,
        caller_did,
        73,
        "POST",
        "/v1/escrow/escrow-missing-71/release",
        state_hash,
    );
}

fn assert_missing_route(
    bind_addr: &str,
    caller_did: &str,
    nonce: u64,
    method: &str,
    path: &str,
    state_hash: &str,
) {
    let signature = service_api_request_signature_for_fields(caller_did, nonce, state_hash, "");
    let response = send_http_request_with_headers(
        bind_addr,
        method,
        path,
        "",
        &[
            ("X-KAMN-Sender-DID", caller_did),
            ("X-KAMN-Request-Nonce", nonce.to_string().as_str()),
            ("X-KAMN-Request-Signature", signature.as_str()),
        ],
    );
    assert!(response.contains("HTTP/1.1 404 Not Found"));
    let payload = parse_error_envelope(extract_http_response_body(response.as_str()));
    assert_eq!(payload.error, "not-found");
    assert_eq!(payload.reason_code, "service_api_route_not_found");
}
