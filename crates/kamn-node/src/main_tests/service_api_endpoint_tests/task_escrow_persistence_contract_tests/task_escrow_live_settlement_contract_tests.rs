use super::super::*;
use super::support::{
    build_task_escrow_snapshot, fund_escrow, raw_signed_request, read_state_json, release_escrow,
    set_live_solana_bridge_rpc_url_env, set_state_file_env, unique_named_state_file,
};

const LIVE_SOLANA_DEVNET_RPC_URL: &str = "https://api.devnet.solana.com";
const UNREACHABLE_SOLANA_RPC_URL: &str = "http://127.0.0.1:1";

#[test]
fn integration_service_api_endpoint_live_settlement_release_persists_external_receipt_linkage() {
    let _env = acquire_service_api_test_env();
    let _live_rpc_guard =
        set_live_solana_bridge_rpc_url_env(Some(LIVE_SOLANA_DEVNET_RPC_URL));
    let state_file = unique_named_state_file("kamn-node-live-settlement-state");
    let (_state_file_text, _state_file_guard) = set_state_file_env(state_file.as_path());
    let caller_did = "kamn:did:agent:test-client-live-settlement";
    let first_snapshot = build_task_escrow_snapshot("127.0.0.1:34126");
    let bind_addr = reserve_loopback_addr();

    let funded_escrow = fund_escrow(
        &first_snapshot,
        bind_addr.as_str(),
        caller_did,
        81,
        r#"{"task_id":"live-settlement-task","amount":9}"#,
    );
    let escrow_id = funded_escrow["escrow_id"]
        .as_str()
        .expect("escrow id should be string");
    let released_escrow =
        release_escrow(&first_snapshot, bind_addr.as_str(), caller_did, 82, escrow_id);

    let receipt_hash = released_escrow["settlement_receipt_hash"]
        .as_str()
        .expect("live settlement release should expose receipt hash");
    assert!(
        !receipt_hash.starts_with("sha256:placeholder"),
        "external settlement receipt hash must not be placeholder"
    );
    assert!(
        !receipt_hash.trim().is_empty(),
        "external settlement receipt hash must not be empty"
    );

    let restart_snapshot = build_task_escrow_snapshot("127.0.0.1:34127");
    let _ = restart_snapshot;
    let state_json = read_state_json(state_file.as_path());

    assert_eq!(state_json["escrows"][escrow_id]["state"], "released");
    assert_eq!(
        state_json["escrows"][escrow_id]["settlement_receipt_hash"],
        receipt_hash
    );
    let _ = fs::remove_file(state_file);
}

#[test]
fn integration_service_api_endpoint_live_settlement_release_fails_loud_for_unreachable_rpc() {
    let _env = acquire_service_api_test_env();
    let _live_rpc_guard = set_live_solana_bridge_rpc_url_env(Some(UNREACHABLE_SOLANA_RPC_URL));
    let state_file = unique_named_state_file("kamn-node-live-settlement-error-state");
    let (_state_file_text, _state_file_guard) = set_state_file_env(state_file.as_path());
    let caller_did = "kamn:did:agent:test-client-live-settlement-error";
    let snapshot = build_task_escrow_snapshot("127.0.0.1:34128");
    let bind_addr = reserve_loopback_addr();

    let funded_escrow = fund_escrow(
        &snapshot,
        bind_addr.as_str(),
        caller_did,
        91,
        r#"{"task_id":"live-settlement-task","amount":11}"#,
    );
    let escrow_id = funded_escrow["escrow_id"]
        .as_str()
        .expect("escrow id should be string");
    let response = raw_signed_request(
        &snapshot,
        bind_addr.as_str(),
        1,
        "POST",
        format!("/v1/escrow/{escrow_id}/release").as_str(),
        caller_did,
        92,
        "",
        &[],
    );

    assert!(response.contains("HTTP/1.1 500 Internal Server Error"));
    assert!(response.contains("service_api_live_settlement_evidence_failed"));
    assert!(response.contains("service api live settlement evidence failed"));
    let _ = fs::remove_file(state_file);
}
