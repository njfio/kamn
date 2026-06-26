use super::super::super::*;
use crate::service_api_endpoint::{
    ServiceApiAgentGetBody, ServiceApiSnapshot, ServiceApiTaskCreateBody,
};

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
    let response = signed_request(
        snapshot,
        bind_addr,
        SignedRequest {
            max_requests: 1,
            method: "POST",
            path: "/v1/tasks/create",
            caller_did,
            nonce,
            body: payload,
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
    let response = signed_request(
        snapshot,
        bind_addr,
        SignedRequest {
            max_requests: 1,
            method: "POST",
            path: format!("/v1/tasks/{task_id}/accept").as_str(),
            caller_did,
            nonce,
            body: "",
            extra_headers: &[],
        },
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
        SignedRequest {
            max_requests: 1,
            method: "GET",
            path: format!("/v1/tasks/{task_id}").as_str(),
            caller_did,
            nonce,
            body: "",
            extra_headers: &[],
        },
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
    let response = signed_request(
        snapshot,
        bind_addr,
        SignedRequest {
            max_requests: 1,
            method: "POST",
            path: "/v1/escrow/fund",
            caller_did,
            nonce,
            body: payload,
            extra_headers: &[],
        },
    );
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
        SignedRequest {
            max_requests: 1,
            method: "POST",
            path: format!("/v1/escrow/{escrow_id}/release").as_str(),
            caller_did,
            nonce,
            body: "",
            extra_headers: &[],
        },
    );
    assert!(response.contains("HTTP/1.1 200 OK"));
    parse_service_api_payload(extract_http_response_body(response.as_str()))
        .expect("escrow release payload should deserialize")
}

pub(crate) struct SignedRequest<'a> {
    pub(crate) max_requests: usize,
    pub(crate) method: &'a str,
    pub(crate) path: &'a str,
    pub(crate) caller_did: &'a str,
    pub(crate) nonce: u64,
    pub(crate) body: &'a str,
    pub(crate) extra_headers: &'a [(&'a str, &'a str)],
}

pub(crate) fn raw_signed_request(
    snapshot: &ServiceApiSnapshot,
    bind_addr: &str,
    request: SignedRequest<'_>,
) -> String {
    super::state_support::with_api_server(snapshot, bind_addr, request.max_requests, |addr| {
        let (nonce_text, signature) =
            build_signed_header_values(snapshot, request.caller_did, request.nonce, request.body);
        let mut headers = vec![
            ("X-KAMN-Sender-DID", request.caller_did),
            ("X-KAMN-Request-Nonce", nonce_text.as_str()),
            ("X-KAMN-Request-Signature", signature.as_str()),
        ];
        headers.extend_from_slice(request.extra_headers);
        send_http_request_with_headers(
            addr,
            request.method,
            request.path,
            request.body,
            headers.as_slice(),
        )
    })
}

fn signed_request(
    snapshot: &ServiceApiSnapshot,
    bind_addr: &str,
    request: SignedRequest<'_>,
) -> String {
    raw_signed_request(snapshot, bind_addr, request)
}

fn build_signed_header_values(
    snapshot: &ServiceApiSnapshot,
    caller_did: &str,
    nonce: u64,
    body: &str,
) -> (String, String) {
    let signature = service_api_request_signature_for_fields(
        caller_did,
        nonce,
        super::state_support::state_hash(snapshot).as_str(),
        body,
    );
    (nonce.to_string(), signature)
}
