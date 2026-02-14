use super::*;
use kamn_core::baseline_signature_for_fields;
use std::io::{ErrorKind, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::thread;
use std::time::{Duration, Instant};

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
        max_requests: 4,
        idle_timeout_ms: 2_000,
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

    let server_result = server.join().expect("endpoint thread should complete");
    assert!(
        server_result.is_ok(),
        "service api endpoint should stop cleanly after configured request budget"
    );
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
        max_requests: 2,
        idle_timeout_ms: 2_000,
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
    assert!(unauth_response.contains("\"error\":\"unauthorized\""));
    assert!(unauth_response.contains("x-kamn-sender-did"));

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
        max_requests: 3,
        idle_timeout_ms: 2_000,
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
    assert!(replay_response.contains("\"error\":\"replay\""));

    let server_result = server.join().expect("endpoint thread should complete");
    assert!(
        server_result.is_ok(),
        "service api endpoint should stop cleanly after configured request budget"
    );
}
