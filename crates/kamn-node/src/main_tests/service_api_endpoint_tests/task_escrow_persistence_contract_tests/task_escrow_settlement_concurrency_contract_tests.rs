use super::super::*;
use super::support::*;

#[path = "settlement_concurrency_support.rs"]
mod concurrency_support;
use concurrency_support::{concurrent_release_requests, ordered_authenticated_release_race};

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

    let gate = crate::service_api_endpoint::set_test_post_auth_gate();
    let responses =
        ordered_authenticated_release_race(&context.harness.snapshot, &path, body, &gate);
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
        2,
        "both authorized retries should succeed: {responses:?}"
    );
    let persisted_signature = settlement_tx_signature(&state["escrows"][&escrow_id]);
    assert!(
        successes
            .iter()
            .all(|success| settlement_tx_signature(success) == persisted_signature),
        "both responses must converge on the persisted signature"
    );
    assert_eq!(
        crate::service_api_endpoint::test_live_solana_settlement_submission_count(),
        1,
        "same-key retries with fresh nonces must not resubmit the adapter transfer"
    );
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
