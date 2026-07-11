use super::*;
use crate::service_api_endpoint::{ServiceApiAgentGetBody, ServiceApiTaskCreateBody};

pub(crate) fn register_agent_profile(
    snapshot: &ServiceApiSnapshot,
    bind_addr: &str,
    caller_did: &str,
    nonce: u64,
    payload: &str,
) -> ServiceApiAgentGetBody {
    let response = raw_signed_request(
        snapshot,
        bind_addr,
        SignedRequest {
            max_requests: 1,
            method: "POST",
            path: "/v1/agents/register",
            caller_did,
            nonce,
            body: payload,
            extra_headers: &[],
        },
    );
    assert!(response.contains("HTTP/1.1 201 Created"));
    parse_service_api_payload(extract_http_response_body(response.as_str()))
        .expect("agent registration payload should deserialize")
}

pub(crate) fn create_task(
    snapshot: &ServiceApiSnapshot,
    bind_addr: &str,
    caller_did: &str,
    nonce: u64,
    payload: &str,
) -> ServiceApiTaskCreateBody {
    let canonical_payload = canonical_task_payload(snapshot, bind_addr, caller_did, nonce, payload);
    let response = signed_request(
        snapshot,
        bind_addr,
        SignedRequest {
            max_requests: 1,
            method: "POST",
            path: "/v1/tasks/create",
            caller_did,
            nonce,
            body: canonical_payload.as_str(),
            extra_headers: &[],
        },
    );
    assert!(response.contains("HTTP/1.1 201 Created"));
    parse_service_api_payload(extract_http_response_body(response.as_str()))
        .expect("task create payload should deserialize")
}

pub(crate) fn accept_task(
    snapshot: &ServiceApiSnapshot,
    bind_addr: &str,
    caller_did: &str,
    nonce: u64,
    task_id: &str,
) {
    let path = format!("/v1/tasks/{task_id}/accept");
    assert_task_transition(
        snapshot,
        bind_addr,
        caller_did,
        nonce,
        path.as_str(),
        r#"{"idempotency_key":"accept-task"}"#,
    );
}

pub(crate) fn complete_task(
    snapshot: &ServiceApiSnapshot,
    bind_addr: &str,
    caller_did: &str,
    nonce: u64,
    task_id: &str,
) {
    let path = format!("/v1/tasks/{task_id}/complete");
    assert_task_transition(
        snapshot,
        bind_addr,
        caller_did,
        nonce,
        path.as_str(),
        r#"{"idempotency_key":"complete-for-escrow","completion_evidence_digest":"bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb"}"#,
    );
}

pub(crate) fn query_task(
    snapshot: &ServiceApiSnapshot,
    bind_addr: &str,
    caller_did: &str,
    nonce: u64,
    task_id: &str,
) -> Value {
    let path = format!("/v1/tasks/{task_id}");
    let response = raw_signed_request(
        snapshot,
        bind_addr,
        SignedRequest {
            max_requests: 1,
            method: "GET",
            path: path.as_str(),
            caller_did,
            nonce,
            body: "",
            extra_headers: &[],
        },
    );
    assert!(response.contains("HTTP/1.1 200 OK"), "{response}");
    parse_service_api_payload(extract_http_response_body(response.as_str()))
        .expect("task query payload should deserialize")
}

fn canonical_task_payload(
    snapshot: &ServiceApiSnapshot,
    bind_addr: &str,
    caller_did: &str,
    nonce: u64,
    payload: &str,
) -> String {
    if payload.contains("\"provider_did\"") {
        return payload.to_owned();
    }
    let provider = register_agent_profile(
        snapshot,
        bind_addr,
        caller_did,
        nonce - 1,
        r#"{"agent_type":"provider","model_family":"test","capabilities":["tasks"]}"#,
    );
    serde_json::json!({
        "provider_did": provider.did,
        "transaction_id": format!("transaction-{nonce}"),
        "terms_digest": "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
        "idempotency_key": format!("create-{nonce}"),
        "description": payload,
    })
    .to_string()
}

fn assert_task_transition(
    snapshot: &ServiceApiSnapshot,
    bind_addr: &str,
    caller_did: &str,
    nonce: u64,
    path: &str,
    body: &str,
) {
    let response = raw_signed_request(
        snapshot,
        bind_addr,
        SignedRequest {
            max_requests: 1,
            method: "POST",
            path,
            caller_did,
            nonce,
            body,
            extra_headers: &[],
        },
    );
    assert!(response.contains("HTTP/1.1 200 OK"), "{response}");
}
