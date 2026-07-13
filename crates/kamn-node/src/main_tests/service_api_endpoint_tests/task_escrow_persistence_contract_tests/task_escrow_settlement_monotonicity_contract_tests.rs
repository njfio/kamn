use super::super::*;
use super::support::*;

const ACTOR: &str = "kamn:did:agent:test-client-settlement-monotonicity";

#[test]
fn integration_confirmed_settlement_cannot_roll_back_on_restart_retry() {
    let outcome = run_restart_retry();
    assert!(
        outcome.first.contains("HTTP/1.1 200 OK"),
        "{}",
        outcome.first
    );
    assert!(
        outcome.retry.contains("HTTP/1.1 200 OK"),
        "{}",
        outcome.retry
    );
    assert_terminal_state_unchanged(&outcome.before, &outcome.after, &outcome.escrow_id);
    assert_eq!(
        outcome.submissions, 1,
        "proof retry must not submit settlement"
    );
}

struct RetryOutcome {
    first: String,
    retry: String,
    before: Value,
    after: Value,
    escrow_id: String,
    submissions: u64,
}

fn run_restart_retry() -> RetryOutcome {
    let _env = acquire_service_api_test_env();
    let _override_guard =
        crate::service_api_endpoint::set_test_live_solana_settlement_override(true);
    let context = build_live_solana_asset_movement_context(params());
    let escrow_id = fund_live_escrow(&context.harness, 211, 53);
    let first = release(&context, 213, &escrow_id);
    let before = read_state_json(context.harness.state_file.as_path());
    let retry = release(&context, 214, &escrow_id);
    let after = read_state_json(context.harness.state_file.as_path());
    let submissions = crate::service_api_endpoint::test_live_solana_settlement_submission_count();
    RetryOutcome {
        first,
        retry,
        before,
        after,
        escrow_id,
        submissions,
    }
}

fn release(context: &LiveSolanaAssetMovementContext, nonce: u64, escrow_id: &str) -> String {
    release_escrow_response_with_key(
        &context.harness.snapshot,
        context.harness.bind_addr.as_str(),
        context.harness.caller_did,
        nonce,
        escrow_id,
        "settlement-monotonic-retry",
    )
}

fn assert_terminal_state_unchanged(before: &Value, after: &Value, escrow_id: &str) {
    let before_intent = &before["settlement_intents"][escrow_id];
    let after_intent = &after["settlement_intents"][escrow_id];
    assert_eq!(after_intent["state"], "confirmed");
    assert_eq!(after["escrows"][escrow_id]["state"], "released");
    assert_eq!(
        after_intent["expected_signature"],
        before_intent["expected_signature"]
    );
    assert_eq!(
        after_intent["submission_attempt_count"],
        before_intent["submission_attempt_count"]
    );
    assert_eq!(
        after_intent["last_error_code"],
        before_intent["last_error_code"]
    );
}

fn params() -> LiveSolanaAssetMovementParams<'static> {
    LiveSolanaAssetMovementParams {
        state_file_prefix: "settlement-monotonicity",
        caller_did: ACTOR,
        api_bind: "127.0.0.1:34138",
        keypair_prefix: "kamn-node-settlement-monotonicity-keypair",
        keypair_env: "KAMN_SERVICE_API_LIVE_SOLANA_SETTLEMENT_KEYPAIR_FILE",
        recipient_env: "KAMN_SERVICE_API_LIVE_SOLANA_SETTLEMENT_RECIPIENT_PUBKEY",
        lamports_env: "KAMN_SERVICE_API_LIVE_SOLANA_SETTLEMENT_LAMPORTS",
        live_rpc_env: "https://api.devnet.solana.com",
        amount_lamports: 53,
    }
}
