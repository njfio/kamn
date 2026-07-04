use super::super::*;
use super::concurrency_support::{assert_has_concurrency_rejection, spawn_concurrency_burst};
use super::support::{assert_server_ok, build_ingress_snapshot, spawn_ingress_server, state_hash};

#[test]
fn integration_service_api_endpoint_rejects_when_concurrency_limit_is_exceeded() {
    let _env = acquire_service_api_test_env();
    let snapshot = build_ingress_snapshot("127.0.0.1:34063");
    let server = spawn_ingress_server(
        &snapshot,
        6,
        2_000,
        DEFAULT_SERVICE_API_BODY_LIMIT_BYTES,
        1,
        1_000,
    );
    let responses = spawn_concurrency_burst(
        server.bind_addr.as_str(),
        6,
        0,
        200,
        state_hash(&snapshot).as_str(),
        |request_index| {
            (
                "kamn:did:agent:test-client-concurrency-limit".to_owned(),
                format!("{{\"message\":\"concurrency-limit-check-{request_index}\"}}"),
            )
        },
    );

    assert_has_concurrency_rejection(
        responses,
        "expected at least one request to fail closed on concurrency limit",
    );
    assert_server_ok(
        server.server,
        "service api endpoint should stop cleanly after configured request budget",
    );
}

#[test]
fn integration_service_api_endpoint_concurrency_rejection_reason_stays_stable_under_bounded_bursts()
{
    let _env = acquire_service_api_test_env();
    let snapshot = build_ingress_snapshot("127.0.0.1:34070");
    let server = spawn_ingress_server(
        &snapshot,
        16,
        3_000,
        DEFAULT_SERVICE_API_BODY_LIMIT_BYTES,
        1,
        10_000,
    );
    let state_hash = state_hash(&snapshot);

    for round in 0..2_u64 {
        let responses = spawn_concurrency_burst(
            server.bind_addr.as_str(),
            8,
            round,
            12_000,
            state_hash.as_str(),
            |request_index| {
                let sender_did =
                    format!("kamn:did:agent:test-client-concurrency-burst-{round}-{request_index}");
                let body = format!(
                    "{{\"message\":\"concurrency-burst-round-{round}-request-{request_index}\"}}"
                );
                (sender_did, body)
            },
        );
        assert_has_concurrency_rejection(responses, "expected fail-closed concurrency rejections");
    }

    assert_server_ok(
        server.server,
        "service api endpoint should stop cleanly after bounded concurrency bursts",
    );
}

#[test]
fn regression_service_api_endpoint_concurrency_limit_reason_code_stays_stable_across_rounds() {
    let _env = acquire_service_api_test_env();
    let snapshot = build_ingress_snapshot("127.0.0.1:34068");
    let server = spawn_ingress_server(
        &snapshot,
        12,
        2_000,
        DEFAULT_SERVICE_API_BODY_LIMIT_BYTES,
        1,
        1_000,
    );
    let state_hash = state_hash(&snapshot);

    for round in 0..3_u64 {
        let responses = spawn_concurrency_burst(
            server.bind_addr.as_str(),
            4,
            round,
            4_000,
            state_hash.as_str(),
            |request_index| {
                let sender_did =
                    format!("kamn:did:agent:test-client-concurrency-regression-{round}");
                let body = format!("{{\"message\":\"concurrency-stability-round-{round}-request-{request_index}\"}}");
                (sender_did, body)
            },
        );
        assert_has_concurrency_rejection(
            responses,
            "expected at least one fail-closed concurrency rejection",
        );
    }

    assert_server_ok(
        server.server,
        "service api endpoint should stop cleanly after concurrency regression rounds",
    );
}
