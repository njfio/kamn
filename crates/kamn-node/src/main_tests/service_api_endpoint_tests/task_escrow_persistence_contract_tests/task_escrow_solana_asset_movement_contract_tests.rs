use super::super::*;
use super::support::{
    assert_persisted_solana_signature_metadata,
    assert_released_escrow_has_solana_signature_metadata, build_live_solana_asset_movement_context,
    build_task_escrow_snapshot, fund_and_release_live_escrow, fund_live_escrow, read_state_json,
    release_escrow_response, release_live_escrow_across_restart, release_live_escrow_twice,
    set_live_solana_bridge_rpc_url_env, settlement_tx_signature, LiveSolanaAssetMovementParams,
};

const LIVE_SOLANA_DEVNET_RPC_URL: &str = "https://api.devnet.solana.com";
const SOLANA_SETTLEMENT_KEYPAIR_FILE_ENV: &str =
    "KAMN_SERVICE_API_LIVE_SOLANA_SETTLEMENT_KEYPAIR_FILE";
const SOLANA_SETTLEMENT_RECIPIENT_ENV: &str =
    "KAMN_SERVICE_API_LIVE_SOLANA_SETTLEMENT_RECIPIENT_PUBKEY";
const SOLANA_SETTLEMENT_LAMPORTS_ENV: &str = "KAMN_SERVICE_API_LIVE_SOLANA_SETTLEMENT_LAMPORTS";

#[test]
fn integration_service_api_endpoint_live_solana_asset_movement_lane_fails_at_startup_when_recipient_env_missing(
) {
    let _env = acquire_service_api_test_env();
    let _live_rpc_guard = set_live_solana_bridge_rpc_url_env(Some(LIVE_SOLANA_DEVNET_RPC_URL));
    let _keypair_guard = EnvVarGuard::set(
        SOLANA_SETTLEMENT_KEYPAIR_FILE_ENV,
        Some("/tmp/kamn-solana-keypair.json"),
    );
    let _lamports_guard = EnvVarGuard::set(SOLANA_SETTLEMENT_LAMPORTS_ENV, Some("1"));
    let snapshot = build_task_escrow_snapshot("127.0.0.1:34129");
    let endpoint_config = asset_movement_endpoint_config();

    let error = serve_service_api_endpoint(&endpoint_config, &snapshot)
        .expect_err("missing recipient env must fail loud at startup");

    assert!(
        error.contains(SOLANA_SETTLEMENT_RECIPIENT_ENV),
        "startup error must identify missing Solana recipient env: {error}"
    );
}

#[test]
fn integration_live_settlement_persists_ambiguous_outcome_without_release() {
    let _env = acquire_service_api_test_env();
    let _override_guard =
        crate::service_api_endpoint::set_test_live_solana_settlement_ambiguous_after_submit();
    let context = build_live_solana_asset_movement_context(LiveSolanaAssetMovementParams {
        state_file_prefix: "kamn-node-settlement-ambiguous-state",
        caller_did: "kamn:did:agent:test-client-settlement-ambiguous",
        api_bind: "127.0.0.1:34135",
        keypair_prefix: "kamn-node-settlement-ambiguous-keypair",
        keypair_env: SOLANA_SETTLEMENT_KEYPAIR_FILE_ENV,
        recipient_env: SOLANA_SETTLEMENT_RECIPIENT_ENV,
        lamports_env: SOLANA_SETTLEMENT_LAMPORTS_ENV,
        live_rpc_env: LIVE_SOLANA_DEVNET_RPC_URL,
        amount_lamports: 29,
    });
    let escrow_id = fund_live_escrow(&context.harness, 141, 29);

    let response = release_escrow_response(
        &context.harness.snapshot,
        context.harness.bind_addr.as_str(),
        context.harness.caller_did,
        143,
        escrow_id.as_str(),
    );
    let state = read_state_json(context.harness.state_file.as_path());

    assert!(
        response.contains("HTTP/1.1 503 Service Unavailable"),
        "{response}"
    );
    assert!(
        response.contains("SETTLEMENT_OUTCOME_AMBIGUOUS"),
        "{response}"
    );
    assert_eq!(
        state["settlement_intents"][&escrow_id]["state"],
        "ambiguous"
    );
    assert_ne!(state["escrows"][&escrow_id]["state"], "released");
}

#[test]
fn integration_service_api_endpoint_live_solana_asset_movement_release_persists_transaction_signature_metadata(
) {
    let _env = acquire_service_api_test_env();
    let _override_guard =
        crate::service_api_endpoint::set_test_live_solana_settlement_override(true);
    let context = build_live_solana_asset_movement_context(LiveSolanaAssetMovementParams {
        state_file_prefix: "kamn-node-solana-asset-movement-state",
        caller_did: "kamn:did:agent:test-client-solana-asset-movement",
        api_bind: "127.0.0.1:34130",
        keypair_prefix: "kamn-node-solana-asset-movement-keypair",
        keypair_env: SOLANA_SETTLEMENT_KEYPAIR_FILE_ENV,
        recipient_env: SOLANA_SETTLEMENT_RECIPIENT_ENV,
        lamports_env: SOLANA_SETTLEMENT_LAMPORTS_ENV,
        live_rpc_env: LIVE_SOLANA_DEVNET_RPC_URL,
        amount_lamports: 13,
    });
    let (escrow_id, released_escrow) = fund_and_release_live_escrow(&context.harness, 101, 103, 13);
    let state_json = read_state_json(context.harness.state_file.as_path());

    assert_released_escrow_has_solana_signature_metadata(&released_escrow);
    assert_eq!(released_escrow["claim_scope"], "devnet-backed");
    assert_persisted_solana_signature_metadata(&state_json, escrow_id.as_str());
}

