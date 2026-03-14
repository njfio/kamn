use super::bootstrap_support::{assert_server_ok, spawn_api_server};
use super::super::super::super::*;
use crate::service_api_endpoint::{
    ServiceApiAgentGetBody, ServiceApiChannelMessagesBody, ServiceApiMessageCreateBody,
    ServiceApiSnapshot, ServiceApiTaskCreateBody,
};

pub(crate) fn send_message(
    snapshot: &ServiceApiSnapshot,
    bind_addr: &str,
    caller_did: &str,
    nonce: u64,
    body: &str,
) -> ServiceApiMessageCreateBody {
    parse_created_message(&signed_request(
        snapshot,
        bind_addr,
        "POST",
        "/v1/messages/send",
        caller_did,
        nonce,
        body,
    ))
}

pub(crate) fn list_mailbox_live(
    snapshot: &ServiceApiSnapshot,
    bind_addr: &str,
    caller_did: &str,
    nonce: u64,
    recipient_did: &str,
) -> ServiceApiChannelMessagesBody {
    let path = format!("/v1/channels/recipient:{recipient_did}/messages");
    parse_ok_payload(&signed_request_without_server(
        snapshot,
        bind_addr,
        "GET",
        path.as_str(),
        caller_did,
        nonce,
        "",
    ))
}

pub(crate) fn query_message_live(
    snapshot: &ServiceApiSnapshot,
    bind_addr: &str,
    caller_did: &str,
    nonce: u64,
    message_id: &str,
) -> Value {
    let path = format!("/v1/messages/{message_id}");
    parse_ok_payload(&signed_request_without_server(
        snapshot,
        bind_addr,
        "GET",
        path.as_str(),
        caller_did,
        nonce,
        "",
    ))
}

pub(crate) fn register_agent_profile(
    snapshot: &ServiceApiSnapshot,
    bind_addr: &str,
    caller_did: &str,
    nonce: u64,
    payload: &str,
) -> ServiceApiAgentGetBody {
    parse_created_payload(&signed_request(
        snapshot,
        bind_addr,
        "POST",
        "/v1/agents/register",
        caller_did,
        nonce,
        payload,
    ))
}

pub(crate) fn create_task(
    snapshot: &ServiceApiSnapshot,
    bind_addr: &str,
    caller_did: &str,
    nonce: u64,
    payload: &str,
) -> ServiceApiTaskCreateBody {
    parse_created_payload(&signed_request(
        snapshot,
        bind_addr,
        "POST",
        "/v1/tasks/create",
        caller_did,
        nonce,
        payload,
    ))
}

pub(crate) fn query_task(
    snapshot: &ServiceApiSnapshot,
    bind_addr: &str,
    caller_did: &str,
    nonce: u64,
    task_id: &str,
) -> Value {
    let path = format!("/v1/tasks/{task_id}");
    parse_ok_payload(&signed_request(
        snapshot,
        bind_addr,
        "GET",
        path.as_str(),
        caller_did,
        nonce,
        "",
    ))
}

fn signed_request(
    snapshot: &ServiceApiSnapshot,
    bind_addr: &str,
    method: &str,
    path: &str,
    caller_did: &str,
    nonce: u64,
    body: &str,
) -> String {
    let server = spawn_api_server(snapshot, bind_addr, 1);
    wait_for_endpoint_ready(bind_addr);
    let response =
        signed_request_without_server(snapshot, bind_addr, method, path, caller_did, nonce, body);
    assert_server_ok(server, "service api endpoint should stop cleanly");
    response
}

fn signed_request_without_server(
    snapshot: &ServiceApiSnapshot,
    bind_addr: &str,
    method: &str,
    path: &str,
    caller_did: &str,
    nonce: u64,
    body: &str,
) -> String {
    let nonce_text = nonce.to_string();
    let signature = service_api_request_signature_for_fields(
        caller_did,
        nonce,
        state_hash(snapshot).as_str(),
        body,
    );
    send_http_request_with_headers(
        bind_addr,
        method,
        path,
        body,
        &[
            ("X-KAMN-Sender-DID", caller_did),
            ("X-KAMN-Request-Nonce", nonce_text.as_str()),
            ("X-KAMN-Request-Signature", signature.as_str()),
        ],
    )
}

fn parse_created_message(response: &str) -> ServiceApiMessageCreateBody {
    assert!(response.contains("HTTP/1.1 202 Accepted"));
    parse_service_api_payload(extract_http_response_body(response))
        .expect("send payload should deserialize")
}

fn parse_created_payload<T>(response: &str) -> T
where
    T: serde::de::DeserializeOwned,
{
    assert!(response.contains("HTTP/1.1 201 Created"));
    parse_service_api_payload(extract_http_response_body(response))
        .expect("created payload should deserialize")
}

fn parse_ok_payload<T>(response: &str) -> T
where
    T: serde::de::DeserializeOwned,
{
    assert!(response.contains("HTTP/1.1 200 OK"));
    parse_service_api_payload(extract_http_response_body(response))
        .expect("ok payload should deserialize")
}

fn state_hash(snapshot: &ServiceApiSnapshot) -> String {
    format!("service-api:{}:{}", snapshot.chain_id.as_str(), snapshot.chain_version.as_str())
}
