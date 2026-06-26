use super::super::*;
use crate::service_api_endpoint::ServiceApiSnapshot;
use std::path::{Path, PathBuf};

const LIVE_SOLANA_DEVNET_RPC_URL: &str = "https://api.devnet.solana.com";

pub(super) fn build_bridge_snapshot(api_bind: &str) -> ServiceApiSnapshot {
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

pub(super) fn set_live_solana_bridge_rpc_url_env(value: Option<&str>) -> EnvVarGuard {
    EnvVarGuard::set("KAMN_SERVICE_API_LIVE_SOLANA_BRIDGE_RPC_URL", value)
}

pub(super) fn set_default_live_solana_bridge_rpc_url_env() -> EnvVarGuard {
    set_live_solana_bridge_rpc_url_env(Some(LIVE_SOLANA_DEVNET_RPC_URL))
}

pub(super) fn read_state_json(path: &Path) -> Value {
    let payload = fs::read_to_string(path).expect("bridge state file should remain readable");
    serde_json::from_str(payload.as_str()).expect("state payload should parse")
}

pub(super) fn submit_bridge(
    snapshot: &ServiceApiSnapshot,
    bind_addr: &str,
    caller_did: &str,
    nonce: u64,
    payload: &str,
) -> Value {
    let response = signed_request(
        snapshot,
        bind_addr,
        1,
        "POST",
        "/v1/bridge/submit",
        caller_did,
        nonce,
        payload,
    );
    assert!(response.contains("HTTP/1.1 202 Accepted"));
    parse_service_api_payload(extract_http_response_body(response.as_str()))
        .expect("bridge submit payload should deserialize")
}

pub(super) fn forward_bridge(
    snapshot: &ServiceApiSnapshot,
    bind_addr: &str,
    caller_did: &str,
    nonce: u64,
    bridge_id: &str,
) -> Value {
    let response = signed_request(
        snapshot,
        bind_addr,
        1,
        "POST",
        format!("/v1/bridge/{bridge_id}/forward").as_str(),
        caller_did,
        nonce,
        "",
    );
    assert!(response.contains("HTTP/1.1 200 OK"));
    parse_service_api_payload(extract_http_response_body(response.as_str()))
        .expect("bridge forward payload should deserialize")
}

pub(super) fn query_bridge(
    snapshot: &ServiceApiSnapshot,
    bind_addr: &str,
    caller_did: &str,
    nonce: u64,
    bridge_id: &str,
) -> Value {
    let response = signed_request(
        snapshot,
        bind_addr,
        1,
        "GET",
        format!("/v1/bridge/{bridge_id}").as_str(),
        caller_did,
        nonce,
        "",
    );
    assert!(response.contains("HTTP/1.1 200 OK"));
    parse_service_api_payload(extract_http_response_body(response.as_str()))
        .expect("bridge query payload should deserialize")
}

pub(super) fn query_missing_bridge(
    snapshot: &ServiceApiSnapshot,
    bind_addr: &str,
    caller_did: &str,
    nonce: u64,
    bridge_id: &str,
) -> ServiceApiErrorEnvelope {
    let response = signed_request(
        snapshot,
        bind_addr,
        1,
        "GET",
        format!("/v1/bridge/{bridge_id}").as_str(),
        caller_did,
        nonce,
        "",
    );
    assert!(response.contains("HTTP/1.1 404 Not Found"));
    parse_error_envelope_from_http_response(response.as_str())
}

fn signed_request(
    snapshot: &ServiceApiSnapshot,
    bind_addr: &str,
    max_requests: usize,
    method: &str,
    path: &str,
    caller_did: &str,
    nonce: u64,
    body: &str,
) -> String {
    with_api_server(snapshot, bind_addr, max_requests, |addr| {
        let signature = service_api_request_signature_for_fields(
            caller_did,
            nonce,
            state_hash(snapshot).as_str(),
            body,
        );
        let nonce_text = nonce.to_string();
        send_http_request_with_headers(
            addr,
            method,
            path,
            body,
            &[
                ("X-KAMN-Sender-DID", caller_did),
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
        idle_timeout_ms: 10_000,
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
