use super::*;
use crate::service_api_endpoint::{
    parse_service_api_payload, ServiceApiAgentGetBody, ServiceApiChannelCreateBody,
    ServiceApiHealthBody, ServiceApiMessageCreateBody, ServiceApiTaskCreateBody,
    DEFAULT_SERVICE_API_BODY_LIMIT_BYTES, DEFAULT_SERVICE_API_CONCURRENCY_LIMIT,
    DEFAULT_SERVICE_API_RATE_LIMIT_PER_SECOND,
};
use kamn_core::baseline_signature_for_fields;
use serde::Deserialize;
use std::io::{ErrorKind, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Barrier};
use std::thread;
use std::time::{Duration, Instant};

#[derive(Debug, Deserialize)]
struct ServiceApiErrorEnvelope {
    error: String,
    reason_code: String,
    message: String,
}

fn reserve_loopback_addr() -> String {
    let listener = TcpListener::bind("127.0.0.1:0").expect("listener should bind");
    let addr = listener.local_addr().expect("local addr should resolve");
    drop(listener);
    addr.to_string()
}

fn send_http_request(addr: &str, method: &str, path: &str, body: &str) -> String {
    send_http_request_with_headers(addr, method, path, body, &[])
}

fn send_http_request_with_headers(
    addr: &str,
    method: &str,
    path: &str,
    body: &str,
    headers: &[(&str, &str)],
) -> String {
    let mut stream = TcpStream::connect(addr).expect("endpoint should accept connections");
    stream
        .set_read_timeout(Some(Duration::from_secs(2)))
        .expect("read timeout should be configurable");
    let mut header_lines = String::new();
    for (name, value) in headers {
        header_lines.push_str(name);
        header_lines.push_str(": ");
        header_lines.push_str(value);
        header_lines.push_str("\r\n");
    }
    let request = format!(
        "{method} {path} HTTP/1.1\r\nHost: {addr}\r\nContent-Type: application/json\r\n{}Content-Length: {}\r\nConnection: close\r\n\r\n{}",
        header_lines,
        body.len(),
        body
    );
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

fn send_websocket_upgrade_request(addr: &str, path: &str, headers: &[(&str, &str)]) -> Vec<u8> {
    send_websocket_upgrade_request_with_version(addr, path, "13", headers)
}

fn send_websocket_upgrade_request_with_version(
    addr: &str,
    path: &str,
    websocket_version: &str,
    headers: &[(&str, &str)],
) -> Vec<u8> {
    let mut stream = TcpStream::connect(addr).expect("endpoint should accept websocket connection");
    stream
        .set_read_timeout(Some(Duration::from_secs(2)))
        .expect("websocket read timeout should be configurable");
    let mut header_lines = String::new();
    for (name, value) in headers {
        header_lines.push_str(name);
        header_lines.push_str(": ");
        header_lines.push_str(value);
        header_lines.push_str("\r\n");
    }
    let request = format!(
        "GET {path} HTTP/1.1\r\nHost: {addr}\r\nConnection: Upgrade\r\nUpgrade: websocket\r\nSec-WebSocket-Key: test-kamn-key\r\nSec-WebSocket-Version: {websocket_version}\r\n{}Content-Length: 0\r\n\r\n",
        header_lines
    );
    stream
        .write_all(request.as_bytes())
        .expect("websocket upgrade request should write");
    let mut response = Vec::new();
    let mut chunk = [0_u8; 1024];
    loop {
        match stream.read(&mut chunk) {
            Ok(0) => break,
            Ok(read_count) => response.extend_from_slice(&chunk[..read_count]),
            Err(error) if matches!(error.kind(), ErrorKind::WouldBlock | ErrorKind::TimedOut) => {
                break;
            }
            Err(error) => panic!("websocket response should be readable: {error}"),
        }
    }
    response
}

fn parse_http_content_length(response_head: &str) -> usize {
    for line in response_head.lines() {
        if let Some((name, value)) = line.split_once(':') {
            if name.eq_ignore_ascii_case("Content-Length") {
                return value.trim().parse::<usize>().unwrap_or(0);
            }
        }
    }
    0
}

fn extract_http_response_body(response: &str) -> &str {
    response
        .split_once("\r\n\r\n")
        .map(|(_, body)| body)
        .unwrap_or("")
}

fn parse_error_envelope(body: &str) -> ServiceApiErrorEnvelope {
    serde_json::from_str(body).expect("error payload should deserialize")
}

fn parse_error_envelope_from_http_response(response: &str) -> ServiceApiErrorEnvelope {
    parse_error_envelope(extract_http_response_body(response))
}

fn read_single_http_response(stream: &mut TcpStream) -> String {
    let mut response = Vec::new();
    let mut chunk = [0_u8; 1024];
    let mut header_end: Option<usize> = None;
    let mut expected_len: Option<usize> = None;

    loop {
        match stream.read(&mut chunk) {
            Ok(0) => break,
            Ok(read_count) => {
                response.extend_from_slice(&chunk[..read_count]);
                if header_end.is_none() {
                    header_end = response
                        .windows(4)
                        .position(|window| window == b"\r\n\r\n")
                        .map(|index| index + 4);
                    if let Some(header_end_index) = header_end {
                        let head = String::from_utf8_lossy(&response[..header_end_index]);
                        let content_len = parse_http_content_length(head.as_ref());
                        expected_len = Some(header_end_index + content_len);
                    }
                }
                if let Some(total) = expected_len {
                    if response.len() >= total {
                        break;
                    }
                }
            }
            Err(error) if matches!(error.kind(), ErrorKind::WouldBlock | ErrorKind::TimedOut) => {
                break;
            }
            Err(error) => panic!("response should be readable: {error}"),
        }
    }

    String::from_utf8(response).expect("http response should be utf-8")
}

fn parse_websocket_response(response: &[u8]) -> (String, String) {
    let header_end = response
        .windows(4)
        .position(|window| window == b"\r\n\r\n")
        .map(|index| index + 4)
        .expect("websocket response should include header terminator");
    let header = std::str::from_utf8(&response[..header_end])
        .expect("websocket header should be utf-8")
        .to_owned();
    let frame = &response[header_end..];
    assert!(
        frame.len() >= 2,
        "websocket response should include at least one frame"
    );
    assert_eq!(
        frame[0], 0x81,
        "expected single-frame text websocket opcode"
    );
    let payload_len = (frame[1] & 0x7f) as usize;
    assert_eq!(
        frame[1] & 0x80,
        0,
        "server websocket frame must be unmasked"
    );
    assert!(
        frame.len() >= payload_len + 2,
        "websocket frame payload length must be available"
    );
    let payload = std::str::from_utf8(&frame[2..2 + payload_len])
        .expect("websocket payload should be utf-8")
        .to_owned();
    (header, payload)
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
fn functional_service_api_endpoint_renders_required_route_contracts() {
    let parsed = parse_args(vec![
        "kamn-node".to_owned(),
        "--role".to_owned(),
        "processor".to_owned(),
        "--runtime-mode".to_owned(),
        "api".to_owned(),
        "--api-bind".to_owned(),
        "127.0.0.1:34051".to_owned(),
    ])
    .expect("api args should parse");
    let report = execute(parsed).expect("api execution should succeed");
    let snapshot = build_service_api_snapshot(&report);

    let send_response = render_service_api_endpoint_response(
        &snapshot,
        "POST",
        "/v1/messages/send",
        "{\"message\":\"hello\"}",
    );
    assert_eq!(send_response.status_code, 202);
    assert!(send_response.body.contains("\"message_id\":\"msg-local-"));

    let read_response =
        render_service_api_endpoint_response(&snapshot, "GET", "/v1/messages/msg-7", "");
    assert_eq!(read_response.status_code, 200);
    assert!(read_response.body.contains("\"status\":\"created\""));

    let channel_response = render_service_api_endpoint_response(
        &snapshot,
        "GET",
        "/v1/channels/channel-1/messages",
        "",
    );
    assert_eq!(channel_response.status_code, 200);

    let task_response =
        render_service_api_endpoint_response(&snapshot, "GET", "/v1/tasks/task-1", "");
    assert_eq!(task_response.status_code, 200);

    let agent_response = render_service_api_endpoint_response(
        &snapshot,
        "GET",
        "/v1/agents/kamn:did:agent:alpha",
        "",
    );
    assert_eq!(agent_response.status_code, 200);

    let health_response = render_service_api_endpoint_response(&snapshot, "GET", "/healthz", "");
    assert_eq!(health_response.status_code, 200);

    let metrics_response = render_service_api_endpoint_response(&snapshot, "GET", "/metrics", "");
    assert_eq!(metrics_response.status_code, 200);
    assert!(metrics_response
        .body
        .contains("kamn_service_api_observability_source{source=\"unknown\"} 1"));
    assert!(metrics_response
        .body
        .contains("kamn_service_api_observability_health{health=\"unknown\"} 0"));
    assert!(
        metrics_response
            .body
            .contains("kamn_service_api_observability_latency_p50_ms 0"),
        "metrics payload should publish runtime telemetry gauges even before daemon/kolme telemetry is available"
    );

    let ws_response = render_service_api_endpoint_response(&snapshot, "GET", "/v1/events/ws", "");
    assert_eq!(ws_response.status_code, 400);
    let ws_payload = parse_error_envelope(ws_response.body.as_str());
    assert_eq!(ws_payload.error, "bad-request");
    assert_eq!(
        ws_payload.reason_code,
        "service_api_websocket_upgrade_required"
    );
    assert!(ws_payload.message.contains("websocket upgrade required"));
}

#[test]
fn unit_service_api_endpoint_serde_payload_roundtrip_contracts() {
    let parsed = parse_args(vec![
        "kamn-node".to_owned(),
        "--role".to_owned(),
        "processor".to_owned(),
        "--runtime-mode".to_owned(),
        "api".to_owned(),
        "--api-bind".to_owned(),
        "127.0.0.1:34060".to_owned(),
    ])
    .expect("api args should parse");
    let report = execute(parsed).expect("api execution should succeed");
    let snapshot = build_service_api_snapshot(&report);

    let health = render_service_api_endpoint_response(&snapshot, "GET", "/healthz", "");
    let health_payload: ServiceApiHealthBody =
        parse_service_api_payload(health.body.as_str()).expect("health payload should deserialize");
    assert_eq!(health_payload.status, "ok");
    assert_eq!(health_payload.runtime_mode, "api");

    let send = render_service_api_endpoint_response(
        &snapshot,
        "POST",
        "/v1/messages/send",
        "{\"message\":\"serde\"}",
    );
    let send_payload: ServiceApiMessageCreateBody =
        parse_service_api_payload(send.body.as_str()).expect("send payload should deserialize");
    assert_eq!(send_payload.status, "created");
    assert!(send_payload.message_id.starts_with("msg-local-"));

    let channel = render_service_api_endpoint_response(
        &snapshot,
        "POST",
        "/v1/channels/create",
        "{\"name\":\"alpha\"}",
    );
    let channel_payload: ServiceApiChannelCreateBody =
        parse_service_api_payload(channel.body.as_str())
            .expect("channel payload should deserialize");
    assert_eq!(channel_payload.status, "created");

    let task = render_service_api_endpoint_response(
        &snapshot,
        "POST",
        "/v1/tasks/create",
        "{\"task\":\"x\"}",
    );
    let task_payload: ServiceApiTaskCreateBody =
        parse_service_api_payload(task.body.as_str()).expect("task payload should deserialize");
    assert_eq!(task_payload.state, "submitted");

    let agent = render_service_api_endpoint_response(
        &snapshot,
        "GET",
        "/v1/agents/kamn:did:agent:alpha",
        "",
    );
    let agent_payload: ServiceApiAgentGetBody =
        parse_service_api_payload(agent.body.as_str()).expect("agent payload should deserialize");
    assert_eq!(agent_payload.did, "kamn:did:agent:alpha");
    assert_eq!(agent_payload.reputation_score, 500);
}

#[test]
fn unit_service_api_endpoint_error_envelopes_use_reason_code_and_message_contracts() {
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

    let websocket_required =
        render_service_api_endpoint_response(&snapshot, "GET", "/v1/events/ws", "");
    assert_eq!(websocket_required.status_code, 400);
    let websocket_required_payload = parse_error_envelope(websocket_required.body.as_str());
    assert_eq!(websocket_required_payload.error, "bad-request");
    assert_eq!(
        websocket_required_payload.reason_code,
        "service_api_websocket_upgrade_required"
    );
    assert!(websocket_required_payload
        .message
        .contains("websocket upgrade required"));

    let method_not_allowed =
        render_service_api_endpoint_response(&snapshot, "DELETE", "/v1/messages/send", "");
    assert_eq!(method_not_allowed.status_code, 405);
    let method_not_allowed_payload = parse_error_envelope(method_not_allowed.body.as_str());
    assert_eq!(method_not_allowed_payload.error, "method-not-allowed");
    assert_eq!(
        method_not_allowed_payload.reason_code,
        "service_api_method_not_allowed"
    );
    assert!(method_not_allowed_payload
        .message
        .contains("method not allowed"));

    let not_found = render_service_api_endpoint_response(&snapshot, "GET", "/v1/nope", "");
    assert_eq!(not_found.status_code, 404);
    let not_found_payload = parse_error_envelope(not_found.body.as_str());
    assert_eq!(not_found_payload.error, "not-found");
    assert_eq!(not_found_payload.reason_code, "service_api_route_not_found");
    assert!(not_found_payload.message.contains("not found"));
}

#[test]
fn regression_service_api_payload_parse_reason_codes_fail_closed() {
    let syntax_error = parse_service_api_payload::<ServiceApiHealthBody>("{\"status\":\"ok\"");
    let syntax_reason = syntax_error.expect_err("invalid json syntax should fail closed");
    assert!(
        syntax_reason.starts_with("service_api_payload_json_syntax_invalid:"),
        "unexpected syntax reason marker: {syntax_reason}"
    );

    let structure_error = parse_service_api_payload::<ServiceApiHealthBody>(
        "{\"status\":\"ok\",\"runtime_mode\":\"api\"}",
    );
    let structure_reason =
        structure_error.expect_err("invalid payload structure should fail closed");
    assert!(
        structure_reason.starts_with("service_api_payload_structure_invalid:"),
        "unexpected structure reason marker: {structure_reason}"
    );
}

#[test]
fn integration_service_api_endpoint_serves_required_http_routes() {
    let parsed = parse_args(vec![
        "kamn-node".to_owned(),
        "--role".to_owned(),
        "processor".to_owned(),
        "--runtime-mode".to_owned(),
        "api".to_owned(),
        "--api-bind".to_owned(),
        "127.0.0.1:34052".to_owned(),
    ])
    .expect("api args should parse");
    let report = execute(parsed).expect("api execution should succeed");
    let snapshot = build_service_api_snapshot(&report);

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
    let server =
        thread::spawn(move || serve_service_api_endpoint(&endpoint_config, &server_snapshot));
    wait_for_endpoint_ready(bind_addr.as_str());

    let message_body = "{\"message\":\"hello\"}";
    let sender_did = "kamn:did:agent:test-client-1";
    let sender_nonce = 1_u64;
    let state_hash = format!(
        "service-api:{}:{}",
        snapshot.chain_id.as_str(),
        snapshot.chain_version.as_str()
    );
    let signature =
        baseline_signature_for_fields(sender_did, sender_nonce, state_hash.as_str(), message_body);
    let send_response = send_http_request_with_headers(
        bind_addr.as_str(),
        "POST",
        "/v1/messages/send",
        message_body,
        &[
            ("X-KAMN-Sender-DID", sender_did),
            ("X-KAMN-Request-Nonce", "1"),
            ("X-KAMN-Request-Signature", signature.as_str()),
        ],
    );
    let health_response = send_http_request(bind_addr.as_str(), "GET", "/healthz", "");
    let metrics_response = send_http_request(bind_addr.as_str(), "GET", "/metrics", "");

    assert!(send_response.contains("HTTP/1.1 202 Accepted"));
    assert!(send_response.contains("\"message_id\":\"msg-local-"));
    assert!(health_response.contains("HTTP/1.1 200 OK"));
    assert!(metrics_response.contains("HTTP/1.1 200 OK"));
    assert!(
        metrics_response.contains("kamn_service_api_observability_source{source=\"unknown\"} 1")
    );

    let server_result = server.join().expect("endpoint thread should complete");
    assert!(
        server_result.is_ok(),
        "service api endpoint should stop cleanly after configured request budget"
    );
}

#[test]
fn integration_service_api_endpoint_http_response_bodies_match_serde_contracts() {
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

    let bind_addr = reserve_loopback_addr();
    let endpoint_config = ServiceApiEndpointConfig {
        bind_addr: bind_addr.clone(),
        max_requests: 2,
        idle_timeout_ms: 2_000,
        body_limit_bytes: DEFAULT_SERVICE_API_BODY_LIMIT_BYTES,
        concurrency_limit: DEFAULT_SERVICE_API_CONCURRENCY_LIMIT,
        rate_limit_per_second: DEFAULT_SERVICE_API_RATE_LIMIT_PER_SECOND,
    };

    let server_snapshot = snapshot.clone();
    let server =
        thread::spawn(move || serve_service_api_endpoint(&endpoint_config, &server_snapshot));
    wait_for_endpoint_ready(bind_addr.as_str());

    let message_body = "{\"message\":\"serde-live\"}";
    let sender_did = "kamn:did:agent:test-client-serde";
    let sender_nonce = 31_u64;
    let state_hash = format!(
        "service-api:{}:{}",
        snapshot.chain_id.as_str(),
        snapshot.chain_version.as_str()
    );
    let signature =
        baseline_signature_for_fields(sender_did, sender_nonce, state_hash.as_str(), message_body);
    let send_response = send_http_request_with_headers(
        bind_addr.as_str(),
        "POST",
        "/v1/messages/send",
        message_body,
        &[
            ("X-KAMN-Sender-DID", sender_did),
            ("X-KAMN-Request-Nonce", "31"),
            ("X-KAMN-Request-Signature", signature.as_str()),
        ],
    );
    let health_response = send_http_request(bind_addr.as_str(), "GET", "/healthz", "");
    assert!(send_response.contains("HTTP/1.1 202 Accepted"));
    assert!(health_response.contains("HTTP/1.1 200 OK"));

    let send_payload: ServiceApiMessageCreateBody =
        parse_service_api_payload(extract_http_response_body(send_response.as_str()))
            .expect("send payload should deserialize");
    assert_eq!(send_payload.status, "created");
    assert_eq!(send_payload.runtime_mode, "api");

    let health_payload: ServiceApiHealthBody =
        parse_service_api_payload(extract_http_response_body(health_response.as_str()))
            .expect("health payload should deserialize");
    assert_eq!(health_payload.status, "ok");
    assert_eq!(health_payload.runtime_mode, "api");

    let server_result = server.join().expect("endpoint thread should complete");
    assert!(
        server_result.is_ok(),
        "service api endpoint should stop cleanly after configured request budget"
    );
}

#[test]
fn integration_service_api_endpoint_supports_keep_alive_requests_on_single_connection() {
    let parsed = parse_args(vec![
        "kamn-node".to_owned(),
        "--role".to_owned(),
        "processor".to_owned(),
        "--runtime-mode".to_owned(),
        "api".to_owned(),
        "--api-bind".to_owned(),
        "127.0.0.1:34059".to_owned(),
    ])
    .expect("api args should parse");
    let report = execute(parsed).expect("api execution should succeed");
    let snapshot = build_service_api_snapshot(&report);

    let bind_addr = reserve_loopback_addr();
    let endpoint_config = ServiceApiEndpointConfig {
        bind_addr: bind_addr.clone(),
        max_requests: 2,
        idle_timeout_ms: 2_000,
        body_limit_bytes: DEFAULT_SERVICE_API_BODY_LIMIT_BYTES,
        concurrency_limit: DEFAULT_SERVICE_API_CONCURRENCY_LIMIT,
        rate_limit_per_second: DEFAULT_SERVICE_API_RATE_LIMIT_PER_SECOND,
    };
    let server_snapshot = snapshot.clone();
    let server =
        thread::spawn(move || serve_service_api_endpoint(&endpoint_config, &server_snapshot));
    wait_for_endpoint_ready(bind_addr.as_str());

    let mut stream = TcpStream::connect(bind_addr.as_str()).expect("endpoint should accept");
    stream
        .set_read_timeout(Some(Duration::from_secs(2)))
        .expect("read timeout should be configurable");

    let request_one = format!(
        "GET /healthz HTTP/1.1\r\nHost: {bind_addr}\r\nConnection: keep-alive\r\nContent-Length: 0\r\n\r\n"
    );
    stream
        .write_all(request_one.as_bytes())
        .expect("first request should write");
    let first_response = read_single_http_response(&mut stream);
    assert!(first_response.contains("HTTP/1.1 200 OK"));
    assert!(first_response.contains("\"status\":\"ok\""));

    let request_two = format!(
        "GET /metrics HTTP/1.1\r\nHost: {bind_addr}\r\nConnection: close\r\nContent-Length: 0\r\n\r\n"
    );
    stream
        .write_all(request_two.as_bytes())
        .expect("second request should write over keep-alive connection");
    let second_response = read_single_http_response(&mut stream);
    assert!(second_response.contains("HTTP/1.1 200 OK"));
    assert!(second_response.contains("kamn_service_api_observability_source{source=\"unknown\"} 1"));

    let server_result = server.join().expect("endpoint thread should complete");
    assert!(
        server_result.is_ok(),
        "service api endpoint should stop cleanly after keep-alive request budget"
    );
}

#[test]
fn functional_service_api_endpoint_emits_structured_ingress_correlation_markers() {
    let _lock = log_env_lock()
        .lock()
        .expect("log env lock should guard test mutation");
    let _level_guard = EnvVarGuard::set("KAMN_NODE_LOG_LEVEL", Some("info"));
    let _format_guard = EnvVarGuard::set("KAMN_NODE_LOG_FORMAT", Some("json"));
    let parsed = parse_args(vec![
        "kamn-node".to_owned(),
        "--role".to_owned(),
        "processor".to_owned(),
        "--runtime-mode".to_owned(),
        "api".to_owned(),
        "--api-bind".to_owned(),
        "127.0.0.1:34058".to_owned(),
    ])
    .expect("api args should parse");
    let report = execute(parsed).expect("api execution should succeed");
    let snapshot = build_service_api_snapshot(&report);

    let bind_addr = reserve_loopback_addr();
    let endpoint_config = ServiceApiEndpointConfig {
        bind_addr: bind_addr.clone(),
        max_requests: 1,
        idle_timeout_ms: 2_000,
        body_limit_bytes: DEFAULT_SERVICE_API_BODY_LIMIT_BYTES,
        concurrency_limit: DEFAULT_SERVICE_API_CONCURRENCY_LIMIT,
        rate_limit_per_second: DEFAULT_SERVICE_API_RATE_LIMIT_PER_SECOND,
    };
    let sender_did = "kamn:did:agent:test-client-correlation";
    let sender_nonce = 41_u64;
    let message_body = "{\"message\":\"structured-correlation\"}";
    let state_hash = format!(
        "service-api:{}:{}",
        snapshot.chain_id.as_str(),
        snapshot.chain_version.as_str()
    );
    let signature =
        baseline_signature_for_fields(sender_did, sender_nonce, state_hash.as_str(), message_body);
    let client_bind_addr = bind_addr.clone();
    let client = thread::spawn(move || {
        thread::sleep(Duration::from_millis(50));
        send_http_request_with_headers(
            client_bind_addr.as_str(),
            "POST",
            "/v1/messages/send",
            message_body,
            &[
                ("X-KAMN-Sender-DID", sender_did),
                ("X-KAMN-Request-Nonce", "41"),
                ("X-KAMN-Request-Signature", signature.as_str()),
            ],
        )
    });

    let (serve_result, captured_logs) =
        capture_test_logs(|| serve_service_api_endpoint(&endpoint_config, &snapshot));
    let response = client.join().expect("client request should complete");
    assert!(
        serve_result.is_ok(),
        "service api endpoint should serve one request"
    );
    assert!(response.contains("HTTP/1.1 202 Accepted"));

    let ingress_line = captured_logs
        .iter()
        .find(|line| line.contains("\"event\":\"service.api.request.received\""))
        .expect("service api ingress should emit received marker");
    let outcome_line = captured_logs
        .iter()
        .find(|line| line.contains("\"event\":\"service.api.request.outcome\""))
        .expect("service api ingress should emit outcome marker");
    let ingress_correlation = extract_json_string_field(ingress_line, "correlation_id")
        .expect("ingress marker should include correlation id");
    let outcome_correlation = extract_json_string_field(outcome_line, "correlation_id")
        .expect("outcome marker should include correlation id");
    assert_eq!(ingress_correlation, outcome_correlation);
    assert_eq!(
        extract_json_string_field(ingress_line, "method").as_deref(),
        Some("POST")
    );
    assert_eq!(
        extract_json_string_field(ingress_line, "path").as_deref(),
        Some("/v1/messages/send")
    );
    assert_eq!(
        extract_json_string_field(outcome_line, "status_code").as_deref(),
        Some("202")
    );
}

#[test]
fn unit_service_api_endpoint_metrics_use_runtime_observability_when_present() {
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
    let snapshot = build_service_api_snapshot(&report);

    assert_eq!(snapshot.observability_source, "daemon");
    assert_eq!(snapshot.observability_health, "healthy");
    let metrics_response = render_service_api_endpoint_response(&snapshot, "GET", "/metrics", "");
    assert_eq!(metrics_response.status_code, 200);
    assert!(metrics_response
        .body
        .contains("kamn_service_api_observability_source{source=\"daemon\"} 1"));
    assert!(metrics_response
        .body
        .contains("kamn_service_api_observability_health{health=\"healthy\"} 1"));
}

#[test]
fn integration_service_api_endpoint_rejects_missing_request_auth_headers() {
    let parsed = parse_args(vec![
        "kamn-node".to_owned(),
        "--role".to_owned(),
        "processor".to_owned(),
        "--runtime-mode".to_owned(),
        "api".to_owned(),
        "--api-bind".to_owned(),
        "127.0.0.1:34053".to_owned(),
    ])
    .expect("api args should parse");
    let report = execute(parsed).expect("api execution should succeed");
    let snapshot = build_service_api_snapshot(&report);

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

    let unauth_response = send_http_request(
        bind_addr.as_str(),
        "POST",
        "/v1/messages/send",
        "{\"message\":\"hello\"}",
    );
    assert!(unauth_response.contains("HTTP/1.1 401 Unauthorized"));
    let unauth_payload = parse_error_envelope_from_http_response(unauth_response.as_str());
    assert_eq!(unauth_payload.error, "unauthorized");
    assert_eq!(
        unauth_payload.reason_code,
        "service_api_auth_sender_did_header_missing"
    );
    assert!(unauth_payload.message.contains("x-kamn-sender-did"));

    let server_result = server.join().expect("endpoint thread should complete");
    assert!(
        server_result.is_ok(),
        "service api endpoint should stop cleanly after configured request budget"
    );
}

#[test]
fn functional_service_api_endpoint_rejects_when_rate_limit_is_exceeded() {
    let parsed = parse_args(vec![
        "kamn-node".to_owned(),
        "--role".to_owned(),
        "processor".to_owned(),
        "--runtime-mode".to_owned(),
        "api".to_owned(),
        "--api-bind".to_owned(),
        "127.0.0.1:34062".to_owned(),
    ])
    .expect("api args should parse");
    let report = execute(parsed).expect("api execution should succeed");
    let snapshot = build_service_api_snapshot(&report);

    let bind_addr = reserve_loopback_addr();
    let endpoint_config = ServiceApiEndpointConfig {
        bind_addr: bind_addr.clone(),
        max_requests: 2,
        idle_timeout_ms: 2_000,
        body_limit_bytes: DEFAULT_SERVICE_API_BODY_LIMIT_BYTES,
        concurrency_limit: DEFAULT_SERVICE_API_CONCURRENCY_LIMIT,
        rate_limit_per_second: 1,
    };
    let server_snapshot = snapshot.clone();
    let server =
        thread::spawn(move || serve_service_api_endpoint(&endpoint_config, &server_snapshot));
    wait_for_endpoint_ready(bind_addr.as_str());

    let message_body = "{\"message\":\"rate-limit-check\"}";
    let sender_did = "kamn:did:agent:test-client-rate-limit";
    let state_hash = format!(
        "service-api:{}:{}",
        snapshot.chain_id.as_str(),
        snapshot.chain_version.as_str()
    );
    let first_signature =
        baseline_signature_for_fields(sender_did, 101, state_hash.as_str(), message_body);
    let second_signature =
        baseline_signature_for_fields(sender_did, 102, state_hash.as_str(), message_body);

    let first_response = send_http_request_with_headers(
        bind_addr.as_str(),
        "POST",
        "/v1/messages/send",
        message_body,
        &[
            ("X-KAMN-Sender-DID", sender_did),
            ("X-KAMN-Request-Nonce", "101"),
            ("X-KAMN-Request-Signature", first_signature.as_str()),
        ],
    );
    let second_response = send_http_request_with_headers(
        bind_addr.as_str(),
        "POST",
        "/v1/messages/send",
        message_body,
        &[
            ("X-KAMN-Sender-DID", sender_did),
            ("X-KAMN-Request-Nonce", "102"),
            ("X-KAMN-Request-Signature", second_signature.as_str()),
        ],
    );

    assert!(first_response.contains("HTTP/1.1 202 Accepted"));
    assert!(second_response.contains("HTTP/1.1 429 Too Many Requests"));
    let second_payload = parse_error_envelope_from_http_response(second_response.as_str());
    assert_eq!(second_payload.error, "too-many-requests");
    assert_eq!(
        second_payload.reason_code,
        "service_api_ingress_rate_limit_exceeded"
    );
    assert!(second_payload
        .message
        .contains("ingress rate limit exceeded"));

    let server_result = server.join().expect("endpoint thread should complete");
    assert!(
        server_result.is_ok(),
        "service api endpoint should stop cleanly after configured request budget"
    );
}

#[test]
fn functional_service_api_endpoint_applies_sender_anti_spam_throttle_and_suspension() {
    let parsed = parse_args(vec![
        "kamn-node".to_owned(),
        "--role".to_owned(),
        "processor".to_owned(),
        "--runtime-mode".to_owned(),
        "api".to_owned(),
        "--api-bind".to_owned(),
        "127.0.0.1:34065".to_owned(),
    ])
    .expect("api args should parse");
    let report = execute(parsed).expect("api execution should succeed");
    let snapshot = build_service_api_snapshot(&report);

    let bind_addr = reserve_loopback_addr();
    let endpoint_config = ServiceApiEndpointConfig {
        bind_addr: bind_addr.clone(),
        max_requests: 6,
        idle_timeout_ms: 3_000,
        body_limit_bytes: DEFAULT_SERVICE_API_BODY_LIMIT_BYTES,
        concurrency_limit: DEFAULT_SERVICE_API_CONCURRENCY_LIMIT,
        rate_limit_per_second: 1_000,
    };
    let server_snapshot = snapshot.clone();
    let server =
        thread::spawn(move || serve_service_api_endpoint(&endpoint_config, &server_snapshot));
    wait_for_endpoint_ready(bind_addr.as_str());

    let message_body = "{\"message\":\"anti-spam-check\"}";
    let sender_did = "kamn:did:agent:test-client-anti-spam";
    let state_hash = format!(
        "service-api:{}:{}",
        snapshot.chain_id.as_str(),
        snapshot.chain_version.as_str()
    );

    let mut responses = Vec::new();
    for nonce in 610_u64..616_u64 {
        let signature =
            baseline_signature_for_fields(sender_did, nonce, state_hash.as_str(), message_body);
        let nonce_text = nonce.to_string();
        responses.push(send_http_request_with_headers(
            bind_addr.as_str(),
            "POST",
            "/v1/messages/send",
            message_body,
            &[
                ("X-KAMN-Sender-DID", sender_did),
                ("X-KAMN-Request-Nonce", nonce_text.as_str()),
                ("X-KAMN-Request-Signature", signature.as_str()),
            ],
        ));
    }

    assert!(responses[0].contains("HTTP/1.1 202 Accepted"));
    assert!(responses[1].contains("HTTP/1.1 202 Accepted"));
    assert!(responses[2].contains("HTTP/1.1 202 Accepted"));

    assert!(responses[3].contains("HTTP/1.1 429 Too Many Requests"));
    let fourth_payload = parse_error_envelope_from_http_response(responses[3].as_str());
    assert_eq!(fourth_payload.error, "too-many-requests");
    assert_eq!(
        fourth_payload.reason_code,
        "service_api_ingress_sender_rate_limit_exceeded"
    );

    assert!(responses[4].contains("HTTP/1.1 429 Too Many Requests"));
    let fifth_payload = parse_error_envelope_from_http_response(responses[4].as_str());
    assert_eq!(fifth_payload.error, "too-many-requests");
    assert_eq!(
        fifth_payload.reason_code,
        "service_api_ingress_sender_rate_limit_exceeded"
    );

    assert!(responses[5].contains("HTTP/1.1 429 Too Many Requests"));
    let sixth_payload = parse_error_envelope_from_http_response(responses[5].as_str());
    assert_eq!(sixth_payload.error, "too-many-requests");
    assert_eq!(
        sixth_payload.reason_code,
        "service_api_ingress_sender_suspended"
    );

    let server_result = server.join().expect("endpoint thread should complete");
    assert!(
        server_result.is_ok(),
        "service api endpoint should stop cleanly after configured request budget"
    );
}

#[test]
fn integration_service_api_endpoint_rejects_when_concurrency_limit_is_exceeded() {
    let parsed = parse_args(vec![
        "kamn-node".to_owned(),
        "--role".to_owned(),
        "processor".to_owned(),
        "--runtime-mode".to_owned(),
        "api".to_owned(),
        "--api-bind".to_owned(),
        "127.0.0.1:34063".to_owned(),
    ])
    .expect("api args should parse");
    let report = execute(parsed).expect("api execution should succeed");
    let snapshot = build_service_api_snapshot(&report);

    let bind_addr = reserve_loopback_addr();
    let worker_count = 6_usize;
    let endpoint_config = ServiceApiEndpointConfig {
        bind_addr: bind_addr.clone(),
        max_requests: worker_count as u64,
        idle_timeout_ms: 2_000,
        body_limit_bytes: DEFAULT_SERVICE_API_BODY_LIMIT_BYTES,
        concurrency_limit: 1,
        rate_limit_per_second: 1_000,
    };
    let server_snapshot = snapshot.clone();
    let server =
        thread::spawn(move || serve_service_api_endpoint(&endpoint_config, &server_snapshot));
    wait_for_endpoint_ready(bind_addr.as_str());

    let sender_did = "kamn:did:agent:test-client-concurrency-limit";
    let message_body = "{\"message\":\"concurrency-limit-check\"}";
    let state_hash = format!(
        "service-api:{}:{}",
        snapshot.chain_id.as_str(),
        snapshot.chain_version.as_str()
    );

    let barrier = Arc::new(Barrier::new(worker_count));
    let mut clients = Vec::with_capacity(worker_count);
    for request_index in 0..worker_count {
        let client_bind_addr = bind_addr.clone();
        let barrier = barrier.clone();
        let state_hash = state_hash.clone();
        clients.push(thread::spawn(move || {
            let nonce = 200 + request_index as u64;
            let signature =
                baseline_signature_for_fields(sender_did, nonce, state_hash.as_str(), message_body);
            let nonce_text = nonce.to_string();
            barrier.wait();
            send_http_request_with_headers(
                client_bind_addr.as_str(),
                "POST",
                "/v1/messages/send",
                message_body,
                &[
                    ("X-KAMN-Sender-DID", sender_did),
                    ("X-KAMN-Request-Nonce", nonce_text.as_str()),
                    ("X-KAMN-Request-Signature", signature.as_str()),
                ],
            )
        }));
    }

    let responses = clients
        .into_iter()
        .map(|client| client.join().expect("client request should complete"))
        .collect::<Vec<String>>();

    assert!(
        responses
            .iter()
            .any(|response| response.contains("HTTP/1.1 202 Accepted")),
        "expected at least one accepted request under constrained concurrency"
    );
    let concurrency_rejection = responses
        .iter()
        .find(|response| response.contains("HTTP/1.1 429 Too Many Requests"))
        .expect("expected at least one request to fail closed on concurrency limit");
    let rejection_payload = parse_error_envelope_from_http_response(concurrency_rejection);
    assert_eq!(rejection_payload.error, "too-many-requests");
    assert_eq!(
        rejection_payload.reason_code,
        "service_api_ingress_concurrency_limit_exceeded"
    );
    assert!(rejection_payload
        .message
        .contains("ingress concurrency limit exceeded"));

    let server_result = server.join().expect("endpoint thread should complete");
    assert!(
        server_result.is_ok(),
        "service api endpoint should stop cleanly after configured request budget"
    );
}

#[test]
fn regression_service_api_endpoint_oversized_payload_maps_body_limit_reason_code() {
    let parsed = parse_args(vec![
        "kamn-node".to_owned(),
        "--role".to_owned(),
        "processor".to_owned(),
        "--runtime-mode".to_owned(),
        "api".to_owned(),
        "--api-bind".to_owned(),
        "127.0.0.1:34064".to_owned(),
    ])
    .expect("api args should parse");
    let report = execute(parsed).expect("api execution should succeed");
    let snapshot = build_service_api_snapshot(&report);

    let bind_addr = reserve_loopback_addr();
    let endpoint_config = ServiceApiEndpointConfig {
        bind_addr: bind_addr.clone(),
        max_requests: 1,
        idle_timeout_ms: 2_000,
        body_limit_bytes: 256,
        concurrency_limit: DEFAULT_SERVICE_API_CONCURRENCY_LIMIT,
        rate_limit_per_second: 1_000,
    };
    let server_snapshot = snapshot.clone();
    let server =
        thread::spawn(move || serve_service_api_endpoint(&endpoint_config, &server_snapshot));
    wait_for_endpoint_ready(bind_addr.as_str());

    let oversized_body = format!("{{\"message\":\"{}\"}}", "x".repeat(700));
    let sender_did = "kamn:did:agent:test-client-oversized";
    let nonce = 303_u64;
    let state_hash = format!(
        "service-api:{}:{}",
        snapshot.chain_id.as_str(),
        snapshot.chain_version.as_str()
    );
    let signature = baseline_signature_for_fields(
        sender_did,
        nonce,
        state_hash.as_str(),
        oversized_body.as_str(),
    );
    let response = send_http_request_with_headers(
        bind_addr.as_str(),
        "POST",
        "/v1/messages/send",
        oversized_body.as_str(),
        &[
            ("X-KAMN-Sender-DID", sender_did),
            ("X-KAMN-Request-Nonce", "303"),
            ("X-KAMN-Request-Signature", signature.as_str()),
        ],
    );
    assert!(response.contains("HTTP/1.1 400 Bad Request"));
    let payload = parse_error_envelope_from_http_response(response.as_str());
    assert_eq!(payload.error, "bad-request");
    assert_eq!(
        payload.reason_code,
        "service_api_ingress_body_size_limit_exceeded"
    );
    assert!(payload.message.contains("request body size limit exceeded"));

    let server_result = server.join().expect("endpoint thread should complete");
    assert!(
        server_result.is_ok(),
        "service api endpoint should stop cleanly after configured request budget"
    );
}

#[test]
fn regression_service_api_endpoint_rejects_replayed_request_nonce_for_sender() {
    let parsed = parse_args(vec![
        "kamn-node".to_owned(),
        "--role".to_owned(),
        "processor".to_owned(),
        "--runtime-mode".to_owned(),
        "api".to_owned(),
        "--api-bind".to_owned(),
        "127.0.0.1:34054".to_owned(),
    ])
    .expect("api args should parse");
    let report = execute(parsed).expect("api execution should succeed");
    let snapshot = build_service_api_snapshot(&report);

    let bind_addr = reserve_loopback_addr();
    let endpoint_config = ServiceApiEndpointConfig {
        bind_addr: bind_addr.clone(),
        max_requests: 2,
        idle_timeout_ms: 2_000,
        body_limit_bytes: DEFAULT_SERVICE_API_BODY_LIMIT_BYTES,
        concurrency_limit: DEFAULT_SERVICE_API_CONCURRENCY_LIMIT,
        rate_limit_per_second: DEFAULT_SERVICE_API_RATE_LIMIT_PER_SECOND,
    };
    let server_snapshot = snapshot.clone();
    let server =
        thread::spawn(move || serve_service_api_endpoint(&endpoint_config, &server_snapshot));
    wait_for_endpoint_ready(bind_addr.as_str());

    let message_body = "{\"message\":\"replay-check\"}";
    let sender_did = "kamn:did:agent:test-client-2";
    let sender_nonce = 7_u64;
    let state_hash = format!(
        "service-api:{}:{}",
        snapshot.chain_id.as_str(),
        snapshot.chain_version.as_str()
    );
    let signature =
        baseline_signature_for_fields(sender_did, sender_nonce, state_hash.as_str(), message_body);
    let auth_headers = [
        ("X-KAMN-Sender-DID", sender_did),
        ("X-KAMN-Request-Nonce", "7"),
        ("X-KAMN-Request-Signature", signature.as_str()),
    ];
    let first_response = send_http_request_with_headers(
        bind_addr.as_str(),
        "POST",
        "/v1/messages/send",
        message_body,
        &auth_headers,
    );
    let replay_response = send_http_request_with_headers(
        bind_addr.as_str(),
        "POST",
        "/v1/messages/send",
        message_body,
        &auth_headers,
    );

    assert!(first_response.contains("HTTP/1.1 202 Accepted"));
    assert!(replay_response.contains("HTTP/1.1 409 Conflict"));
    let replay_payload = parse_error_envelope_from_http_response(replay_response.as_str());
    assert_eq!(replay_payload.error, "replay");
    assert_eq!(
        replay_payload.reason_code,
        "service_api_auth_replay_nonce_detected"
    );
    assert!(replay_payload
        .message
        .contains("request nonce replay detected"));

    let server_result = server.join().expect("endpoint thread should complete");
    assert!(
        server_result.is_ok(),
        "service api endpoint should stop cleanly after configured request budget"
    );
}

#[test]
fn integration_service_api_endpoint_replay_rejection_remains_stable_with_anti_spam_enforcement() {
    let parsed = parse_args(vec![
        "kamn-node".to_owned(),
        "--role".to_owned(),
        "processor".to_owned(),
        "--runtime-mode".to_owned(),
        "api".to_owned(),
        "--api-bind".to_owned(),
        "127.0.0.1:34066".to_owned(),
    ])
    .expect("api args should parse");
    let report = execute(parsed).expect("api execution should succeed");
    let snapshot = build_service_api_snapshot(&report);

    let bind_addr = reserve_loopback_addr();
    let endpoint_config = ServiceApiEndpointConfig {
        bind_addr: bind_addr.clone(),
        max_requests: 3,
        idle_timeout_ms: 2_000,
        body_limit_bytes: DEFAULT_SERVICE_API_BODY_LIMIT_BYTES,
        concurrency_limit: DEFAULT_SERVICE_API_CONCURRENCY_LIMIT,
        rate_limit_per_second: 1_000,
    };
    let server_snapshot = snapshot.clone();
    let server =
        thread::spawn(move || serve_service_api_endpoint(&endpoint_config, &server_snapshot));
    wait_for_endpoint_ready(bind_addr.as_str());

    let message_body = "{\"message\":\"replay-anti-spam-matrix\"}";
    let sender_did = "kamn:did:agent:test-client-replay-anti-spam";
    let state_hash = format!(
        "service-api:{}:{}",
        snapshot.chain_id.as_str(),
        snapshot.chain_version.as_str()
    );
    let signature_nonce_one =
        baseline_signature_for_fields(sender_did, 701, state_hash.as_str(), message_body);
    let signature_nonce_two =
        baseline_signature_for_fields(sender_did, 702, state_hash.as_str(), message_body);

    let first_response = send_http_request_with_headers(
        bind_addr.as_str(),
        "POST",
        "/v1/messages/send",
        message_body,
        &[
            ("X-KAMN-Sender-DID", sender_did),
            ("X-KAMN-Request-Nonce", "701"),
            ("X-KAMN-Request-Signature", signature_nonce_one.as_str()),
        ],
    );
    let replay_response = send_http_request_with_headers(
        bind_addr.as_str(),
        "POST",
        "/v1/messages/send",
        message_body,
        &[
            ("X-KAMN-Sender-DID", sender_did),
            ("X-KAMN-Request-Nonce", "701"),
            ("X-KAMN-Request-Signature", signature_nonce_one.as_str()),
        ],
    );
    let fresh_nonce_response = send_http_request_with_headers(
        bind_addr.as_str(),
        "POST",
        "/v1/messages/send",
        message_body,
        &[
            ("X-KAMN-Sender-DID", sender_did),
            ("X-KAMN-Request-Nonce", "702"),
            ("X-KAMN-Request-Signature", signature_nonce_two.as_str()),
        ],
    );

    assert!(first_response.contains("HTTP/1.1 202 Accepted"));
    assert!(replay_response.contains("HTTP/1.1 409 Conflict"));
    let replay_payload = parse_error_envelope_from_http_response(replay_response.as_str());
    assert_eq!(replay_payload.error, "replay");
    assert_eq!(
        replay_payload.reason_code,
        "service_api_auth_replay_nonce_detected"
    );

    assert!(
        fresh_nonce_response.contains("HTTP/1.1 202 Accepted"),
        "replay rejection should not force anti-spam limiter rejection for the next valid nonce"
    );

    let server_result = server.join().expect("endpoint thread should complete");
    assert!(
        server_result.is_ok(),
        "service api endpoint should stop cleanly after configured request budget"
    );
}

#[test]
fn integration_service_api_endpoint_websocket_upgrade_streams_state_transition_event() {
    let parsed = parse_args(vec![
        "kamn-node".to_owned(),
        "--role".to_owned(),
        "processor".to_owned(),
        "--runtime-mode".to_owned(),
        "api".to_owned(),
        "--api-bind".to_owned(),
        "127.0.0.1:34055".to_owned(),
    ])
    .expect("api args should parse");
    let report = execute(parsed).expect("api execution should succeed");
    let snapshot = build_service_api_snapshot(&report);

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

    let sender_did = "kamn:did:agent:ws-client-1";
    let nonce = 19_u64;
    let state_hash = format!(
        "service-api:{}:{}",
        snapshot.chain_id.as_str(),
        snapshot.chain_version.as_str()
    );
    let signature = baseline_signature_for_fields(sender_did, nonce, state_hash.as_str(), "");
    let response = send_websocket_upgrade_request(
        bind_addr.as_str(),
        "/v1/events/ws",
        &[
            ("X-KAMN-Sender-DID", sender_did),
            ("X-KAMN-Request-Nonce", "19"),
            ("X-KAMN-Request-Signature", signature.as_str()),
        ],
    );
    let (header, payload) = parse_websocket_response(response.as_slice());
    assert!(header.contains("HTTP/1.1 101 Switching Protocols"));
    let normalized_header = header.to_ascii_lowercase();
    assert!(normalized_header.contains("upgrade: websocket"));
    assert!(normalized_header.contains("x-kamn-websocket-contract: v1"));
    assert!(payload.contains("\"event\":\"state-transition\""));
    assert!(payload.contains("\"runtime_mode\":\"api\""));
    assert!(payload.contains("\"role\":\"processor\""));

    let server_result = server.join().expect("endpoint thread should complete");
    assert!(
        server_result.is_ok(),
        "service api endpoint should stop cleanly after websocket request budget"
    );
}

#[test]
fn regression_service_api_endpoint_websocket_route_rejects_missing_upgrade_headers() {
    let parsed = parse_args(vec![
        "kamn-node".to_owned(),
        "--role".to_owned(),
        "processor".to_owned(),
        "--runtime-mode".to_owned(),
        "api".to_owned(),
        "--api-bind".to_owned(),
        "127.0.0.1:34056".to_owned(),
    ])
    .expect("api args should parse");
    let report = execute(parsed).expect("api execution should succeed");
    let snapshot = build_service_api_snapshot(&report);

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

    let sender_did = "kamn:did:agent:ws-client-2";
    let nonce = 23_u64;
    let state_hash = format!(
        "service-api:{}:{}",
        snapshot.chain_id.as_str(),
        snapshot.chain_version.as_str()
    );
    let signature = baseline_signature_for_fields(sender_did, nonce, state_hash.as_str(), "");
    let response = send_http_request_with_headers(
        bind_addr.as_str(),
        "GET",
        "/v1/events/ws",
        "",
        &[
            ("X-KAMN-Sender-DID", sender_did),
            ("X-KAMN-Request-Nonce", "23"),
            ("X-KAMN-Request-Signature", signature.as_str()),
        ],
    );
    assert!(response.contains("HTTP/1.1 400 Bad Request"));
    let payload = parse_error_envelope_from_http_response(response.as_str());
    assert_eq!(payload.error, "bad-request");
    assert_eq!(payload.reason_code, "service_api_ws_upgrade_header_missing");
    assert!(payload
        .message
        .contains("missing required websocket upgrade header"));

    let server_result = server.join().expect("endpoint thread should complete");
    assert!(
        server_result.is_ok(),
        "service api endpoint should stop cleanly after websocket rejection budget"
    );
}

#[test]
fn regression_service_api_endpoint_websocket_rejects_invalid_version_header() {
    let parsed = parse_args(vec![
        "kamn-node".to_owned(),
        "--role".to_owned(),
        "processor".to_owned(),
        "--runtime-mode".to_owned(),
        "api".to_owned(),
        "--api-bind".to_owned(),
        "127.0.0.1:34057".to_owned(),
    ])
    .expect("api args should parse");
    let report = execute(parsed).expect("api execution should succeed");
    let snapshot = build_service_api_snapshot(&report);

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

    let sender_did = "kamn:did:agent:ws-client-3";
    let nonce = 29_u64;
    let state_hash = format!(
        "service-api:{}:{}",
        snapshot.chain_id.as_str(),
        snapshot.chain_version.as_str()
    );
    let signature = baseline_signature_for_fields(sender_did, nonce, state_hash.as_str(), "");
    let response = send_websocket_upgrade_request_with_version(
        bind_addr.as_str(),
        "/v1/events/ws",
        "12",
        &[
            ("X-KAMN-Sender-DID", sender_did),
            ("X-KAMN-Request-Nonce", "29"),
            ("X-KAMN-Request-Signature", signature.as_str()),
        ],
    );
    let response_text =
        String::from_utf8(response).expect("invalid websocket version response should be utf-8");
    assert!(response_text.contains("HTTP/1.1 400 Bad Request"));
    let payload = parse_error_envelope_from_http_response(response_text.as_str());
    assert_eq!(payload.error, "bad-request");
    assert_eq!(payload.reason_code, "service_api_ws_version_header_invalid");
    assert!(payload.message.contains("invalid websocket version header"));

    let server_result = server.join().expect("endpoint thread should complete");
    assert!(
        server_result.is_ok(),
        "service api endpoint should stop cleanly after websocket version rejection"
    );
}
