use super::super::*;
use super::support::{
    assert_server_ok, build_ingress_snapshot, send_signed_message_request_with_signature,
    spawn_ingress_server, state_hash,
};
use std::sync::{Arc, Barrier};

struct BurstContext<'a> {
    bind_addr: &'a str,
    worker_count: usize,
    round: u64,
    base_nonce: u64,
    state_hash: &'a str,
    barrier: Arc<Barrier>,
}

#[test]
fn integration_service_api_endpoint_rejects_when_concurrency_limit_is_exceeded() {
    let _env = acquire_service_api_test_env();
    let snapshot = build_ingress_snapshot("127.0.0.1:34063");
    let server = spawn_ingress_server(&snapshot, 6, 2_000, DEFAULT_SERVICE_API_BODY_LIMIT_BYTES, 1, 1_000);
    let responses = spawn_concurrency_burst(
        server.bind_addr.as_str(),
        6,
        0,
        200,
        state_hash(&snapshot).as_str(),
        |request_index| ("kamn:did:agent:test-client-concurrency-limit".to_owned(), format!("{{\"message\":\"concurrency-limit-check-{request_index}\"}}")),
    );

    assert_has_concurrency_rejection(responses, "expected at least one request to fail closed on concurrency limit");
    assert_server_ok(server.server, "service api endpoint should stop cleanly after configured request budget");
}

#[test]
fn integration_service_api_endpoint_concurrency_rejection_reason_stays_stable_under_bounded_bursts() {
    let _env = acquire_service_api_test_env();
    let snapshot = build_ingress_snapshot("127.0.0.1:34070");
    let server = spawn_ingress_server(&snapshot, 16, 3_000, DEFAULT_SERVICE_API_BODY_LIMIT_BYTES, 1, 10_000);
    let state_hash = state_hash(&snapshot);

    for round in 0..2_u64 {
        let responses = spawn_concurrency_burst(server.bind_addr.as_str(), 8, round, 12_000, state_hash.as_str(), |request_index| {
            let sender_did = format!("kamn:did:agent:test-client-concurrency-burst-{round}-{request_index}");
            let body = format!("{{\"message\":\"concurrency-burst-round-{round}-request-{request_index}\"}}");
            (sender_did, body)
        });
        assert_has_concurrency_rejection(responses, "expected fail-closed concurrency rejections");
    }

    assert_server_ok(server.server, "service api endpoint should stop cleanly after bounded concurrency bursts");
}

#[test]
fn regression_service_api_endpoint_concurrency_limit_reason_code_stays_stable_across_rounds() {
    let _env = acquire_service_api_test_env();
    let snapshot = build_ingress_snapshot("127.0.0.1:34068");
    let server = spawn_ingress_server(&snapshot, 12, 2_000, DEFAULT_SERVICE_API_BODY_LIMIT_BYTES, 1, 1_000);
    let state_hash = state_hash(&snapshot);

    for round in 0..3_u64 {
        let responses = spawn_concurrency_burst(server.bind_addr.as_str(), 4, round, 4_000, state_hash.as_str(), |request_index| {
            let sender_did = format!("kamn:did:agent:test-client-concurrency-regression-{round}");
            let body = format!("{{\"message\":\"concurrency-stability-round-{round}-request-{request_index}\"}}");
            (sender_did, body)
        });
        assert_has_concurrency_rejection(responses, "expected at least one fail-closed concurrency rejection");
    }

    assert_server_ok(server.server, "service api endpoint should stop cleanly after concurrency regression rounds");
}

fn spawn_concurrency_burst<F>(bind_addr: &str, worker_count: usize, round: u64, base_nonce: u64, state_hash: &str, request: F) -> Vec<String>
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
    let clients =
        (0..worker_count).map(|idx| spawn_concurrency_client(&context, idx, &request));
    clients
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
    let nonce = context.base_nonce + context.round * context.worker_count as u64 + request_index as u64;
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
    let signature =
        service_api_request_signature_for_fields(sender_did, nonce, state_hash, body);
    barrier.wait();
    send_signed_message_request_with_signature(
        bind_addr,
        sender_did,
        nonce,
        signature.as_str(),
        body,
    )
}

fn assert_has_concurrency_rejection(responses: Vec<String>, missing_rejection_message: &str) {
    assert!(responses.iter().any(|response| response.contains("HTTP/1.1 202 Accepted")));
    let rejection_payloads = responses
        .iter()
        .filter(|response| response.contains("HTTP/1.1 429 Too Many Requests"))
        .map(|response| parse_error_envelope_from_http_response(response))
        .collect::<Vec<ServiceApiErrorEnvelope>>();
    assert!(!rejection_payloads.is_empty(), "{missing_rejection_message}");
    for payload in rejection_payloads {
        assert_eq!(payload.error, "too-many-requests");
        assert_eq!(payload.reason_code, "service_api_ingress_concurrency_limit_exceeded");
    }
}
