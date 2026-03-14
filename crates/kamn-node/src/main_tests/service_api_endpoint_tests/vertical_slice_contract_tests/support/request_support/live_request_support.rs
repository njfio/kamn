use super::bootstrap_support::{assert_server_ok, spawn_api_server};
use super::response_support::{
    parse_created_message, parse_created_payload, parse_ok_payload, state_hash,
};
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
    let signature = request_signature(snapshot, caller_did, nonce, body);
    send_http_request_with_headers(
        bind_addr,
        method,
        path,
        body,
        &signed_headers(caller_did, nonce_text.as_str(), signature.as_str()),
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

fn signed_headers<'a>(
    caller_did: &'a str,
    nonce_text: &'a str,
    signature: &'a str,
) -> [(&'a str, &'a str); 3] {
    [
        ("X-KAMN-Sender-DID", caller_did),
        ("X-KAMN-Request-Nonce", nonce_text),
        ("X-KAMN-Request-Signature", signature),
    ]
}
