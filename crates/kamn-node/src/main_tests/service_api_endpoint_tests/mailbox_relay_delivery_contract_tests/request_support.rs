use super::super::*;
use super::support::state_hash;
use crate::service_api_endpoint::{
    ServiceApiChannelMessagesBody, ServiceApiMessageCreateBody, ServiceApiSnapshot,
};

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
    let response = send_signed_request(
        snapshot,
        bind_addr,
        "POST",
        "/v1/messages/send",
        caller_did,
        nonce,
        body,
    );
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
    let response = send_signed_request(
        snapshot,
        bind_addr,
        "GET",
        path.as_str(),
        caller_did,
        nonce,
        "",
    );
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
    let response = send_signed_request(
        snapshot,
        bind_addr,
        "GET",
        path.as_str(),
        caller_did,
        nonce,
        "",
    );
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
    send_signed_request(
        snapshot,
        bind_addr,
        "POST",
        "/v1/messages/relay",
        caller_did,
        nonce,
        body,
    )
}
