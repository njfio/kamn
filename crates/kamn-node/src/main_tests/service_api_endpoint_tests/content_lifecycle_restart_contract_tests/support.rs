use super::super::*;
use crate::service_api_endpoint::ServiceApiSnapshot;
use std::path::{Path, PathBuf};

pub(super) fn build_content_snapshot(api_bind: &str) -> ServiceApiSnapshot {
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
    std::env::temp_dir().join(format!(
        "{prefix}-{}-{}.json",
        std::process::id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock should be monotonic")
            .as_nanos()
    ))
}

pub(super) fn set_state_file_env(path: &Path) -> EnvVarGuard {
    let path_text = path.to_string_lossy().to_string();
    EnvVarGuard::set("KAMN_SERVICE_API_STATE_FILE", Some(path_text.as_str()))
}

pub(super) fn read_state_json(path: &Path) -> Value {
    let payload =
        fs::read_to_string(path).expect("content lifecycle state file should remain readable");
    serde_json::from_str(payload.as_str()).expect("state payload should parse")
}

pub(super) fn register_content(
    snapshot: &ServiceApiSnapshot,
    bind_addr: &str,
    caller_did: &str,
    nonce: u64,
    payload: &str,
) -> Value {
    let response = signed_request(
        snapshot,
        bind_addr,
        SignedRequest {
            max_requests: 1,
            method: "POST",
            path: "/v1/content/register",
            caller_did,
            nonce,
            body: payload,
        },
    );
    assert!(response.contains("HTTP/1.1 201 Created"));
    parse_service_api_payload(extract_http_response_body(response.as_str()))
        .expect("content register payload should deserialize")
}

pub(super) fn expire_content(
    snapshot: &ServiceApiSnapshot,
    bind_addr: &str,
    caller_did: &str,
    nonce: u64,
    content_id: &str,
) -> Value {
    let response = signed_request(
        snapshot,
        bind_addr,
        SignedRequest {
            max_requests: 1,
            method: "POST",
            path: format!("/v1/content/{content_id}/expire").as_str(),
            caller_did,
            nonce,
            body: "",
        },
    );
    assert!(response.contains("HTTP/1.1 200 OK"));
    parse_service_api_payload(extract_http_response_body(response.as_str()))
        .expect("expire payload should deserialize")
}

pub(super) fn query_content(
    snapshot: &ServiceApiSnapshot,
    bind_addr: &str,
    caller_did: &str,
    nonce: u64,
    content_id: &str,
) -> Value {
    let response = signed_request(
        snapshot,
        bind_addr,
        SignedRequest {
            max_requests: 1,
            method: "GET",
            path: format!("/v1/content/{content_id}").as_str(),
            caller_did,
            nonce,
            body: "",
        },
    );
    assert!(response.contains("HTTP/1.1 200 OK"));
    parse_service_api_payload(extract_http_response_body(response.as_str()))
        .expect("query payload should deserialize")
}

pub(super) fn tombstone_content(
    snapshot: &ServiceApiSnapshot,
    bind_addr: &str,
    caller_did: &str,
    nonce: u64,
    content_id: &str,
) -> Value {
    let response = signed_request(
        snapshot,
        bind_addr,
        SignedRequest {
            max_requests: 1,
            method: "POST",
            path: format!("/v1/content/{content_id}/tombstone").as_str(),
            caller_did,
            nonce,
            body: "",
        },
    );
    assert!(response.contains("HTTP/1.1 200 OK"));
    parse_service_api_payload(extract_http_response_body(response.as_str()))
        .expect("tombstone payload should deserialize")
}

pub(super) fn query_missing_content(
    snapshot: &ServiceApiSnapshot,
    bind_addr: &str,
    caller_did: &str,
    nonce: u64,
    content_id: &str,
) -> ServiceApiErrorEnvelope {
    let response = signed_request(
        snapshot,
        bind_addr,
        SignedRequest {
            max_requests: 1,
            method: "GET",
            path: format!("/v1/content/{content_id}").as_str(),
            caller_did,
            nonce,
            body: "",
        },
    );
    assert!(response.contains("HTTP/1.1 404 Not Found"));
    parse_error_envelope_from_http_response(response.as_str())
}

struct SignedRequest<'a> {
    max_requests: usize,
    method: &'a str,
    path: &'a str,
    caller_did: &'a str,
    nonce: u64,
    body: &'a str,
}

fn signed_request(
    snapshot: &ServiceApiSnapshot,
    bind_addr: &str,
    request: SignedRequest<'_>,
) -> String {
    with_api_server(snapshot, bind_addr, request.max_requests, |addr| {
        let signature = service_api_request_signature_for_fields(
            request.caller_did,
            request.nonce,
            state_hash(snapshot).as_str(),
            request.body,
        );
        let nonce_text = request.nonce.to_string();
        send_http_request_with_headers(
            addr,
            request.method,
            request.path,
            request.body,
            &[
                ("X-KAMN-Sender-DID", request.caller_did),
                ("X-KAMN-Request-Nonce", nonce_text.as_str()),
                ("X-KAMN-Request-Signature", signature.as_str()),
            ],
        )
    })
}

fn with_api_server<T, F>(
    snapshot: &ServiceApiSnapshot,
    bind_addr: &str,
    max_requests: usize,
    request: F,
) -> T
where
    F: FnOnce(&str) -> T,
{
    let endpoint_config = ServiceApiEndpointConfig {
        bind_addr: bind_addr.to_owned(),
        max_requests: max_requests as u64,
        idle_timeout_ms: 2_000,
        body_limit_bytes: DEFAULT_SERVICE_API_BODY_LIMIT_BYTES,
        concurrency_limit: DEFAULT_SERVICE_API_CONCURRENCY_LIMIT,
        rate_limit_per_second: DEFAULT_SERVICE_API_RATE_LIMIT_PER_SECOND,
    };
    let server_snapshot = snapshot.clone();
    let server =
        thread::spawn(move || serve_service_api_endpoint(&endpoint_config, &server_snapshot));
    wait_for_endpoint_ready(bind_addr);
    let response = request(bind_addr);
    let server_result = server.join().expect("endpoint thread should complete");
    assert!(
        server_result.is_ok(),
        "service api endpoint should stop cleanly"
    );
    response
}

fn state_hash(snapshot: &ServiceApiSnapshot) -> String {
    format!(
        "service-api:{}:{}",
        snapshot.chain_id.as_str(),
        snapshot.chain_version.as_str()
    )
}
