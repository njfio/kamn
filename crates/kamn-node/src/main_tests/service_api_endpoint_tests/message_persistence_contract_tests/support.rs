use super::super::*;
use crate::service_api_endpoint::{
    ServiceApiMessageCreateBody, ServiceApiMessageGetBody, ServiceApiSnapshot,
};
use std::path::{Path, PathBuf};

pub(super) fn build_message_snapshot(api_bind: &str) -> ServiceApiSnapshot {
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

pub(super) fn send_persisted_message(
    snapshot: &ServiceApiSnapshot,
    bind_addr: &str,
    sender_did: &str,
    nonce: u64,
    payload: &str,
) -> ServiceApiMessageCreateBody {
    let response = run_single_request_phase(snapshot, bind_addr, "send-phase", |addr| {
        let nonce_text = nonce.to_string();
        let signature = service_api_request_signature_for_fields(
            sender_did,
            nonce,
            service_api_state_hash(snapshot).as_str(),
            payload,
        );
        send_http_request_with_headers(
            addr,
            "POST",
            "/v1/messages/send",
            payload,
            &request_headers(sender_did, nonce_text.as_str(), signature.as_str()),
        )
    });
    assert!(response.contains("HTTP/1.1 202 Accepted"));
    parse_service_api_payload(extract_http_response_body(response.as_str()))
        .expect("send payload should deserialize")
}

pub(super) fn query_persisted_message(
    snapshot: &ServiceApiSnapshot,
    bind_addr: &str,
    sender_did: &str,
    nonce: u64,
    query_path: &str,
) -> ServiceApiMessageGetBody {
    let response = run_single_request_phase(snapshot, bind_addr, "query-phase", |addr| {
        let nonce_text = nonce.to_string();
        let signature = service_api_request_signature_for_fields(
            sender_did,
            nonce,
            service_api_state_hash(snapshot).as_str(),
            "",
        );
        send_http_request_with_headers(
            addr,
            "GET",
            query_path,
            "",
            &request_headers(sender_did, nonce_text.as_str(), signature.as_str()),
        )
    });
    assert!(response.contains("HTTP/1.1 200 OK"));
    parse_service_api_payload(extract_http_response_body(response.as_str()))
        .expect("query payload should deserialize")
}

pub(super) fn unique_named_state_file(prefix: &str) -> PathBuf {
    std::env::temp_dir().join(format!(
        "{prefix}-{}-{}.json",
        std::process::id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock should be monotonic")
            .as_nanos()
    ))
}

pub(super) fn read_state_json(path: &Path) -> Value {
    let payload = fs::read_to_string(path).expect("state file should remain readable");
    serde_json::from_str(payload.as_str()).expect("state file should parse")
}

fn run_single_request_phase<F>(
    snapshot: &ServiceApiSnapshot,
    bind_addr: &str,
    phase: &str,
    request: F,
) -> String
where
    F: FnOnce(&str) -> String,
{
    let endpoint_config = message_endpoint_config(bind_addr);
    let server_snapshot = snapshot.clone();
    let server = thread::spawn(move || serve_service_api_endpoint(&endpoint_config, &server_snapshot));
    wait_for_endpoint_ready(bind_addr);
    let response = request(bind_addr);
    let server_result = server.join().expect("endpoint thread should complete");
    assert!(
        server_result.is_ok(),
        "{phase} service api endpoint should stop cleanly after request budget"
    );
    response
}

fn message_endpoint_config(bind_addr: &str) -> ServiceApiEndpointConfig {
    ServiceApiEndpointConfig {
        bind_addr: bind_addr.to_owned(),
        max_requests: 1,
        idle_timeout_ms: 2_000,
        body_limit_bytes: DEFAULT_SERVICE_API_BODY_LIMIT_BYTES,
        concurrency_limit: DEFAULT_SERVICE_API_CONCURRENCY_LIMIT,
        rate_limit_per_second: DEFAULT_SERVICE_API_RATE_LIMIT_PER_SECOND,
    }
}

fn request_headers<'a>(
    sender_did: &'a str,
    nonce: &'a str,
    signature: &'a str,
) -> [(&'a str, &'a str); 3] {
    [
        ("X-KAMN-Sender-DID", sender_did),
        ("X-KAMN-Request-Nonce", nonce),
        ("X-KAMN-Request-Signature", signature),
    ]
}

fn service_api_state_hash(snapshot: &ServiceApiSnapshot) -> String {
    format!(
        "service-api:{}:{}",
        snapshot.chain_id.as_str(),
        snapshot.chain_version.as_str()
    )
}
