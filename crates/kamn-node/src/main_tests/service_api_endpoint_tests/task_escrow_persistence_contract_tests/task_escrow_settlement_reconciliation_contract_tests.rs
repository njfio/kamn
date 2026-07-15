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
    assert_eq!(
        state["settlement_intents"][&escrow_id]["submission_attempt_count"],
        1,
        "reconciliation must persist one submission attempt"
    );
}

#[test]
fn integration_settlement_intent_rejects_conflicting_release_key_without_submission() {
    let _env = acquire_service_api_test_env();
    let _override_guard =
        crate::service_api_endpoint::set_test_live_solana_settlement_ambiguous_after_submit();
    let context = build_live_solana_asset_movement_context(params_for("conflict", 37));
    let escrow_id = fund_live_escrow(&context.harness, 161, 37);
    let first = release_escrow_response_with_key(
        &context.harness.snapshot,
        context.harness.bind_addr.as_str(),
        context.harness.caller_did,
        163,
        escrow_id.as_str(),
        "settlement-conflict-original",
    );

    let conflict = release_escrow_response_with_key(
        &context.harness.snapshot,
        context.harness.bind_addr.as_str(),
        context.harness.caller_did,
        164,
        escrow_id.as_str(),
        "settlement-conflict-different",
    );

    assert!(first.contains("SETTLEMENT_OUTCOME_AMBIGUOUS"), "{first}");
    assert!(conflict.contains("HTTP/1.1 409 Conflict"), "{conflict}");
    assert!(
        conflict.contains("SETTLEMENT_INTENT_CONFLICT"),
        "{conflict}"
    );
    assert_eq!(
        crate::service_api_endpoint::test_live_solana_settlement_submission_count(),
        1
    );
}

#[test]
fn integration_settlement_rejects_mismatched_confirmed_evidence_without_release() {
    let (response, state, escrow_id) = mismatched_evidence_outcome();
    assert!(
        response.contains("SETTLEMENT_EVIDENCE_MISMATCH"),
        "{response}"
    );
    assert_eq!(state["settlement_intents"][&escrow_id]["state"], "failed");
    assert_eq!(
        state["settlement_intents"][&escrow_id]["submission_attempt_count"],
        1
    );
    assert_ne!(state["escrows"][&escrow_id]["state"], "released");
}

fn mismatched_evidence_outcome() -> (String, Value, String) {
    let _env = acquire_service_api_test_env();
    let _override_guard =
        crate::service_api_endpoint::set_test_live_solana_settlement_override(true);
    crate::service_api_endpoint::set_test_live_solana_settlement_evidence_mismatch();
    let context = build_live_solana_asset_movement_context(params_for("evidence-mismatch", 41));
    let escrow_id = fund_live_escrow(&context.harness, 171, 41);

    let response = release_with_key(&context, 173, &escrow_id, "settlement-evidence-mismatch");
    let state = read_state_json(context.harness.state_file.as_path());
    (response, state, escrow_id)
}

#[test]
fn integration_expired_ambiguous_settlement_fails_without_resubmission() {
    let outcome = expired_settlement_outcome();
    assert!(outcome.first.contains("SETTLEMENT_OUTCOME_AMBIGUOUS"));
    assert!(outcome.retry.contains("HTTP/1.1 409 Conflict"));
    assert!(outcome.retry.contains("SETTLEMENT_TRANSACTION_EXPIRED"));
    assert_eq!(
        outcome.state["settlement_intents"][&outcome.escrow_id]["state"],
        "failed"
    );
    assert_ne!(
        outcome.state["escrows"][&outcome.escrow_id]["state"],
        "released"
    );
    assert_eq!(outcome.submissions, 1);
}

struct ExpiredOutcome {
    first: String,
    retry: String,
    state: Value,
    escrow_id: String,
    submissions: u64,
}

fn expired_settlement_outcome() -> ExpiredOutcome {
    let _env = acquire_service_api_test_env();
    let _override_guard =
        crate::service_api_endpoint::set_test_live_solana_settlement_ambiguous_after_submit();
    let context = build_live_solana_asset_movement_context(params_for("expired", 47));
    let escrow_id = fund_live_escrow(&context.harness, 181, 47);
    let first = release_with_key(&context, 183, &escrow_id, "settlement-expired");
    crate::service_api_endpoint::set_test_live_solana_settlement_expired();
    let retry = release_with_key(&context, 184, &escrow_id, "settlement-expired");
    let state = read_state_json(context.harness.state_file.as_path());
    let submissions = crate::service_api_endpoint::test_live_solana_settlement_submission_count();
    ExpiredOutcome {
        first,
        retry,
        state,
        escrow_id,
        submissions,
    }
}

fn release_with_key(
    context: &LiveSolanaAssetMovementContext,
    nonce: u64,
    escrow_id: &str,
    key: &str,
) -> String {
    release_escrow_response_with_key(
        &context.harness.snapshot,
        context.harness.bind_addr.as_str(),
        context.harness.caller_did,
        nonce,
        escrow_id,
        key,
    )
}

fn params() -> LiveSolanaAssetMovementParams<'static> {
    params_for("reconciliation", 31)
}

fn params_for(label: &str, amount_lamports: u64) -> LiveSolanaAssetMovementParams<'_> {
    LiveSolanaAssetMovementParams {
        state_file_prefix: label,
        caller_did: "kamn:did:agent:test-client-settlement-reconciliation",
        api_bind: "127.0.0.1:34136",
        keypair_prefix: "kamn-node-settlement-reconciliation-keypair",
        keypair_env: KEYPAIR_ENV,
        recipient_env: RECIPIENT_ENV,
        lamports_env: LAMPORTS_ENV,
        live_rpc_env: RPC_URL,
        amount_lamports,
    }
}
