use super::super::*;
use super::support::{
    build_task_escrow_snapshot, fund_escrow, read_state_json, release_escrow,
    set_live_solana_bridge_rpc_url_env, set_state_file_env, unique_named_state_file,
};
use crate::service_api_endpoint::ServiceApiSnapshot;

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
    let _live_rpc_guard =
        set_live_solana_bridge_rpc_url_env(Some(LIVE_SOLANA_DEVNET_RPC_URL));
    let _keypair_guard =
        EnvVarGuard::set(SOLANA_SETTLEMENT_KEYPAIR_FILE_ENV, Some("/tmp/kamn-solana-keypair.json"));
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
fn integration_service_api_endpoint_live_solana_asset_movement_release_persists_transaction_signature_metadata(
) {
    let _env = acquire_service_api_test_env();
    let _live_rpc_guard =
        set_live_solana_bridge_rpc_url_env(Some(LIVE_SOLANA_DEVNET_RPC_URL));
    let _keypair_guard =
        EnvVarGuard::set(SOLANA_SETTLEMENT_KEYPAIR_FILE_ENV, Some("/tmp/kamn-solana-keypair.json"));
    let _recipient_guard = EnvVarGuard::set(
        SOLANA_SETTLEMENT_RECIPIENT_ENV,
        Some("7Yv7xQ6Qk6i2JZp4Y2yXb4m7fK3rN3mQ7H5qA4d9pW1K"),
    );
    let _lamports_guard = EnvVarGuard::set(SOLANA_SETTLEMENT_LAMPORTS_ENV, Some("1"));
    let harness = build_asset_movement_harness(
        "kamn-node-solana-asset-movement-state",
        "kamn:did:agent:test-client-solana-asset-movement",
        "127.0.0.1:34130",
    );
    let (escrow_id, released_escrow) = fund_and_release_live_escrow(&harness, 101, 102, 13);
    let state_json = read_state_json(harness.state_file.as_path());

    assert_released_escrow_has_solana_signature_metadata(&released_escrow);
    assert_persisted_solana_signature_metadata(&state_json, escrow_id.as_str());
    cleanup_state_file(harness.state_file.as_path());
}

#[test]
fn integration_service_api_endpoint_live_solana_asset_movement_release_is_idempotent_for_repeated_submit(
) {
    let _env = acquire_service_api_test_env();
    let _live_rpc_guard =
        set_live_solana_bridge_rpc_url_env(Some(LIVE_SOLANA_DEVNET_RPC_URL));
    let _keypair_guard =
        EnvVarGuard::set(SOLANA_SETTLEMENT_KEYPAIR_FILE_ENV, Some("/tmp/kamn-solana-keypair.json"));
    let _recipient_guard = EnvVarGuard::set(
        SOLANA_SETTLEMENT_RECIPIENT_ENV,
        Some("7Yv7xQ6Qk6i2JZp4Y2yXb4m7fK3rN3mQ7H5qA4d9pW1K"),
    );
    let _lamports_guard = EnvVarGuard::set(SOLANA_SETTLEMENT_LAMPORTS_ENV, Some("1"));
    let harness = build_asset_movement_harness(
        "kamn-node-solana-asset-movement-repeat-state",
        "kamn:did:agent:test-client-solana-asset-movement-repeat",
        "127.0.0.1:34131",
    );
    let escrow_id = fund_live_escrow(&harness, 111, 17);
    let first = release_escrow(
        &harness.snapshot,
        harness.bind_addr.as_str(),
        harness.caller_did,
        112,
        escrow_id.as_str(),
    );
    let second = release_escrow(
        &harness.snapshot,
        harness.bind_addr.as_str(),
        harness.caller_did,
        113,
        escrow_id.as_str(),
    );

    assert_eq!(
        settlement_tx_signature(&first),
        settlement_tx_signature(&second),
        "repeated release must keep the same Solana transaction signature"
    );
    cleanup_state_file(harness.state_file.as_path());
}

struct AssetMovementHarness {
    state_file: std::path::PathBuf,
    _state_file_text: String,
    _state_file_guard: EnvVarGuard,
    snapshot: ServiceApiSnapshot,
    bind_addr: String,
    caller_did: &'static str,
}

fn build_asset_movement_harness(
    state_file_prefix: &str,
    caller_did: &'static str,
    api_bind: &str,
) -> AssetMovementHarness {
    let state_file = unique_named_state_file(state_file_prefix);
    let (state_file_text, state_file_guard) = set_state_file_env(state_file.as_path());
    AssetMovementHarness {
        state_file,
        _state_file_text: state_file_text,
        _state_file_guard: state_file_guard,
        snapshot: build_task_escrow_snapshot(api_bind),
        bind_addr: reserve_loopback_addr(),
        caller_did,
    }
}

fn fund_and_release_live_escrow(
    harness: &AssetMovementHarness,
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

fn fund_live_escrow(harness: &AssetMovementHarness, nonce: u64, amount: u64) -> String {
    let payload = format!(r#"{{"task_id":"solana-asset-movement-task","amount":{amount}}}"#);
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

fn assert_released_escrow_has_solana_signature_metadata(released_escrow: &Value) {
    assert_eq!(released_escrow["state"], "released");
    assert_eq!(released_escrow["settlement_network"], "solana:devnet");
    assert_eq!(released_escrow["settlement_commitment"], "finalized");
    assert_base58ish_signature(settlement_tx_signature(released_escrow));
}

fn assert_persisted_solana_signature_metadata(state_json: &Value, escrow_id: &str) {
    let persisted = &state_json["escrows"][escrow_id];
    assert_eq!(persisted["state"], "released");
    assert_eq!(persisted["settlement_network"], "solana:devnet");
    assert_eq!(persisted["settlement_commitment"], "finalized");
    assert_base58ish_signature(settlement_tx_signature(persisted));
}

fn settlement_tx_signature(payload: &Value) -> &str {
    payload["settlement_tx_signature"]
        .as_str()
        .expect("release payload must expose a Solana transaction signature")
}

fn assert_base58ish_signature(signature: &str) {
    let valid = !signature.is_empty()
        && signature
            .chars()
            .all(|ch| matches!(ch, '1'..='9' | 'A'..='H' | 'J'..='N' | 'P'..='Z' | 'a'..='k' | 'm'..='z'));
    assert!(valid, "expected a base58ish Solana signature, got: {signature}");
}

fn cleanup_state_file(path: &std::path::Path) {
    let _ = fs::remove_file(path);
}
