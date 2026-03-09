use super::super::*;
use super::support::{assert_server_ok, build_transport_snapshot, send_signed_message_request, spawn_transport_server, state_hash};

#[test]
fn integration_service_api_endpoint_serves_required_http_routes() {
    let _env = acquire_service_api_test_env();
    let snapshot = build_transport_snapshot("127.0.0.1:34052");
    let server = spawn_transport_server(&snapshot, 3);
    let send_response = send_signed_message_request(&snapshot, server.bind_addr.as_str(), "kamn:did:agent:test-client-1", 1, "{\"message\":\"hello\"}");
    let health_response = send_http_request(server.bind_addr.as_str(), "GET", "/healthz", "");
    let metrics_response = send_http_request(server.bind_addr.as_str(), "GET", "/metrics", "");

    assert!(send_response.contains("HTTP/1.1 202 Accepted"));
    assert!(send_response.contains("\"message_id\":\"msg-local-"));
    assert!(health_response.contains("HTTP/1.1 200 OK"));
    assert!(metrics_response.contains("HTTP/1.1 200 OK"));
    route_render_contract_tests::assert_common_route_metrics(metrics_response.as_str());
    assert!(metrics_response.contains("kamn_service_api_observability_source{source=\"service-api-runtime\"} 1"));
    assert_server_ok(server.server, "service api endpoint should stop cleanly after configured request budget");
}

#[test]
fn integration_service_api_endpoint_async_runtime_handles_concurrent_http_routes() {
    let _env = acquire_service_api_test_env();
    let snapshot = build_transport_snapshot("127.0.0.1:34066");
    let server = spawn_transport_server(&snapshot, 8);
    let (health, metrics, send_one, send_two) = run_async_transport_burst(server.bind_addr.as_str(), state_hash(&snapshot).as_str());

    assert!(health.expect("async health request should succeed").contains("HTTP/1.1 200 OK"));
    assert!(metrics.expect("async metrics request should succeed").contains("HTTP/1.1 200 OK"));
    assert!(send_one.expect("async send request one should succeed").contains("HTTP/1.1 202 Accepted"));
    assert!(send_two.expect("async send request two should succeed").contains("HTTP/1.1 202 Accepted"));
    let result = server.server.join().expect("endpoint thread should complete");
    let ended_cleanly_or_timeout =
        result.is_ok() || result.as_ref().is_err_and(|error| error.contains("service api timed out after"));
    assert!(ended_cleanly_or_timeout);
}

#[test]
fn integration_service_api_endpoint_tls_mode_serves_required_https_routes() {
    let _env = acquire_service_api_test_env();
    let (cert_file, key_file) = write_test_service_api_tls_materials();
    let _tls_mode = EnvVarGuard::set("KAMN_SERVICE_API_TLS_MODE", Some("require"));
    let _tls_cert = EnvVarGuard::set("KAMN_SERVICE_API_TLS_CERT_FILE", Some(cert_file.as_str()));
    let _tls_key = EnvVarGuard::set("KAMN_SERVICE_API_TLS_KEY_FILE", Some(key_file.as_str()));
    let snapshot = build_transport_snapshot("127.0.0.1:34091");
    let server = spawn_transport_server(&snapshot, 2);
    let health = send_https_request_with_headers(server.bind_addr.as_str(), "GET", "/healthz", "", &[], TEST_SERVICE_API_TLS_CERT_PEM);
    let metrics = send_https_request_with_headers(server.bind_addr.as_str(), "GET", "/metrics", "", &[], TEST_SERVICE_API_TLS_CERT_PEM);

    assert!(health.contains("HTTP/1.1 200 OK"));
    assert!(health.contains("\"status\":\"ok\""));
    assert!(metrics.contains("HTTP/1.1 200 OK"));
    route_render_contract_tests::assert_common_route_metrics(metrics.as_str());
    assert!(metrics.contains("kamn_service_api_observability_source{source=\"service-api-runtime\"} 1"));
    assert_server_ok(server.server, "service api endpoint tls mode should stop cleanly after configured request budget");
}

