use super::super::*;
use super::support::{
    assert_server_ok, build_ingress_snapshot, send_signed_message_request, spawn_ingress_server,
};

#[test]
fn regression_service_api_endpoint_rejects_replayed_request_nonce_for_sender() {
    let _env = acquire_service_api_test_env();
    let snapshot = build_ingress_snapshot("127.0.0.1:34054");
    let server = spawn_ingress_server(
        &snapshot,
        2,
        2_000,
        DEFAULT_SERVICE_API_BODY_LIMIT_BYTES,
        DEFAULT_SERVICE_API_CONCURRENCY_LIMIT,
        DEFAULT_SERVICE_API_RATE_LIMIT_PER_SECOND,
    );
    let body = "{\"message\":\"replay-check\"}";
    let sender_did = "kamn:did:agent:test-client-2";
    let first_response =
        send_signed_message_request(&snapshot, server.bind_addr.as_str(), sender_did, 7, body);
    let replay_response =
        send_signed_message_request(&snapshot, server.bind_addr.as_str(), sender_did, 7, body);

    assert!(first_response.contains("HTTP/1.1 202 Accepted"));
    assert_replay_rejection(replay_response.as_str());
    assert_server_ok(
        server.server,
        "service api endpoint should stop cleanly after configured request budget",
    );
}

#[test]
fn integration_service_api_endpoint_rejects_previously_accepted_nonce_after_restart_reload() {
    let _env = acquire_service_api_test_env();
    let state_file = std::env::temp_dir().join(format!(
        "kamn-node-ingress-replay-restart-{}-{}.json",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("clock")
            .as_nanos()
    ));
    let _state_guard = EnvVarGuard::set(
        "KAMN_SERVICE_API_STATE_FILE",
        Some(state_file.to_string_lossy().as_ref()),
    );
    let sender_did = "kamn:did:agent:test-client-replay-restart";
    let body = "{\"message\":\"replay-after-restart\"}";

    let first_snapshot = build_ingress_snapshot("127.0.0.1:34072");
    let first_server = spawn_ingress_server(
        &first_snapshot,
        1,
        2_000,
        DEFAULT_SERVICE_API_BODY_LIMIT_BYTES,
        DEFAULT_SERVICE_API_CONCURRENCY_LIMIT,
        DEFAULT_SERVICE_API_RATE_LIMIT_PER_SECOND,
    );
    let first_response = send_signed_message_request(
        &first_snapshot,
        first_server.bind_addr.as_str(),
        sender_did,
        17,
        body,
    );
    assert!(first_response.contains("HTTP/1.1 202 Accepted"));
    assert_server_ok(
        first_server.server,
        "first ingress server should stop cleanly after accepting the nonce",
    );

    let restarted_snapshot = build_ingress_snapshot("127.0.0.1:34073");
    let restarted_server = spawn_ingress_server(
        &restarted_snapshot,
        1,
        2_000,
        DEFAULT_SERVICE_API_BODY_LIMIT_BYTES,
        DEFAULT_SERVICE_API_CONCURRENCY_LIMIT,
        DEFAULT_SERVICE_API_RATE_LIMIT_PER_SECOND,
    );
    let replay_response = send_signed_message_request(
        &restarted_snapshot,
        restarted_server.bind_addr.as_str(),
        sender_did,
        17,
        body,
    );

    assert_replay_rejection(replay_response.as_str());
    assert_server_ok(
        restarted_server.server,
        "restarted ingress server should stop cleanly after rejecting the replayed nonce",
    );
    let _ = std::fs::remove_file(state_file);
}

#[test]
fn integration_service_api_endpoint_replay_rejection_remains_stable_with_anti_spam_enforcement() {
    let _env = acquire_service_api_test_env();
    let snapshot = build_ingress_snapshot("127.0.0.1:34066");
    let server = spawn_ingress_server(
        &snapshot,
        3,
        2_000,
        DEFAULT_SERVICE_API_BODY_LIMIT_BYTES,
        DEFAULT_SERVICE_API_CONCURRENCY_LIMIT,
        1_000,
    );
    let body = "{\"message\":\"replay-anti-spam-matrix\"}";
    let sender_did = "kamn:did:agent:test-client-replay-anti-spam";
    let first_response =
        send_signed_message_request(&snapshot, server.bind_addr.as_str(), sender_did, 701, body);
    let replay_response =
        send_signed_message_request(&snapshot, server.bind_addr.as_str(), sender_did, 701, body);
    let fresh_nonce_response =
        send_signed_message_request(&snapshot, server.bind_addr.as_str(), sender_did, 702, body);

    assert!(first_response.contains("HTTP/1.1 202 Accepted"));
    assert_replay_rejection(replay_response.as_str());
    assert!(fresh_nonce_response.contains("HTTP/1.1 202 Accepted"));
    assert_server_ok(
        server.server,
        "service api endpoint should stop cleanly after configured request budget",
    );
}

#[test]
fn regression_service_api_endpoint_replay_duplicate_sequence_reason_ordering_stays_stable() {
    let _env = acquire_service_api_test_env();
    let snapshot = build_ingress_snapshot("127.0.0.1:34071");
    let server = spawn_ingress_server(
        &snapshot,
        6,
        2_000,
        DEFAULT_SERVICE_API_BODY_LIMIT_BYTES,
        DEFAULT_SERVICE_API_CONCURRENCY_LIMIT,
        DEFAULT_SERVICE_API_RATE_LIMIT_PER_SECOND,
    );
    let sender_did = "kamn:did:agent:test-client-replay-duplicate-sequence";
    let observed = replay_duplicate_sequence(&snapshot, server.bind_addr.as_str(), sender_did, 3);
    let expected = vec![
        "accepted".to_owned(),
        "service_api_auth_replay_nonce_detected".to_owned(),
        "accepted".to_owned(),
        "service_api_auth_replay_nonce_detected".to_owned(),
        "accepted".to_owned(),
        "service_api_auth_replay_nonce_detected".to_owned(),
    ];
    assert_eq!(observed, expected);
    assert_server_ok(
        server.server,
        "service api endpoint should stop cleanly after configured request budget",
    );
}

fn assert_replay_rejection(response: &str) {
    assert!(response.contains("HTTP/1.1 409 Conflict"));
    let payload = parse_error_envelope_from_http_response(response);
    assert_eq!(payload.error, "replay");
    assert_eq!(
        payload.reason_code,
        "service_api_auth_replay_nonce_detected"
    );
    assert!(payload.message.contains("request nonce replay detected"));
}

fn replay_duplicate_sequence(
    snapshot: &crate::service_api_endpoint::ServiceApiSnapshot,
    bind_addr: &str,
    sender_did: &str,
    rounds: u64,
) -> Vec<String> {
    let mut observed = Vec::new();
    for round in 0..rounds {
        let body = format!("{{\"message\":\"replay-duplicate-round-{round}\"}}");
        let nonce = 13_000 + round;
        let first =
            send_signed_message_request(snapshot, bind_addr, sender_did, nonce, body.as_str());
        assert!(first.contains("HTTP/1.1 202 Accepted"));
        observed.push("accepted".to_owned());
        let replay =
            send_signed_message_request(snapshot, bind_addr, sender_did, nonce, body.as_str());
        let replay_payload = parse_error_envelope_from_http_response(replay.as_str());
        observed.push(replay_payload.reason_code);
    }
    observed
}
