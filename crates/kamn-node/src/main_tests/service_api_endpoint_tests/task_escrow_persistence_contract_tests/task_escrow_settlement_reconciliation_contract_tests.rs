use super::super::*;
use super::support::*;

const RPC_URL: &str = "https://api.devnet.solana.com";
const KEYPAIR_ENV: &str = "KAMN_SERVICE_API_LIVE_SOLANA_SETTLEMENT_KEYPAIR_FILE";
const RECIPIENT_ENV: &str = "KAMN_SERVICE_API_LIVE_SOLANA_SETTLEMENT_RECIPIENT_PUBKEY";
const LAMPORTS_ENV: &str = "KAMN_SERVICE_API_LIVE_SOLANA_SETTLEMENT_LAMPORTS";

#[test]
fn integration_ambiguous_settlement_retry_reconciles_without_resubmission() {
    let _env = acquire_service_api_test_env();
    let _override_guard =
        crate::service_api_endpoint::set_test_live_solana_settlement_ambiguous_after_submit();
    let context = build_live_solana_asset_movement_context(params());
    let escrow_id = fund_live_escrow(&context.harness, 151, 31);

    let first = release_escrow_response_with_key(
        &context.harness.snapshot,
        context.harness.bind_addr.as_str(),
        context.harness.caller_did,
        153,
        escrow_id.as_str(),
        "settlement-reconcile-1",
    );
    crate::service_api_endpoint::set_test_live_solana_settlement_reconcile_confirmed();
    let second = release_escrow_response_with_key(
        &context.harness.snapshot,
        context.harness.bind_addr.as_str(),
        context.harness.caller_did,
        154,
        escrow_id.as_str(),
        "settlement-reconcile-1",
    );
    let state = read_state_json(context.harness.state_file.as_path());

    assert!(first.contains("SETTLEMENT_OUTCOME_AMBIGUOUS"), "{first}");
    assert!(second.contains("HTTP/1.1 200 OK"), "{second}");
    assert_eq!(
        crate::service_api_endpoint::test_live_solana_settlement_submission_count(),
        1,
        "reconciliation must not submit the signed transaction again"
    );
    assert_eq!(
        state["settlement_intents"][&escrow_id]["state"],
        "confirmed"
    );
    assert_eq!(state["escrows"][&escrow_id]["state"], "released");
}

fn params() -> LiveSolanaAssetMovementParams<'static> {
    LiveSolanaAssetMovementParams {
        state_file_prefix: "kamn-node-settlement-reconciliation-state",
        caller_did: "kamn:did:agent:test-client-settlement-reconciliation",
        api_bind: "127.0.0.1:34136",
        keypair_prefix: "kamn-node-settlement-reconciliation-keypair",
        keypair_env: KEYPAIR_ENV,
        recipient_env: RECIPIENT_ENV,
        lamports_env: LAMPORTS_ENV,
        live_rpc_env: RPC_URL,
        amount_lamports: 31,
    }
}
