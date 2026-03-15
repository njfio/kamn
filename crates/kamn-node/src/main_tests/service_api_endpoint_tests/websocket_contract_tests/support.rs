use super::super::*;
use crate::service_api_endpoint::ServiceApiSnapshot;

#[path = "support/frame_support.rs"]
mod frame_support;
#[path = "support/request_support.rs"]
mod request_support;

pub(super) use frame_support::*;
pub(super) use request_support::*;

pub(super) struct WebsocketHarness {
    pub bind_addr: String,
    pub snapshot: ServiceApiSnapshot,
    pub server: thread::JoinHandle<Result<(), String>>,
}

pub(super) fn build_websocket_harness(api_bind: &str, max_requests: u64) -> WebsocketHarness {
    let snapshot = build_websocket_snapshot(api_bind);
    let (bind_addr, server) = spawn_websocket_server(&snapshot, max_requests);
    WebsocketHarness {
        bind_addr,
        snapshot,
        server,
    }
}

fn build_websocket_snapshot(api_bind: &str) -> ServiceApiSnapshot {
    let parsed = parse_args(vec![
        "kamn-node".to_owned(),
        "--role".to_owned(),
        "processor".to_owned(),
        "--runtime-mode".to_owned(),
        "api".to_owned(),
        "--api-bind".to_owned(),
        api_bind.to_owned(),
    ])
    .expect("api args should parse");
    let report = execute(parsed).expect("api execution should succeed");
    build_service_api_snapshot(&report)
}

fn spawn_websocket_server(
    snapshot: &ServiceApiSnapshot,
    max_requests: u64,
) -> (String, thread::JoinHandle<Result<(), String>>) {
    let bind_addr = reserve_loopback_addr();
    let endpoint_config = ServiceApiEndpointConfig {
        bind_addr: bind_addr.clone(),
        max_requests,
        idle_timeout_ms: 2_000,
        body_limit_bytes: DEFAULT_SERVICE_API_BODY_LIMIT_BYTES,
        concurrency_limit: DEFAULT_SERVICE_API_CONCURRENCY_LIMIT,
        rate_limit_per_second: DEFAULT_SERVICE_API_RATE_LIMIT_PER_SECOND,
    };
    let server_snapshot = snapshot.clone();
    let server =
        thread::spawn(move || serve_service_api_endpoint(&endpoint_config, &server_snapshot));
    wait_for_endpoint_ready(bind_addr.as_str());
    (bind_addr, server)
}

pub(super) fn state_hash(snapshot: &ServiceApiSnapshot) -> String {
    format!(
        "service-api:{}:{}",
        snapshot.chain_id.as_str(),
        snapshot.chain_version.as_str()
    )
}

pub(super) fn websocket_signature(
    snapshot: &ServiceApiSnapshot,
    sender_did: &str,
    nonce: u64,
) -> String {
    service_api_request_signature_for_fields(sender_did, nonce, state_hash(snapshot).as_str(), "")
}

fn signed_websocket_headers<'a>(
    sender_did: &'a str,
    nonce: &'a str,
    signature: &'a str,
    extra_headers: &'a [(&'a str, &'a str)],
) -> Vec<(&'a str, &'a str)> {
    let mut headers = vec![
        ("X-KAMN-Sender-DID", sender_did),
        ("X-KAMN-Request-Nonce", nonce),
        ("X-KAMN-Request-Signature", signature),
    ];
    headers.extend_from_slice(extra_headers);
    headers
}

pub(super) fn send_signed_websocket_request(
    snapshot: &ServiceApiSnapshot,
    bind_addr: &str,
    sender_did: &str,
    nonce: u64,
    extra_headers: &[(&str, &str)],
) -> Vec<u8> {
    let signature = websocket_signature(snapshot, sender_did, nonce);
    let nonce_text = nonce.to_string();
    let headers = signed_websocket_headers(
        sender_did,
        nonce_text.as_str(),
        signature.as_str(),
        extra_headers,
    );
    send_websocket_upgrade_request(bind_addr, WEBSOCKET_EVENTS_PATH, headers.as_slice())
}

pub(super) fn send_signed_websocket_request_with_version(
    snapshot: &ServiceApiSnapshot,
    bind_addr: &str,
    sender_did: &str,
    nonce: u64,
    version: &str,
    extra_headers: &[(&str, &str)],
) -> Vec<u8> {
    let signature = websocket_signature(snapshot, sender_did, nonce);
    let nonce_text = nonce.to_string();
    let headers = signed_websocket_headers(
        sender_did,
        nonce_text.as_str(),
        signature.as_str(),
        extra_headers,
    );
    send_websocket_upgrade_request_with_version(
        bind_addr,
        WEBSOCKET_EVENTS_PATH,
        version,
        headers.as_slice(),
    )
}

pub(super) fn send_signed_websocket_request_with_close_observation(
    snapshot: &ServiceApiSnapshot,
    bind_addr: &str,
    sender_did: &str,
    nonce: u64,
    extra_headers: &[(&str, &str)],
) -> (Vec<u8>, bool) {
    let signature = websocket_signature(snapshot, sender_did, nonce);
    let nonce_text = nonce.to_string();
    let headers = signed_websocket_headers(
        sender_did,
        nonce_text.as_str(),
        signature.as_str(),
        extra_headers,
    );
    send_websocket_upgrade_request_with_version_close_observation(
        bind_addr,
        WEBSOCKET_EVENTS_PATH,
        "13",
        headers.as_slice(),
    )
}

pub(super) fn assert_server_ok(server: thread::JoinHandle<Result<(), String>>, context: &str) {
    let result = server.join().expect("endpoint thread should complete");
    assert!(result.is_ok(), "{context}");
}

pub(super) fn assert_server_ok_or_timeout(
    server: thread::JoinHandle<Result<(), String>>,
    context: &str,
) {
    let result = server.join().expect("endpoint thread should complete");
    let ended_cleanly_or_timeout = matches!(&result, Ok(()))
        || result
            .as_ref()
            .is_err_and(|error| error.contains("service api timed out after"));
    assert!(ended_cleanly_or_timeout, "{context}: {result:?}");
}

pub(super) fn assert_websocket_bad_request(
    response: Vec<u8>,
    reason_code: &str,
    message_fragment: Option<&str>,
) {
    let response_text = String::from_utf8(response).expect("websocket rejection should be utf-8");
    assert!(response_text.contains("HTTP/1.1 400 Bad Request"));
    let payload = parse_error_envelope_from_http_response(response_text.as_str());
    assert_eq!(payload.error, "bad-request");
    assert_eq!(payload.reason_code, reason_code);
    if let Some(fragment) = message_fragment {
        assert!(payload.message.contains(fragment));
    }
}

pub(super) fn assert_websocket_forbidden(response: Vec<u8>, reason_code: &str) {
    let response_text = String::from_utf8(response).expect("websocket rejection should be utf-8");
    assert!(response_text.contains("HTTP/1.1 403 Forbidden"));
    let payload = parse_error_envelope_from_http_response(response_text.as_str());
    assert_eq!(payload.error, "forbidden");
    assert_eq!(payload.reason_code, reason_code);
}
