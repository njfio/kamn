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
        RawSignedRequest {
            max_requests: 1,
            method: "POST",
            path: "/v1/messages/send",
            sender_did,
            nonce,
            body: payload,
            extra_headers: &[],
        },
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
        RawSignedRequest {
            max_requests: 1,
            method: "GET",
            path: format!("/v1/channels/{channel_id}/messages").as_str(),
            sender_did,
            nonce,
            body: "",
            extra_headers: &[],
        },
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
        RawSignedRequest {
            max_requests: 1,
            method: "POST",
            path: "/v1/channels/create",
            sender_did: caller_did,
            nonce,
            body: payload,
            extra_headers: &[],
        },
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
        RawSignedRequest {
            max_requests: 1,
            method: "GET",
            path: format!("/v1/agents/{target_did}").as_str(),
            sender_did: caller_did,
            nonce,
            body: "",
            extra_headers: &[],
        },
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
        RawSignedRequest {
            max_requests: 1,
            method: "POST",
            path: "/v1/agents/register",
            sender_did: caller_did,
            nonce,
            body: payload,
            extra_headers: &[],
        },
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
        RawSignedRequest {
            max_requests: 1,
            method: "POST",
            path: "/v1/agents/search",
            sender_did: caller_did,
            nonce,
            body: payload,
            extra_headers: &[("X-KAMN-Authz-Scope", "agents:read")],
        },
    ))
}

pub(super) struct RawSignedRequest<'a> {
    pub(super) max_requests: usize,
    pub(super) method: &'a str,
    pub(super) path: &'a str,
    pub(super) sender_did: &'a str,
    pub(super) nonce: u64,
    pub(super) body: &'a str,
    pub(super) extra_headers: &'a [(&'a str, &'a str)],
}

pub(super) fn raw_signed_request(
    snapshot: &ServiceApiSnapshot,
    bind_addr: &str,
    request: RawSignedRequest<'_>,
) -> String {
    with_api_server(snapshot, bind_addr, request.max_requests, |addr| {
        let nonce_text = request.nonce.to_string();
        let signature = service_api_request_signature_for_fields(
            request.sender_did,
            request.nonce,
            state_hash(snapshot).as_str(),
            request.body,
        );
        let mut headers = vec![
            ("X-KAMN-Sender-DID", request.sender_did),
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
