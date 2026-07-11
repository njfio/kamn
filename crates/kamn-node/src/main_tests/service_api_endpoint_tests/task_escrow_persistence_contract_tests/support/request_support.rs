use super::super::super::*;
use crate::service_api_endpoint::ServiceApiSnapshot;

#[path = "request_support/escrow_support.rs"]
mod escrow_support;
#[path = "request_support/task_support.rs"]
mod task_support;

pub(crate) use escrow_support::{
    fund_escrow, release_escrow, release_escrow_response, release_escrow_response_with_key,
};
pub(crate) use task_support::{
    accept_task, complete_task, create_task, query_task, register_agent_profile,
};

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

pub(crate) fn authorized_signed_request(
    snapshot: &ServiceApiSnapshot,
    bind_addr: &str,
    request: SignedRequest<'_>,
) -> String {
    super::authorization_fixture::provision_request_grant(&request);
    raw_signed_request(snapshot, bind_addr, request)
}

pub(crate) fn provision_signed_request_grant(request: &SignedRequest<'_>) {
    super::authorization_fixture::provision_request_grant(request);
}

fn signed_request(
    snapshot: &ServiceApiSnapshot,
    bind_addr: &str,
    request: SignedRequest<'_>,
) -> String {
    authorized_signed_request(snapshot, bind_addr, request)
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
