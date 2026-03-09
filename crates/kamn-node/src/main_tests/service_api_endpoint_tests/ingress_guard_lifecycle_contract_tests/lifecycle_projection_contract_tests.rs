use super::super::*;
use super::support::{
    assert_server_ok, build_ingress_snapshot, send_signed_message_request_with_signature,
    spawn_ingress_server, state_hash,
};
use crate::service_api_endpoint::project_service_api_lifecycle_rejection;
use std::sync::{Arc, Barrier};

const LIFECYCLE_PROJECTION_SENDER_DID: &str = "kamn:did:agent:test-client-lifecycle-projection";
const LIFECYCLE_PROJECTION_BODY: &str = "{\"message\":\"lifecycle-projection-check\"}";

#[test]
fn unit_service_api_endpoint_lifecycle_rejection_projection_is_deterministic() {
    let first = project_service_api_lifecycle_rejection("service_api_ingress_rate_limit_exceeded").expect("known lifecycle reason code should project");
    let second = project_service_api_lifecycle_rejection("service_api_ingress_rate_limit_exceeded").expect("known lifecycle reason code should project");
    assert_eq!(first, second);
}

#[test]
fn functional_service_api_endpoint_lifecycle_rejection_projection_maps_limiter_classes() {
    let concurrency = project_service_api_lifecycle_rejection("service_api_ingress_concurrency_limit_exceeded").expect("concurrency limiter reason must project");
    assert_eq!(concurrency.rejection_class, "async-lifecycle-limiter");
    assert_eq!(concurrency.outcome, "concurrency-limit");
    let sender_suspended = project_service_api_lifecycle_rejection("service_api_ingress_sender_suspended").expect("sender suspension reason must project");
    assert_eq!(sender_suspended.rejection_class, "sender-admission-limiter");
    assert_eq!(sender_suspended.outcome, "anti-spam");
}

#[test]
fn functional_service_api_endpoint_backpressure_projection_covers_reason_codes() {
    for (reason_code, rejection_class, outcome) in [
        ("service_api_ingress_concurrency_limit_exceeded", "async-lifecycle-limiter", "concurrency-limit"),
        ("service_api_ingress_rate_limit_exceeded", "async-lifecycle-limiter", "rate-limit"),
        ("service_api_ingress_sender_rate_limit_exceeded", "sender-admission-limiter", "anti-spam"),
    ] {
        let projection = project_service_api_lifecycle_rejection(reason_code).expect("known backpressure reason code should project");
        assert_eq!(projection.reason_code, reason_code);
        assert_eq!(projection.rejection_class, rejection_class);
        assert_eq!(projection.status_code, 429);
        assert_eq!(projection.error_label, "too-many-requests");
        assert_eq!(projection.outcome, outcome);
    }
}

#[test]
fn integration_service_api_endpoint_lifecycle_projection_matches_live_concurrency_rejection() {
    let _env = acquire_service_api_test_env();
    let snapshot = build_ingress_snapshot("127.0.0.1:34067");
    let server = spawn_ingress_server(&snapshot, 4, 2_000, DEFAULT_SERVICE_API_BODY_LIMIT_BYTES, 1, 1_000);
    let responses = lifecycle_projection_responses(server.bind_addr.as_str(), 4, state_hash(&snapshot).as_str());
    let rejection = responses.iter().find(|response| response.contains("HTTP/1.1 429 Too Many Requests")).expect("expected at least one lifecycle limiter rejection");
    let rejection_payload = parse_error_envelope_from_http_response(rejection);
    let projection = project_service_api_lifecycle_rejection(rejection_payload.reason_code.as_str()).expect("live rejection reason should have projection");
    assert_eq!(projection.rejection_class, "async-lifecycle-limiter");
    assert_eq!(projection.reason_code, "service_api_ingress_concurrency_limit_exceeded");
    assert_eq!(projection.outcome, "concurrency-limit");
    assert_server_ok(server.server, "service api endpoint should stop cleanly after lifecycle projection integration budget");
}

#[test]
fn regression_service_api_endpoint_lifecycle_projection_sender_suspension_class_stays_stable() {
    let projection = project_service_api_lifecycle_rejection("service_api_ingress_sender_suspended").expect("sender suspension projection should exist");
    assert_eq!(projection.rejection_class, "sender-admission-limiter");
    assert_eq!(projection.status_code, 429);
    assert_eq!(projection.outcome, "anti-spam");
}

#[test]
fn performance_service_api_endpoint_lifecycle_projection_loop_stays_within_local_budget() {
    let started = Instant::now();
    let reason_codes = [
        "service_api_ingress_concurrency_limit_exceeded",
        "service_api_ingress_rate_limit_exceeded",
        "service_api_ingress_sender_rate_limit_exceeded",
        "service_api_ingress_sender_suspended",
    ];
    for idx in 0..60_000_u32 {
        let reason_code = reason_codes[idx as usize % reason_codes.len()];
        let projection = project_service_api_lifecycle_rejection(reason_code).expect("known lifecycle reason should project");
        assert_eq!(projection.reason_code, reason_code);
    }
    assert!(started.elapsed() <= Duration::from_secs(2));
}

fn lifecycle_projection_responses(bind_addr: &str, worker_count: usize, state_hash: &str) -> Vec<String> {
    let barrier = Arc::new(Barrier::new(worker_count));
    (0..worker_count)
        .map(|idx| spawn_lifecycle_projection_client(bind_addr, state_hash, barrier.clone(), idx))
        .map(|client| client.join().expect("client request should complete"))
        .collect()
}

fn spawn_lifecycle_projection_client(
    bind_addr: &str,
    state_hash: &str,
    barrier: Arc<Barrier>,
    request_index: usize,
) -> thread::JoinHandle<String> {
    let client_bind_addr = bind_addr.to_owned();
    let state_hash = state_hash.to_owned();
    let nonce = 810 + request_index as u64;
    thread::spawn(move || {
        lifecycle_projection_response(client_bind_addr.as_str(), state_hash.as_str(), barrier, nonce)
    })
}

fn lifecycle_projection_response(
    bind_addr: &str,
    state_hash: &str,
    barrier: Arc<Barrier>,
    nonce: u64,
) -> String {
    let signature = service_api_request_signature_for_fields(
        LIFECYCLE_PROJECTION_SENDER_DID,
        nonce,
        state_hash,
        LIFECYCLE_PROJECTION_BODY,
    );
    barrier.wait();
    send_signed_message_request_with_signature(
        bind_addr,
        LIFECYCLE_PROJECTION_SENDER_DID,
        nonce,
        signature.as_str(),
        LIFECYCLE_PROJECTION_BODY,
    )
}
