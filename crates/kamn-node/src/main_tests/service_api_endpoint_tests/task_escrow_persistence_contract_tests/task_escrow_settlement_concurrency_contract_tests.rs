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

fn concurrent_release_requests(
    snapshot: &crate::service_api_endpoint::ServiceApiSnapshot,
    path: &str,
    body: &str,
) -> Vec<String> {
    let bind_addr = reserve_loopback_addr();
    with_api_server(snapshot, bind_addr.as_str(), 2, |addr| {
        let barrier = std::sync::Arc::new(std::sync::Barrier::new(2));
        thread::scope(|scope| {
            [183_u64, 183_u64]
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
