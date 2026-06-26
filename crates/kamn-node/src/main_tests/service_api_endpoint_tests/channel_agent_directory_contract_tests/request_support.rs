use super::super::*;
use super::state_support::state_hash;
use crate::service_api_endpoint::{
    ServiceApiAgentGetBody, ServiceApiChannelCreateBody, ServiceApiChannelMessagesBody,
    ServiceApiMessageCreateBody, ServiceApiSnapshot,
};

pub(super) fn send_channel_message(
    snapshot: &ServiceApiSnapshot,
    bind_addr: &str,
    sender_did: &str,
    nonce: u64,
    payload: &str,
) -> ServiceApiMessageCreateBody {
    parse_created_message(&raw_signed_request(
        snapshot,
        bind_addr,
        1,
        "POST",
        "/v1/messages/send",
        sender_did,
        nonce,
        payload,
        &[],
    ))
}

pub(super) fn list_channel_messages(
    snapshot: &ServiceApiSnapshot,
    bind_addr: &str,
    sender_did: &str,
    nonce: u64,
    channel_id: &str,
) -> ServiceApiChannelMessagesBody {
    parse_channel_messages(&raw_signed_request(
        snapshot,
        bind_addr,
        1,
        "GET",
        format!("/v1/channels/{channel_id}/messages").as_str(),
        sender_did,
        nonce,
        "",
        &[],
    ))
}

pub(super) fn create_channel(
    snapshot: &ServiceApiSnapshot,
    bind_addr: &str,
    caller_did: &str,
    nonce: u64,
    payload: &str,
) -> ServiceApiChannelCreateBody {
    parse_created_channel(&raw_signed_request(
        snapshot,
        bind_addr,
        1,
        "POST",
        "/v1/channels/create",
        caller_did,
        nonce,
        payload,
        &[],
    ))
}

pub(super) fn query_agent_profile(
    snapshot: &ServiceApiSnapshot,
    bind_addr: &str,
    caller_did: &str,
    nonce: u64,
    target_did: &str,
) -> ServiceApiAgentGetBody {
    parse_agent_profile(&raw_signed_request(
        snapshot,
        bind_addr,
        1,
        "GET",
        format!("/v1/agents/{target_did}").as_str(),
        caller_did,
        nonce,
        "",
        &[],
    ))
}

pub(super) fn register_agent(
    snapshot: &ServiceApiSnapshot,
    bind_addr: &str,
    caller_did: &str,
    nonce: u64,
    payload: &str,
) -> ServiceApiAgentGetBody {
    parse_registered_agent(&raw_signed_request(
        snapshot,
        bind_addr,
        1,
        "POST",
        "/v1/agents/register",
        caller_did,
        nonce,
        payload,
        &[],
    ))
}

pub(super) fn search_agents(
    snapshot: &ServiceApiSnapshot,
    bind_addr: &str,
    caller_did: &str,
    nonce: u64,
    payload: &str,
) -> Vec<ServiceApiAgentGetBody> {
    parse_agent_search_results(&raw_signed_request(
        snapshot,
        bind_addr,
        1,
        "POST",
        "/v1/agents/search",
        caller_did,
        nonce,
        payload,
        &[("X-KAMN-Authz-Scope", "agents:read")],
    ))
}

pub(super) fn raw_signed_request(
    snapshot: &ServiceApiSnapshot,
    bind_addr: &str,
    max_requests: usize,
    method: &str,
    path: &str,
    sender_did: &str,
    nonce: u64,
    body: &str,
    extra_headers: &[(&str, &str)],
) -> String {
    with_api_server(snapshot, bind_addr, max_requests, |addr| {
        let nonce_text = nonce.to_string();
        let signature = service_api_request_signature_for_fields(
            sender_did,
            nonce,
            state_hash(snapshot).as_str(),
            body,
        );
        let mut headers = vec![
            ("X-KAMN-Sender-DID", sender_did),
            ("X-KAMN-Request-Nonce", nonce_text.as_str()),
            ("X-KAMN-Request-Signature", signature.as_str()),
        ];
        headers.extend_from_slice(extra_headers);
        send_http_request_with_headers(addr, method, path, body, headers.as_slice())
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
    let endpoint_config = endpoint_config(bind_addr, max_requests);
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

fn endpoint_config(bind_addr: &str, max_requests: usize) -> ServiceApiEndpointConfig {
    ServiceApiEndpointConfig {
        bind_addr: bind_addr.to_owned(),
        max_requests: max_requests as u64,
        idle_timeout_ms: 2_000,
        body_limit_bytes: DEFAULT_SERVICE_API_BODY_LIMIT_BYTES,
        concurrency_limit: DEFAULT_SERVICE_API_CONCURRENCY_LIMIT,
        rate_limit_per_second: DEFAULT_SERVICE_API_RATE_LIMIT_PER_SECOND,
    }
}

fn parse_created_message(response: &str) -> ServiceApiMessageCreateBody {
    assert!(response.contains("HTTP/1.1 202 Accepted"));
    parse_service_api_payload(extract_http_response_body(response))
        .expect("send payload should deserialize")
}

fn parse_channel_messages(response: &str) -> ServiceApiChannelMessagesBody {
    assert!(response.contains("HTTP/1.1 200 OK"));
    parse_service_api_payload(extract_http_response_body(response))
        .expect("channel list payload should deserialize")
}

fn parse_created_channel(response: &str) -> ServiceApiChannelCreateBody {
    assert!(response.contains("HTTP/1.1 201 Created"));
    parse_service_api_payload(extract_http_response_body(response))
        .expect("channel create payload should deserialize")
}

fn parse_agent_profile(response: &str) -> ServiceApiAgentGetBody {
    assert!(response.contains("HTTP/1.1 200 OK"));
    parse_service_api_payload(extract_http_response_body(response))
        .expect("agent query payload should deserialize")
}

fn parse_registered_agent(response: &str) -> ServiceApiAgentGetBody {
    assert!(response.contains("HTTP/1.1 201 Created"));
    parse_service_api_payload(extract_http_response_body(response))
        .expect("registration payload should deserialize")
}

fn parse_agent_search_results(response: &str) -> Vec<ServiceApiAgentGetBody> {
    assert!(response.contains("HTTP/1.1 200 OK"));
    parse_service_api_payload(extract_http_response_body(response))
        .expect("search payload should deserialize")
}
