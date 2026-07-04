use super::super::*;
use super::support::send_signed_message_request_with_signature;
use std::sync::{Arc, Barrier};

struct BurstContext<'a> {
    bind_addr: &'a str,
    worker_count: usize,
    round: u64,
    base_nonce: u64,
    state_hash: &'a str,
    barrier: Arc<Barrier>,
}

pub(super) fn spawn_concurrency_burst<F>(
    bind_addr: &str,
    worker_count: usize,
    round: u64,
    base_nonce: u64,
    state_hash: &str,
    request: F,
) -> Vec<String>
where
    F: Fn(usize) -> (String, String),
{
    let context = BurstContext {
        bind_addr,
        worker_count,
        round,
        base_nonce,
        state_hash,
        barrier: Arc::new(Barrier::new(worker_count)),
    };
    let clients = (0..worker_count)
        .map(|idx| spawn_concurrency_client(&context, idx, &request))
        .collect::<Vec<_>>();
    clients
        .into_iter()
        .map(|client| client.join().expect("client request should complete"))
        .collect()
}

fn spawn_concurrency_client<F>(
    context: &BurstContext<'_>,
    request_index: usize,
    request: &F,
) -> thread::JoinHandle<String>
where
    F: Fn(usize) -> (String, String),
{
    let client_bind_addr = context.bind_addr.to_owned();
    let state_hash = context.state_hash.to_owned();
    let barrier = context.barrier.clone();
    let nonce =
        context.base_nonce + context.round * context.worker_count as u64 + request_index as u64;
    let (sender_did, body) = request(request_index);
    thread::spawn(move || {
        concurrency_client_response(
            client_bind_addr.as_str(),
            state_hash.as_str(),
            barrier,
            nonce,
            sender_did.as_str(),
            body.as_str(),
        )
    })
}

fn concurrency_client_response(
    bind_addr: &str,
    state_hash: &str,
    barrier: Arc<Barrier>,
    nonce: u64,
    sender_did: &str,
    body: &str,
) -> String {
    let signature = service_api_request_signature_for_fields(sender_did, nonce, state_hash, body);
    barrier.wait();
    send_signed_message_request_with_signature(
        bind_addr,
        sender_did,
        nonce,
        signature.as_str(),
        body,
    )
}

pub(super) fn assert_has_concurrency_rejection(
    responses: Vec<String>,
    missing_rejection_message: &str,
) {
    assert!(responses
        .iter()
        .any(|response| response.contains("HTTP/1.1 202 Accepted")));
    let rejection_payloads = responses
        .iter()
        .filter(|response| response.contains("HTTP/1.1 429 Too Many Requests"))
        .map(|response| parse_error_envelope_from_http_response(response))
        .collect::<Vec<ServiceApiErrorEnvelope>>();
    assert!(
        !rejection_payloads.is_empty(),
        "{missing_rejection_message}"
    );
    for payload in rejection_payloads {
        assert_eq!(payload.error, "too-many-requests");
        assert_eq!(
            payload.reason_code,
            "service_api_ingress_concurrency_limit_exceeded"
        );
    }
}
