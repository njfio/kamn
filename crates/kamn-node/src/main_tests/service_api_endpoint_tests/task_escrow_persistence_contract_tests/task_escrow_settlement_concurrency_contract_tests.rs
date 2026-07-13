use super::super::*;
use super::support::*;

const ACTOR: &str = "kamn:did:agent:test-client-settlement-concurrency";
const KEYPAIR_ENV: &str = "KAMN_SERVICE_API_LIVE_SOLANA_SETTLEMENT_KEYPAIR_FILE";
const RECIPIENT_ENV: &str = "KAMN_SERVICE_API_LIVE_SOLANA_SETTLEMENT_RECIPIENT_PUBKEY";
const LAMPORTS_ENV: &str = "KAMN_SERVICE_API_LIVE_SOLANA_SETTLEMENT_LAMPORTS";

#[test]
fn integration_concurrent_settlement_release_submits_one_transaction_identity() {
    let _env = acquire_service_api_test_env();
    let _override_guard =
        crate::service_api_endpoint::set_test_live_solana_settlement_override(true);
    let context = build_live_solana_asset_movement_context(params());
    let escrow_id = fund_live_escrow(&context.harness, 181, 43);
    let path = format!("/v1/escrow/{escrow_id}/release");
    let body = r#"{"idempotency_key":"settlement-concurrent-release"}"#;
    provision_signed_request_grant(&SignedRequest {
        max_requests: 1,
        method: "POST",
        path: path.as_str(),
        caller_did: ACTOR,
        nonce: 183,
        body,
        extra_headers: &[],
    });

    let responses = concurrent_release_requests(&context.harness.snapshot, &path, body);

    assert_eq!(
        responses
            .iter()
            .filter(|response| response.contains("200 OK"))
            .count(),
        1
    );
    assert_eq!(
        responses
            .iter()
            .filter(|response| response.contains("409 Conflict"))
            .count(),
        1
    );
    assert_eq!(
        crate::service_api_endpoint::test_live_solana_settlement_submission_count(),
        1
    );
}

#[test]
fn integration_distinct_release_nonces_with_same_key_converge_on_one_persisted_signature() {
    let _env = acquire_service_api_test_env();
    let _override_guard =
        crate::service_api_endpoint::set_test_live_solana_settlement_override(true);
    let context = build_live_solana_asset_movement_context(params());
    let escrow_id = fund_live_escrow(&context.harness, 191, 43);
    let path = format!("/v1/escrow/{escrow_id}/release");
    let body = r#"{"idempotency_key":"settlement-distinct-nonce-shared-key"}"#;
    provision_signed_request_grant(&SignedRequest {
        max_requests: 2,
        method: "POST",
        path: path.as_str(),
        caller_did: ACTOR,
        nonce: 193,
        body,
        extra_headers: &[],
    });

    let responses =
        concurrent_release_requests_for_nonces(&context.harness.snapshot, &path, body, [193, 194]);
    let successes: Vec<Value> = responses
        .iter()
        .filter(|response| response.contains("HTTP/1.1 200 OK"))
        .map(|response| {
            parse_service_api_payload(extract_http_response_body(response))
                .expect("success payload")
        })
        .collect();
    let state = read_state_json(context.harness.state_file.as_path());

    assert_eq!(
        successes.len(),
        1,
        "expected exactly one successful raced release: {responses:?}"
    );
    assert_eq!(
        responses
            .iter()
            .filter(|response| response.contains("HTTP/1.1 409 Conflict"))
            .count(),
        1,
        "expected exactly one raced release conflict: {responses:?}"
    );
    let conflict = responses
        .iter()
        .find(|response| response.contains("HTTP/1.1 409 Conflict"))
        .expect("one raced response should conflict");
    let conflict_payload = parse_error_envelope_from_http_response(conflict);
    assert_ne!(
        conflict_payload.reason_code, "service_api_auth_replay_nonce_detected",
        "distinct nonces must not collapse into replay rejection"
    );
    let success = &successes[0];
    assert_eq!(
        settlement_tx_signature(&state["escrows"][&escrow_id]),
        settlement_tx_signature(success),
        "persisted release state should keep the converged signature"
    );
    assert_eq!(
        crate::service_api_endpoint::test_live_solana_settlement_submission_count(),
        1,
        "same-key retries with fresh nonces must not resubmit the adapter transfer"
    );
}

fn concurrent_release_requests(
    snapshot: &crate::service_api_endpoint::ServiceApiSnapshot,
    path: &str,
    body: &str,
) -> Vec<String> {
    concurrent_release_requests_for_nonces(snapshot, path, body, [183, 183])
}

fn concurrent_release_requests_for_nonces(
    snapshot: &crate::service_api_endpoint::ServiceApiSnapshot,
    path: &str,
    body: &str,
    nonces: [u64; 2],
) -> Vec<String> {
    let bind_addr = reserve_loopback_addr();
    with_api_server(snapshot, bind_addr.as_str(), 2, |addr| {
        let barrier = std::sync::Arc::new(std::sync::Barrier::new(2));
        thread::scope(|scope| {
            nonces
                .into_iter()
                .map(|nonce| {
                    spawn_release(scope, barrier.clone(), snapshot, addr, path, body, nonce)
                })
                .collect::<Vec<_>>()
                .into_iter()
                .map(|handle| handle.join().expect("release request should complete"))
                .collect()
        })
    })
}

fn spawn_release<'scope>(
    scope: &'scope thread::Scope<'scope, '_>,
    barrier: std::sync::Arc<std::sync::Barrier>,
    snapshot: &'scope crate::service_api_endpoint::ServiceApiSnapshot,
    addr: &'scope str,
    path: &'scope str,
    body: &'scope str,
    nonce: u64,
) -> thread::ScopedJoinHandle<'scope, String> {
    scope.spawn(move || {
        let nonce_text = nonce.to_string();
        let signature = service_api_request_signature_for_fields(
            ACTOR,
            nonce,
            state_hash(snapshot).as_str(),
            body,
        );
        barrier.wait();
        send_http_request_with_headers(
            addr,
            "POST",
            path,
            body,
            &[
                ("X-KAMN-Sender-DID", ACTOR),
                ("X-KAMN-Request-Nonce", nonce_text.as_str()),
                ("X-KAMN-Request-Signature", signature.as_str()),
                ("X-KAMN-Authz-Scope", "escrow:write"),
            ],
        )
    })
}

fn params() -> LiveSolanaAssetMovementParams<'static> {
    LiveSolanaAssetMovementParams {
        state_file_prefix: "kamn-node-settlement-concurrency-state",
        caller_did: ACTOR,
        api_bind: "127.0.0.1:34137",
        keypair_prefix: "kamn-node-settlement-concurrency-keypair",
        keypair_env: KEYPAIR_ENV,
        recipient_env: RECIPIENT_ENV,
        lamports_env: LAMPORTS_ENV,
        live_rpc_env: "https://api.devnet.solana.com",
        amount_lamports: 43,
    }
}
