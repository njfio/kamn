use super::super::super::*;
use crate::service_api_endpoint::{ServiceApiSnapshot, ServiceApiTaskCreateBody};

pub(crate) fn build_task_escrow_snapshot(api_bind: &str) -> ServiceApiSnapshot {
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

pub(crate) fn create_task(
    snapshot: &ServiceApiSnapshot,
    bind_addr: &str,
    caller_did: &str,
    nonce: u64,
    payload: &str,
) -> ServiceApiTaskCreateBody {
    let response = raw_create_task_response(snapshot, bind_addr, caller_did, nonce, payload);
    assert!(response.contains("HTTP/1.1 201 Created"));
    parse_service_api_payload(extract_http_response_body(response.as_str()))
        .expect("task create payload should deserialize")
}

pub(crate) fn raw_create_task_response(
    snapshot: &ServiceApiSnapshot,
    bind_addr: &str,
    caller_did: &str,
    nonce: u64,
    payload: &str,
) -> String {
    signed_request(
        snapshot,
        bind_addr,
        1,
        "POST",
        "/v1/tasks/create",
        caller_did,
        nonce,
        payload,
    )
}

pub(crate) fn accept_task(
    snapshot: &ServiceApiSnapshot,
    bind_addr: &str,
    caller_did: &str,
    nonce: u64,
    task_id: &str,
) {
    let response = signed_request(
        snapshot,
        bind_addr,
        1,
        "POST",
        format!("/v1/tasks/{task_id}/accept").as_str(),
        caller_did,
        nonce,
        "",
    );
    assert!(response.contains("HTTP/1.1 200 OK"));
}

pub(crate) fn query_task(
    snapshot: &ServiceApiSnapshot,
    bind_addr: &str,
    caller_did: &str,
    nonce: u64,
    task_id: &str,
) -> Value {
    let response = signed_request(
        snapshot,
        bind_addr,
        1,
        "GET",
        format!("/v1/tasks/{task_id}").as_str(),
        caller_did,
        nonce,
        "",
    );
    assert!(response.contains("HTTP/1.1 200 OK"));
    parse_service_api_payload(extract_http_response_body(response.as_str()))
        .expect("task query payload should deserialize")
}

pub(crate) fn fund_escrow(
    snapshot: &ServiceApiSnapshot,
    bind_addr: &str,
    caller_did: &str,
    nonce: u64,
    payload: &str,
) -> Value {
    let response =
        signed_request(snapshot, bind_addr, 1, "POST", "/v1/escrow/fund", caller_did, nonce, payload);
    assert!(response.contains("HTTP/1.1 200 OK"));
    parse_service_api_payload(extract_http_response_body(response.as_str()))
        .expect("escrow fund payload should deserialize")
}

pub(crate) fn release_escrow(
    snapshot: &ServiceApiSnapshot,
    bind_addr: &str,
    caller_did: &str,
    nonce: u64,
    escrow_id: &str,
) -> Value {
    let response = signed_request(
        snapshot,
        bind_addr,
        1,
        "POST",
        format!("/v1/escrow/{escrow_id}/release").as_str(),
        caller_did,
        nonce,
        "",
    );
    assert!(response.contains("HTTP/1.1 200 OK"));
    parse_service_api_payload(extract_http_response_body(response.as_str()))
        .expect("escrow release payload should deserialize")
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
        let signature =
            service_api_request_signature_for_fields(caller_did, nonce, state_hash(snapshot).as_str(), body);
        send_http_request_with_headers(
            addr,
            method,
            path,
            body,
            &[
                ("X-KAMN-Sender-DID", caller_did),
                ("X-KAMN-Request-Nonce", nonce.to_string().as_str()),
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
    let server = thread::spawn(move || serve_service_api_endpoint(&endpoint_config, &server_snapshot));
    wait_for_endpoint_ready(bind_addr);
    let response = request(bind_addr);
    let server_result = server.join().expect("endpoint thread should complete");
    assert!(server_result.is_ok(), "service api endpoint should stop cleanly");
    response
}

fn state_hash(snapshot: &ServiceApiSnapshot) -> String {
    format!(
        "service-api:{}:{}",
        snapshot.chain_id.as_str(),
        snapshot.chain_version.as_str()
    )
}
