use super::super::*;
use crate::service_api_endpoint::{
    ServiceApiChannelMessagesBody, ServiceApiMessageCreateBody, ServiceApiSnapshot,
};
use std::path::{Path, PathBuf};

pub(super) fn build_mailbox_relay_snapshot(api_bind: &str) -> ServiceApiSnapshot {
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

pub(super) fn unique_named_state_file(prefix: &str) -> PathBuf {
    unique_named_path(prefix, "json")
}

pub(super) fn unique_named_relay_spool_file(prefix: &str) -> PathBuf {
    unique_named_path(prefix, "ndjson")
}

fn unique_named_path(prefix: &str, extension: &str) -> PathBuf {
    std::env::temp_dir().join(format!(
        "{prefix}-{}-{}.{}",
        std::process::id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock should be monotonic")
            .as_nanos(),
        extension,
    ))
}

pub(super) fn set_state_file_env(path: &Path) -> EnvVarGuard {
    let path_text = path.to_string_lossy().to_string();
    EnvVarGuard::set("KAMN_SERVICE_API_STATE_FILE", Some(path_text.as_str()))
}

pub(super) fn set_relay_spool_env(path: &Path) -> EnvVarGuard {
    let path_text = path.to_string_lossy().to_string();
    EnvVarGuard::set("KAMN_SERVICE_API_RELAY_SPOOL_FILE", Some(path_text.as_str()))
}

pub(super) fn read_state_json(path: &Path) -> Value {
    let payload = fs::read_to_string(path).expect("state file should remain readable");
    serde_json::from_str(payload.as_str()).expect("state payload should parse")
}

pub(super) fn spawn_api_server(
    snapshot: &ServiceApiSnapshot,
    bind_addr: &str,
    max_requests: u64,
) -> thread::JoinHandle<Result<(), String>> {
    let endpoint_config = ServiceApiEndpointConfig {
        bind_addr: bind_addr.to_owned(),
        max_requests,
        idle_timeout_ms: 2_000,
        body_limit_bytes: DEFAULT_SERVICE_API_BODY_LIMIT_BYTES,
        concurrency_limit: DEFAULT_SERVICE_API_CONCURRENCY_LIMIT,
        rate_limit_per_second: DEFAULT_SERVICE_API_RATE_LIMIT_PER_SECOND,
    };
    let server_snapshot = snapshot.clone();
    thread::spawn(move || serve_service_api_endpoint(&endpoint_config, &server_snapshot))
}

pub(super) fn assert_server_ok(
    server: thread::JoinHandle<Result<(), String>>,
    context: &str,
) {
    let result = server.join().expect("endpoint thread should complete");
    assert!(result.is_ok(), "{context}");
}

pub(super) fn send_signed_request(
    snapshot: &ServiceApiSnapshot,
    bind_addr: &str,
    method: &str,
    path: &str,
    caller_did: &str,
    nonce: u64,
    body: &str,
) -> String {
    let nonce_text = nonce.to_string();
    let signature = request_signature(snapshot, caller_did, nonce, body);
    send_http_request_with_headers(
        bind_addr,
        method,
        path,
        body,
        request_headers(caller_did, nonce_text.as_str(), signature.as_str()).as_slice(),
    )
}

fn request_signature(
    snapshot: &ServiceApiSnapshot,
    caller_did: &str,
    nonce: u64,
    body: &str,
) -> String {
    service_api_request_signature_for_fields(caller_did, nonce, state_hash(snapshot).as_str(), body)
}

fn request_headers<'a>(
    caller_did: &'a str,
    nonce: &'a str,
    signature: &'a str,
) -> [(&'a str, &'a str); 3] {
    [
        ("X-KAMN-Sender-DID", caller_did),
        ("X-KAMN-Request-Nonce", nonce),
        ("X-KAMN-Request-Signature", signature),
    ]
}

pub(super) fn send_message(
    snapshot: &ServiceApiSnapshot,
    bind_addr: &str,
    caller_did: &str,
    nonce: u64,
    body: &str,
) -> ServiceApiMessageCreateBody {
    let response = send_signed_request(snapshot, bind_addr, "POST", "/v1/messages/send", caller_did, nonce, body);
    assert!(response.contains("HTTP/1.1 202 Accepted"));
    parse_service_api_payload(extract_http_response_body(response.as_str()))
        .expect("send payload should deserialize")
}

pub(super) fn list_mailbox(
    snapshot: &ServiceApiSnapshot,
    bind_addr: &str,
    caller_did: &str,
    nonce: u64,
    recipient_did: &str,
) -> ServiceApiChannelMessagesBody {
    let path = format!("/v1/channels/recipient:{recipient_did}/messages");
    let response = send_signed_request(snapshot, bind_addr, "GET", path.as_str(), caller_did, nonce, "");
    assert!(response.contains("HTTP/1.1 200 OK"));
    parse_service_api_payload(extract_http_response_body(response.as_str()))
        .expect("mailbox payload should deserialize")
}

pub(super) fn query_message(
    snapshot: &ServiceApiSnapshot,
    bind_addr: &str,
    caller_did: &str,
    nonce: u64,
    message_id: &str,
) -> Value {
    let path = format!("/v1/messages/{message_id}");
    let response = send_signed_request(snapshot, bind_addr, "GET", path.as_str(), caller_did, nonce, "");
    assert!(response.contains("HTTP/1.1 200 OK"));
    parse_service_api_payload(extract_http_response_body(response.as_str()))
        .expect("message payload should deserialize")
}

pub(super) fn relay_message(
    snapshot: &ServiceApiSnapshot,
    bind_addr: &str,
    caller_did: &str,
    nonce: u64,
    body: &str,
) -> String {
    send_signed_request(snapshot, bind_addr, "POST", "/v1/messages/relay", caller_did, nonce, body)
}

pub(super) fn state_hash(snapshot: &ServiceApiSnapshot) -> String {
    format!(
        "service-api:{}:{}",
        snapshot.chain_id.as_str(),
        snapshot.chain_version.as_str()
    )
}
