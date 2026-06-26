use super::super::*;
use super::support::{
    build_task_escrow_snapshot, fund_escrow, raw_signed_request, read_state_json, release_escrow,
    set_live_solana_bridge_rpc_url_env, set_state_file_env, unique_named_state_file,
};
use crate::service_api_endpoint::ServiceApiSnapshot;

const LIVE_SOLANA_DEVNET_RPC_URL: &str = "https://api.devnet.solana.com";
const UNREACHABLE_SOLANA_RPC_URL: &str = "http://127.0.0.1:1";

#[test]
fn integration_service_api_endpoint_live_settlement_release_persists_external_receipt_linkage() {
    let _env = acquire_service_api_test_env();
    let _live_rpc_guard = set_live_solana_bridge_rpc_url_env(Some(LIVE_SOLANA_DEVNET_RPC_URL));
    let harness = build_live_settlement_harness(
        "kamn-node-live-settlement-state",
        "kamn:did:agent:test-client-live-settlement",
        "127.0.0.1:34126",
    );
    let (escrow_id, released_escrow) = fund_and_release_live_escrow(&harness, 81, 82, 9);
    let receipt_hash = settlement_receipt_hash(&released_escrow);
    let state_json = read_state_json(harness.state_file.as_path());

    assert_persisted_live_receipt(&state_json, escrow_id.as_str(), receipt_hash.as_str());
    cleanup_state_file(harness.state_file.as_path());
}

#[test]
fn integration_service_api_endpoint_live_settlement_release_fails_loud_for_unreachable_rpc() {
    let _env = acquire_service_api_test_env();
    let _live_rpc_guard = set_live_solana_bridge_rpc_url_env(Some(UNREACHABLE_SOLANA_RPC_URL));
    let harness = build_live_settlement_harness(
        "kamn-node-live-settlement-error-state",
        "kamn:did:agent:test-client-live-settlement-error",
        "127.0.0.1:34128",
    );
    let response = release_live_escrow_with_unreachable_rpc(&harness, 91, 92, 11);

    assert_failed_live_release(response.as_str());
    cleanup_state_file(harness.state_file.as_path());
}

struct LiveSettlementHarness {
    state_file: std::path::PathBuf,
    _state_file_text: String,
    _state_file_guard: EnvVarGuard,
    snapshot: ServiceApiSnapshot,
    bind_addr: String,
    caller_did: &'static str,
}

fn build_live_settlement_harness(
    state_file_prefix: &str,
    caller_did: &'static str,
    api_bind: &str,
) -> LiveSettlementHarness {
    let state_file = unique_named_state_file(state_file_prefix);
    let (state_file_text, state_file_guard) = set_state_file_env(state_file.as_path());
    LiveSettlementHarness {
        state_file,
        _state_file_text: state_file_text,
        _state_file_guard: state_file_guard,
        snapshot: build_task_escrow_snapshot(api_bind),
        bind_addr: reserve_loopback_addr(),
        caller_did,
    }
}

fn fund_and_release_live_escrow(
    harness: &LiveSettlementHarness,
    fund_nonce: u64,
    release_nonce: u64,
    amount: u64,
) -> (String, Value) {
    let escrow_id = fund_live_escrow(harness, fund_nonce, amount);
    let released = release_escrow(
        &harness.snapshot,
        harness.bind_addr.as_str(),
        harness.caller_did,
        release_nonce,
        escrow_id.as_str(),
    );
    (escrow_id, released)
}

fn fund_live_escrow(harness: &LiveSettlementHarness, nonce: u64, amount: u64) -> String {
    let payload = format!(r#"{{"task_id":"live-settlement-task","amount":{amount}}}"#);
    let funded_escrow = fund_escrow(
        &harness.snapshot,
        harness.bind_addr.as_str(),
        harness.caller_did,
        nonce,
        payload.as_str(),
    );
    funded_escrow["escrow_id"]
        .as_str()
        .expect("escrow id should be string")
        .to_owned()
}

fn release_live_escrow_with_unreachable_rpc(
    harness: &LiveSettlementHarness,
    fund_nonce: u64,
    release_nonce: u64,
    amount: u64,
) -> String {
    let escrow_id = fund_live_escrow(harness, fund_nonce, amount);
    raw_signed_request(
        &harness.snapshot,
        harness.bind_addr.as_str(),
        1,
        "POST",
        format!("/v1/escrow/{escrow_id}/release").as_str(),
        harness.caller_did,
        release_nonce,
        "",
        &[],
    )
}

fn settlement_receipt_hash(released_escrow: &Value) -> String {
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
    receipt_hash.to_owned()
}

fn assert_persisted_live_receipt(state_json: &Value, escrow_id: &str, receipt_hash: &str) {
    assert_eq!(state_json["escrows"][escrow_id]["state"], "released");
    assert_eq!(
        state_json["escrows"][escrow_id]["settlement_receipt_hash"],
        receipt_hash
    );
}

fn assert_failed_live_release(response: &str) {
    assert!(response.contains("HTTP/1.1 500 Internal Server Error"));
    assert!(response.contains("service_api_live_settlement_evidence_failed"));
    assert!(response.contains("service api live settlement evidence failed"));
}

fn cleanup_state_file(path: &std::path::Path) {
    let _ = fs::remove_file(path);
}
