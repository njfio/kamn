use super::*;

#[test]
fn unit_service_api_endpoint_balance_route_returns_did_and_balance_contract() {
    let _env = acquire_service_api_test_env();
    let parsed = parse_args(vec![
        "kamn-node".to_owned(),
        "--role".to_owned(),
        "processor".to_owned(),
        "--runtime-mode".to_owned(),
        "api".to_owned(),
        "--api-bind".to_owned(),
        "127.0.0.1:34130".to_owned(),
    ])
    .expect("api args should parse");
    let report = execute(parsed).expect("api execution should succeed");
    let snapshot = build_service_api_snapshot(&report);

    let response = render_service_api_endpoint_response(
        &snapshot,
        "GET",
        "/v1/agents/kamn:did:agent:alpha/balance",
        "",
    );
    assert_eq!(response.status_code, 200);
    let payload: Value =
        parse_service_api_payload(response.body.as_str()).expect("payload should deserialize");
    assert_eq!(payload["did"], "kamn:did:agent:alpha");
    assert_eq!(payload["balance"], 100);
}

#[test]
fn integration_service_api_endpoint_balance_route_backfills_legacy_state_and_persists_restart() {
    let _env = acquire_service_api_test_env();
    let state_file = std::env::temp_dir().join(format!(
        "kamn-node-service-api-agent-balance-restart-state-{}-{}.json",
        std::process::id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock should be monotonic")
            .as_nanos()
    ));
    let state_file_str = state_file.to_string_lossy().to_string();
    let _state_file_guard =
        EnvVarGuard::set("KAMN_SERVICE_API_STATE_FILE", Some(state_file_str.as_str()));
    let target_agent_did = "kamn:did:agent:balance-restart-target";
    fs::write(
        state_file.as_path(),
        format!(
            concat!(
                "{{",
                "\"schema_version\":\"kamn.runtime.service-api-message-store.v2\",",
                "\"messages\":{{}},",
                "\"channel_messages\":{{}},",
                "\"auth_nonce_high_watermarks\":{{}},",
                "\"tasks\":{{}},",
                "\"escrows\":{{}},",
                "\"contents\":{{}},",
                "\"bridges\":{{}},",
                "\"agents\":{{",
                "\"{did}\":{{\"did\":\"{did}\",\"reputation_score\":500}}",
                "}}",
                "}}"
            ),
            did = target_agent_did
        ),
    )
    .expect("legacy state file should write");

    let parsed = parse_args(vec![
        "kamn-node".to_owned(),
        "--role".to_owned(),
        "processor".to_owned(),
        "--runtime-mode".to_owned(),
        "api".to_owned(),
        "--api-bind".to_owned(),
        "127.0.0.1:34131".to_owned(),
    ])
    .expect("api args should parse");
    let report = execute(parsed).expect("api execution should succeed");
    let snapshot = build_service_api_snapshot(&report);
    let state_hash = format!(
        "service-api:{}:{}",
        snapshot.chain_id.as_str(),
        snapshot.chain_version.as_str()
    );
    let caller_did = "kamn:did:agent:test-client-agent-balance-restart";
    let query_path = format!("/v1/agents/{target_agent_did}/balance");

    let bind_addr = reserve_loopback_addr();
    let endpoint_config = ServiceApiEndpointConfig {
        bind_addr: bind_addr.clone(),
        max_requests: 1,
        idle_timeout_ms: 2_000,
        body_limit_bytes: DEFAULT_SERVICE_API_BODY_LIMIT_BYTES,
        concurrency_limit: DEFAULT_SERVICE_API_CONCURRENCY_LIMIT,
        rate_limit_per_second: DEFAULT_SERVICE_API_RATE_LIMIT_PER_SECOND,
    };
    let server_snapshot = snapshot.clone();
    let server =
        thread::spawn(move || serve_service_api_endpoint(&endpoint_config, &server_snapshot));
    wait_for_endpoint_ready(bind_addr.as_str());

    let query_signature =
        service_api_request_signature_for_fields(caller_did, 131, state_hash.as_str(), "");
    let query_response = send_http_request_with_headers(
        bind_addr.as_str(),
        "GET",
        query_path.as_str(),
        "",
        &[
            ("X-KAMN-Sender-DID", caller_did),
            ("X-KAMN-Request-Nonce", "131"),
            ("X-KAMN-Request-Signature", query_signature.as_str()),
        ],
    );
    assert!(query_response.contains("HTTP/1.1 200 OK"));
    let first_balance_payload: Value =
        parse_service_api_payload(extract_http_response_body(query_response.as_str()))
            .expect("balance payload should deserialize");
    assert_eq!(first_balance_payload["did"], target_agent_did);
    assert_eq!(first_balance_payload["balance"], 100);

    let server_result = server.join().expect("endpoint thread should complete");
    assert!(
        server_result.is_ok(),
        "service api endpoint should stop cleanly after first agent balance query phase"
    );

    let phase_one_state_payload = fs::read_to_string(state_file.as_path())
        .expect("balance state file should remain readable");
    let phase_one_state_json: Value =
        serde_json::from_str(phase_one_state_payload.as_str()).expect("state payload should parse");
    assert_eq!(
        phase_one_state_json["agents"][target_agent_did]["balance"],
        100
    );

    let restart_report = execute(
        parse_args(vec![
            "kamn-node".to_owned(),
            "--role".to_owned(),
            "processor".to_owned(),
            "--runtime-mode".to_owned(),
            "api".to_owned(),
            "--api-bind".to_owned(),
            "127.0.0.1:34132".to_owned(),
        ])
        .expect("restart api args should parse"),
    )
    .expect("restart api execution should succeed");
    let restart_snapshot = build_service_api_snapshot(&restart_report);
    let restart_state_hash = format!(
        "service-api:{}:{}",
        restart_snapshot.chain_id.as_str(),
        restart_snapshot.chain_version.as_str()
    );
    let restart_bind_addr = reserve_loopback_addr();
    let restart_endpoint_config = ServiceApiEndpointConfig {
        bind_addr: restart_bind_addr.clone(),
        max_requests: 1,
        idle_timeout_ms: 2_000,
        body_limit_bytes: DEFAULT_SERVICE_API_BODY_LIMIT_BYTES,
        concurrency_limit: DEFAULT_SERVICE_API_CONCURRENCY_LIMIT,
        rate_limit_per_second: DEFAULT_SERVICE_API_RATE_LIMIT_PER_SECOND,
    };
    let restart_server_snapshot = restart_snapshot.clone();
    let restart_server = thread::spawn(move || {
        serve_service_api_endpoint(&restart_endpoint_config, &restart_server_snapshot)
    });
    wait_for_endpoint_ready(restart_bind_addr.as_str());

    let restart_query_signature =
        service_api_request_signature_for_fields(caller_did, 132, restart_state_hash.as_str(), "");
    let restart_query_response = send_http_request_with_headers(
        restart_bind_addr.as_str(),
        "GET",
        query_path.as_str(),
        "",
        &[
            ("X-KAMN-Sender-DID", caller_did),
            ("X-KAMN-Request-Nonce", "132"),
            ("X-KAMN-Request-Signature", restart_query_signature.as_str()),
        ],
    );
    assert!(restart_query_response.contains("HTTP/1.1 200 OK"));
    let restart_balance_payload: Value =
        parse_service_api_payload(extract_http_response_body(restart_query_response.as_str()))
            .expect("restart balance payload should deserialize");
    assert_eq!(restart_balance_payload["did"], target_agent_did);
    assert_eq!(restart_balance_payload["balance"], 100);

    let restart_server_result = restart_server
        .join()
        .expect("restart endpoint thread should complete");
    assert!(
        restart_server_result.is_ok(),
        "service api endpoint should stop cleanly after restart balance query"
    );

    let _ = fs::remove_file(state_file);
}
