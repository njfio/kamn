use super::super::*;
use super::support::{
    assert_server_ok, build_ingress_snapshot, send_signed_message_request, spawn_ingress_server,
};

#[test]
fn functional_service_api_endpoint_applies_sender_anti_spam_throttle_and_suspension() {
    let _env = acquire_service_api_test_env();
    let snapshot = build_ingress_snapshot("127.0.0.1:34065");
    let server = spawn_ingress_server(
        &snapshot,
        6,
        3_000,
        DEFAULT_SERVICE_API_BODY_LIMIT_BYTES,
        DEFAULT_SERVICE_API_CONCURRENCY_LIMIT,
        1_000,
    );
    let sender_did = "kamn:did:agent:test-client-anti-spam";
    let responses = anti_spam_round_responses(
        &snapshot,
        server.bind_addr.as_str(),
        sender_did,
        610,
        "{\"message\":\"anti-spam-check\"}",
    );

    assert!(responses[0].contains("HTTP/1.1 202 Accepted"));
    assert!(responses[1].contains("HTTP/1.1 202 Accepted"));
    assert!(responses[2].contains("HTTP/1.1 202 Accepted"));
    assert_reason_code(
        responses[3].as_str(),
        "service_api_ingress_sender_rate_limit_exceeded",
    );
    assert_reason_code(
        responses[4].as_str(),
        "service_api_ingress_sender_rate_limit_exceeded",
    );
    assert_reason_code(
        responses[5].as_str(),
        "service_api_ingress_sender_suspended",
    );
    assert_server_ok(
        server.server,
        "service api endpoint should stop cleanly after configured request budget",
    );
}

#[test]
fn integration_service_api_endpoint_sender_anti_spam_burst_rounds_remain_deterministic() {
    let _env = acquire_service_api_test_env();
    let snapshot = build_ingress_snapshot("127.0.0.1:34069");
    let server = spawn_ingress_server(
        &snapshot,
        18,
        3_000,
        DEFAULT_SERVICE_API_BODY_LIMIT_BYTES,
        DEFAULT_SERVICE_API_CONCURRENCY_LIMIT,
        10_000,
    );

    for round in 0..3_u64 {
        let sender_did = format!("kamn:did:agent:test-client-anti-spam-burst-{round}");
        let body = format!("{{\"message\":\"anti-spam-burst-round-{round}\"}}");
        let responses = anti_spam_round_responses(
            &snapshot,
            server.bind_addr.as_str(),
            sender_did.as_str(),
            9_000 + round * 6,
            body.as_str(),
        );
        assert!(responses[0].contains("HTTP/1.1 202 Accepted"));
        assert!(responses[1].contains("HTTP/1.1 202 Accepted"));
        assert!(responses[2].contains("HTTP/1.1 202 Accepted"));
        assert_reason_code(
            responses[3].as_str(),
            "service_api_ingress_sender_rate_limit_exceeded",
        );
        assert_reason_code(
            responses[4].as_str(),
            "service_api_ingress_sender_rate_limit_exceeded",
        );
        assert_reason_code(
            responses[5].as_str(),
            "service_api_ingress_sender_suspended",
        );
    }

    assert_server_ok(
        server.server,
        "service api endpoint should stop cleanly after anti-spam burst rounds",
    );
}

fn anti_spam_round_responses(
    snapshot: &crate::service_api_endpoint::ServiceApiSnapshot,
    bind_addr: &str,
    sender_did: &str,
    start_nonce: u64,
    body: &str,
) -> Vec<String> {
    (0..6_u64)
        .map(|offset| {
            send_signed_message_request(snapshot, bind_addr, sender_did, start_nonce + offset, body)
        })
        .collect()
}

fn assert_reason_code(response: &str, reason_code: &str) {
    assert!(response.contains("HTTP/1.1 429 Too Many Requests"));
    let payload = parse_error_envelope_from_http_response(response);
    assert_eq!(payload.error, "too-many-requests");
    assert_eq!(payload.reason_code, reason_code);
}
