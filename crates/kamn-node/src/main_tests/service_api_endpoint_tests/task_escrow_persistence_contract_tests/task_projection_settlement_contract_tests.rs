use super::super::*;
use super::support::*;

const RPC_URL: &str = "https://api.devnet.solana.com";
const KEYPAIR_ENV: &str = "KAMN_SERVICE_API_LIVE_SOLANA_SETTLEMENT_KEYPAIR_FILE";
const RECIPIENT_ENV: &str = "KAMN_SERVICE_API_LIVE_SOLANA_SETTLEMENT_RECIPIENT_PUBKEY";
const LAMPORTS_ENV: &str = "KAMN_SERVICE_API_LIVE_SOLANA_SETTLEMENT_LAMPORTS";
const ACTOR: &str = "kamn:did:agent:projection-settlement";

#[test]
fn integration_participant_projection_uses_persisted_settlement_evidence() {
    let _env = acquire_service_api_test_env();
    let _override_guard =
        crate::service_api_endpoint::set_test_live_solana_settlement_override(true);
    let context = build_live_solana_asset_movement_context(LiveSolanaAssetMovementParams {
        state_file_prefix: "kamn-projection-settlement-state",
        caller_did: ACTOR,
        api_bind: "127.0.0.1:34251",
        keypair_prefix: "kamn-projection-settlement-keypair",
        keypair_env: KEYPAIR_ENV,
        recipient_env: RECIPIENT_ENV,
        lamports_env: LAMPORTS_ENV,
        live_rpc_env: RPC_URL,
        amount_lamports: 31,
    });
    let (escrow_id, released) = fund_and_release_live_escrow(&context.harness, 181, 183, 31);
    let state = read_state_json(context.harness.state_file.as_path());
    let task_id = state["escrows"][&escrow_id]["task_id"]
        .as_str()
        .expect("task id");
    let path = format!("/v1/tasks/{task_id}/participant-view");

    let response = raw_signed_request(
        &context.harness.snapshot,
        context.harness.bind_addr.as_str(),
        SignedRequest {
            max_requests: 1,
            method: "GET",
            path: path.as_str(),
            caller_did: ACTOR,
            nonce: 184,
            body: "",
            extra_headers: &[("X-KAMN-Authz-Scope", "tasks:read")],
        },
    );
    let projection: Value =
        parse_service_api_payload(extract_http_response_body(&response)).expect("projection");

    assert!(response.contains("HTTP/1.1 200 OK"), "{response}");
    assert_eq!(projection["escrow_state"], "released");
    assert_eq!(projection["network"], "solana-devnet");
    assert_eq!(
        projection["settlement_tx_signature"],
        released["settlement_tx_signature"]
    );
    assert_eq!(projection["settlement_commitment"], "finalized");
}