#[test]
fn regression_service_api_endpoint_tls_mode_rejects_missing_cert_file() {
    let _env = acquire_service_api_test_env();
    let missing_cert = std::env::temp_dir().join("kamn-service-api-missing-cert.pem");
    let missing_key = std::env::temp_dir().join("kamn-service-api-missing-key.pem");
    let _tls_mode = EnvVarGuard::set("KAMN_SERVICE_API_TLS_MODE", Some("require"));
    let _tls_cert = EnvVarGuard::set("KAMN_SERVICE_API_TLS_CERT_FILE", Some(missing_cert.to_string_lossy().as_ref()));
    let _tls_key = EnvVarGuard::set("KAMN_SERVICE_API_TLS_KEY_FILE", Some(missing_key.to_string_lossy().as_ref()));
    let snapshot = build_transport_snapshot("127.0.0.1:34101");
    let bind_addr = reserve_loopback_addr();
    let error = serve_service_api_endpoint(&ServiceApiEndpointConfig {
        bind_addr,
        max_requests: 1,
        idle_timeout_ms: 2_000,
        body_limit_bytes: DEFAULT_SERVICE_API_BODY_LIMIT_BYTES,
        concurrency_limit: DEFAULT_SERVICE_API_CONCURRENCY_LIMIT,
        rate_limit_per_second: DEFAULT_SERVICE_API_RATE_LIMIT_PER_SECOND,
    }, &snapshot).expect_err("missing tls cert should fail closed");
    assert!(error.contains("service api tls certificate file read failed"));
}

#[test]
fn regression_service_api_endpoint_rejects_disabled_tls_for_non_loopback_api_runtime_path() {
    let _env = acquire_service_api_test_env();
    let _tls_mode = EnvVarGuard::set("KAMN_SERVICE_API_TLS_MODE", Some("disabled"));
    let snapshot = build_transport_snapshot("127.0.0.1:34102");
    let error = serve_service_api_endpoint(&ServiceApiEndpointConfig {
        bind_addr: "0.0.0.0:34103".to_owned(),
        max_requests: 1,
        idle_timeout_ms: 2_000,
        body_limit_bytes: DEFAULT_SERVICE_API_BODY_LIMIT_BYTES,
        concurrency_limit: DEFAULT_SERVICE_API_CONCURRENCY_LIMIT,
        rate_limit_per_second: DEFAULT_SERVICE_API_RATE_LIMIT_PER_SECOND,
    }, &snapshot).expect_err("disabled tls must fail closed for non-loopback api runtime path");
    assert!(error.contains("service api tls disabled is forbidden"));
}

#[test]
fn integration_service_api_endpoint_http_response_bodies_match_serde_contracts() {
    let _env = acquire_service_api_test_env();
    let snapshot = build_transport_snapshot("127.0.0.1:34061");
    let server = spawn_transport_server(&snapshot, 2);
    let send = send_signed_message_request(&snapshot, server.bind_addr.as_str(), "kamn:did:agent:test-client-serde", 31, "{\"message\":\"serde-live\"}");
    let health = send_http_request(server.bind_addr.as_str(), "GET", "/healthz", "");

    assert!(send.contains("HTTP/1.1 202 Accepted"));
    assert!(health.contains("HTTP/1.1 200 OK"));
    let send_payload: ServiceApiMessageCreateBody = parse_service_api_payload(extract_http_response_body(send.as_str())).expect("send payload should deserialize");
    let health_payload: ServiceApiHealthBody = parse_service_api_payload(extract_http_response_body(health.as_str())).expect("health payload should deserialize");
    assert_eq!(send_payload.status, "created");
    assert_eq!(send_payload.runtime_mode, "api");
    assert_eq!(health_payload.status, "ok");
    assert_eq!(health_payload.runtime_mode, "api");
    assert_server_ok(server.server, "service api endpoint should stop cleanly after configured request budget");
}

async fn async_signed_send(bind_addr: &str, sender_did: &str, nonce: u64, state_hash: &str, body: &str) -> Result<String, String> {
    let signature = service_api_request_signature_for_fields(sender_did, nonce, state_hash, body);
    let nonce_text = nonce.to_string();
    let headers = [
        ("X-KAMN-Sender-DID", sender_did),
        ("X-KAMN-Request-Nonce", nonce_text.as_str()),
        ("X-KAMN-Request-Signature", signature.as_str()),
    ];
    send_http_request_with_headers_async(bind_addr, "POST", "/v1/messages/send", body, &headers).await
}

fn run_async_transport_burst(bind_addr: &str, state_hash: &str) -> (Result<String, String>, Result<String, String>, Result<String, String>, Result<String, String>) {
    let runtime =
        tokio::runtime::Builder::new_multi_thread().worker_threads(2).enable_all().build().expect("async runtime should initialize");
    runtime.block_on(async move {
        let one = async_signed_send(bind_addr, "kamn:did:agent:async-http-client-1", 900, state_hash, "{\"message\":\"async-route-1\"}");
        let two = async_signed_send(bind_addr, "kamn:did:agent:async-http-client-2", 901, state_hash, "{\"message\":\"async-route-2\"}");
        tokio::join!(
            send_http_request_with_headers_async(bind_addr, "GET", "/healthz", "", &[]),
            send_http_request_with_headers_async(bind_addr, "GET", "/metrics", "", &[]),
            one,
            two,
        )
    })
}