#[test]
fn integration_live_settlement_persists_submitted_intent_before_adapter_submission() {
    let _env = acquire_service_api_test_env();
    let _override_guard =
        crate::service_api_endpoint::set_test_live_solana_settlement_override(true);
    let context = build_live_solana_asset_movement_context(LiveSolanaAssetMovementParams {
        state_file_prefix: "kamn-node-settlement-prepared-intent-state",
        caller_did: "kamn:did:agent:test-client-settlement-prepared-intent",
        api_bind: "127.0.0.1:34134",
        keypair_prefix: "kamn-node-settlement-prepared-intent-keypair",
        keypair_env: SOLANA_SETTLEMENT_KEYPAIR_FILE_ENV,
        recipient_env: SOLANA_SETTLEMENT_RECIPIENT_ENV,
        lamports_env: SOLANA_SETTLEMENT_LAMPORTS_ENV,
        live_rpc_env: LIVE_SOLANA_DEVNET_RPC_URL,
        amount_lamports: 23,
    });

    fund_and_release_live_escrow(&context.harness, 131, 133, 23);

    assert!(
        crate::service_api_endpoint::test_live_settlement_observed_submitted_intent(),
        "submitted intent and one attempt must be durable before adapter submission"
    );
}

#[test]
fn integration_service_api_endpoint_live_solana_asset_movement_release_is_idempotent_for_repeated_submit(
) {
    let _env = acquire_service_api_test_env();
    let _override_guard =
        crate::service_api_endpoint::set_test_live_solana_settlement_override(true);
    let context = build_live_solana_asset_movement_context(LiveSolanaAssetMovementParams {
        state_file_prefix: "kamn-node-solana-asset-movement-repeat-state",
        caller_did: "kamn:did:agent:test-client-solana-asset-movement-repeat",
        api_bind: "127.0.0.1:34131",
        keypair_prefix: "kamn-node-solana-asset-movement-repeat-keypair",
        keypair_env: SOLANA_SETTLEMENT_KEYPAIR_FILE_ENV,
        recipient_env: SOLANA_SETTLEMENT_RECIPIENT_ENV,
        lamports_env: SOLANA_SETTLEMENT_LAMPORTS_ENV,
        live_rpc_env: LIVE_SOLANA_DEVNET_RPC_URL,
        amount_lamports: 17,
    });
    let (first, second) = release_live_escrow_twice(&context.harness, 111, 113, 114, 17);

    assert_eq!(
        settlement_tx_signature(&first),
        settlement_tx_signature(&second),
        "repeated release must keep the same Solana transaction signature"
    );
}

#[test]
fn integration_service_api_endpoint_live_solana_asset_movement_release_reuses_signature_after_restart(
) {
    let _env = acquire_service_api_test_env();
    let _override_guard =
        crate::service_api_endpoint::set_test_live_solana_settlement_override(true);
    let context = build_live_solana_asset_movement_context(LiveSolanaAssetMovementParams {
        state_file_prefix: "kamn-node-solana-asset-movement-restart-state",
        caller_did: "kamn:did:agent:test-client-solana-asset-movement-restart",
        api_bind: "127.0.0.1:34132",
        keypair_prefix: "kamn-node-solana-asset-movement-restart-keypair",
        keypair_env: SOLANA_SETTLEMENT_KEYPAIR_FILE_ENV,
        recipient_env: SOLANA_SETTLEMENT_RECIPIENT_ENV,
        lamports_env: SOLANA_SETTLEMENT_LAMPORTS_ENV,
        live_rpc_env: LIVE_SOLANA_DEVNET_RPC_URL,
        amount_lamports: 19,
    });
    let (first, second) =
        release_live_escrow_across_restart(&context.harness, "127.0.0.1:34133", 121, 123, 124, 19);

    assert_eq!(
        settlement_tx_signature(&first),
        settlement_tx_signature(&second)
    );
    assert_eq!(second["settlement_network"], "solana:devnet");
    assert_eq!(second["settlement_commitment"], "finalized");
}
fn asset_movement_endpoint_config() -> ServiceApiEndpointConfig {
    ServiceApiEndpointConfig {
        bind_addr: reserve_loopback_addr(),
        max_requests: 1,
        idle_timeout_ms: 1,
        body_limit_bytes: DEFAULT_SERVICE_API_BODY_LIMIT_BYTES,
        concurrency_limit: DEFAULT_SERVICE_API_CONCURRENCY_LIMIT,
        rate_limit_per_second: DEFAULT_SERVICE_API_RATE_LIMIT_PER_SECOND,
    }
}
