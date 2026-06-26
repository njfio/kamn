use super::super::*;
use crate::service_api_endpoint::ServiceApiSnapshot;

pub(super) struct IngressServer {
    pub bind_addr: String,
    pub server: thread::JoinHandle<Result<(), String>>,
}

pub(super) fn build_ingress_snapshot(api_bind: &str) -> ServiceApiSnapshot {
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

pub(super) fn spawn_ingress_server(
    snapshot: &ServiceApiSnapshot,
    max_requests: u64,
    idle_timeout_ms: u64,
    body_limit_bytes: u64,
    concurrency_limit: u64,
    rate_limit_per_second: u64,
) -> IngressServer {
    let bind_addr = reserve_loopback_addr();
    let endpoint_config = ServiceApiEndpointConfig {
        bind_addr: bind_addr.clone(),
        max_requests,
        idle_timeout_ms,
        body_limit_bytes,
        concurrency_limit,
        rate_limit_per_second,
    };
    let server_snapshot = snapshot.clone();
    let server =
        thread::spawn(move || serve_service_api_endpoint(&endpoint_config, &server_snapshot));
    wait_for_endpoint_ready(bind_addr.as_str());
    IngressServer { bind_addr, server }
}

pub(super) fn assert_server_ok(server: thread::JoinHandle<Result<(), String>>, context: &str) {
    let result = server.join().expect("endpoint thread should complete");
    assert!(result.is_ok(), "{context}");
}

pub(super) fn send_signed_message_request(
    snapshot: &ServiceApiSnapshot,
    bind_addr: &str,
    sender_did: &str,
    nonce: u64,
    body: &str,
) -> String {
    send_signed_request(
        snapshot,
        bind_addr,
        "POST",
        "/v1/messages/send",
        sender_did,
        nonce,
        body,
    )
}

pub(super) fn send_signed_message_request_with_signature(
    bind_addr: &str,
    sender_did: &str,
    nonce: u64,
    signature: &str,
    body: &str,
) -> String {
    let nonce_text = nonce.to_string();
    send_http_request_with_headers(
        bind_addr,
        "POST",
        "/v1/messages/send",
        body,
        &[
            ("X-KAMN-Sender-DID", sender_did),
            ("X-KAMN-Request-Nonce", nonce_text.as_str()),
            ("X-KAMN-Request-Signature", signature),
        ],
    )
}

pub(super) fn send_signed_request(
    snapshot: &ServiceApiSnapshot,
    bind_addr: &str,
    method: &str,
    path: &str,
    sender_did: &str,
    nonce: u64,
    body: &str,
) -> String {
    let nonce_text = nonce.to_string();
    let signature = request_signature(snapshot, sender_did, nonce, body);
    send_http_request_with_headers(
        bind_addr,
        method,
        path,
        body,
        &[
            ("X-KAMN-Sender-DID", sender_did),
            ("X-KAMN-Request-Nonce", nonce_text.as_str()),
            ("X-KAMN-Request-Signature", signature.as_str()),
        ],
    )
}

fn request_signature(
    snapshot: &ServiceApiSnapshot,
    sender_did: &str,
    nonce: u64,
    body: &str,
) -> String {
    service_api_request_signature_for_fields(sender_did, nonce, state_hash(snapshot).as_str(), body)
}

pub(super) fn state_hash(snapshot: &ServiceApiSnapshot) -> String {
    format!(
        "service-api:{}:{}",
        snapshot.chain_id.as_str(),
        snapshot.chain_version.as_str()
    )
}
