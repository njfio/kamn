use super::super::super::*;
use super::{
    build_task_escrow_snapshot, fund_escrow, release_escrow, set_state_file_env,
    set_live_solana_bridge_rpc_url_env, unique_named_state_file,
};
use crate::service_api_endpoint::ServiceApiSnapshot;
use solana_sdk::signer::keypair::{write_keypair_file, Keypair};
use solana_sdk::signer::Signer;

pub(crate) struct AssetMovementHarness {
    pub(crate) state_file: std::path::PathBuf,
    _state_file_text: String,
    _state_file_guard: EnvVarGuard,
    pub(crate) snapshot: ServiceApiSnapshot,
    pub(crate) bind_addr: String,
    pub(crate) caller_did: &'static str,
}

pub(crate) struct SolanaSettlementFixture {
    keypair_file: std::path::PathBuf,
    pub(crate) keypair_file_text: String,
    pub(crate) recipient_pubkey: String,
}

pub(crate) struct LiveSolanaAssetMovementContext {
    pub(crate) harness: AssetMovementHarness,
    _live_rpc_guard: EnvVarGuard,
    _keypair_guard: EnvVarGuard,
    _recipient_guard: EnvVarGuard,
    _lamports_guard: EnvVarGuard,
    fixture: SolanaSettlementFixture,
}

pub(crate) fn build_asset_movement_harness(
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

pub(crate) fn fund_and_release_live_escrow(
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

pub(crate) fn fund_live_escrow(
    harness: &AssetMovementHarness,
    nonce: u64,
    amount: u64,
) -> String {
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

pub(crate) fn build_solana_settlement_fixture(prefix: &str) -> SolanaSettlementFixture {
    let keypair_file = unique_named_state_file(prefix);
    let keypair = Keypair::new();
    write_keypair_file(&keypair, keypair_file.as_path())
        .expect("test settlement keypair should write");
    let recipient = Keypair::new();
    SolanaSettlementFixture {
        keypair_file_text: keypair_file.to_string_lossy().to_string(),
        keypair_file,
        recipient_pubkey: recipient.pubkey().to_string(),
    }
}

pub(crate) fn build_live_solana_asset_movement_context(
    state_file_prefix: &str,
    caller_did: &'static str,
    api_bind: &str,
    keypair_prefix: &str,
    keypair_env: &'static str,
    recipient_env: &'static str,
    lamports_env: &'static str,
    live_rpc_env: &'static str,
) -> LiveSolanaAssetMovementContext {
    let live_rpc_guard = set_live_solana_bridge_rpc_url_env(Some(live_rpc_env));
    let fixture = build_solana_settlement_fixture(keypair_prefix);
    let keypair_guard = EnvVarGuard::set(keypair_env, Some(fixture.keypair_file_text.as_str()));
    let recipient_guard =
        EnvVarGuard::set(recipient_env, Some(fixture.recipient_pubkey.as_str()));
    let lamports_guard = EnvVarGuard::set(lamports_env, Some("1"));
    let harness = build_asset_movement_harness(state_file_prefix, caller_did, api_bind);
    LiveSolanaAssetMovementContext {
        harness,
        _live_rpc_guard: live_rpc_guard,
        _keypair_guard: keypair_guard,
        _recipient_guard: recipient_guard,
        _lamports_guard: lamports_guard,
        fixture,
    }
}

pub(crate) fn release_live_escrow_twice(
    harness: &AssetMovementHarness,
    fund_nonce: u64,
    first_release_nonce: u64,
    second_release_nonce: u64,
    amount: u64,
) -> (Value, Value) {
    let escrow_id = fund_live_escrow(harness, fund_nonce, amount);
    let first = release_escrow(
        &harness.snapshot,
        harness.bind_addr.as_str(),
        harness.caller_did,
        first_release_nonce,
        escrow_id.as_str(),
    );
    let second = release_escrow(
        &harness.snapshot,
        harness.bind_addr.as_str(),
        harness.caller_did,
        second_release_nonce,
        escrow_id.as_str(),
    );
    (first, second)
}

pub(crate) fn release_live_escrow_across_restart(
    harness: &AssetMovementHarness,
    restart_bind: &str,
    fund_nonce: u64,
    first_release_nonce: u64,
    second_release_nonce: u64,
    amount: u64,
) -> (Value, Value) {
    let escrow_id = fund_live_escrow(harness, fund_nonce, amount);
    let first = release_escrow(
        &harness.snapshot,
        harness.bind_addr.as_str(),
        harness.caller_did,
        first_release_nonce,
        escrow_id.as_str(),
    );
    let restart_snapshot = build_task_escrow_snapshot(restart_bind);
    let second = release_escrow(
        &restart_snapshot,
        reserve_loopback_addr().as_str(),
        harness.caller_did,
        second_release_nonce,
        escrow_id.as_str(),
    );
    (first, second)
}

pub(crate) fn assert_released_escrow_has_solana_signature_metadata(released_escrow: &Value) {
    assert_eq!(released_escrow["state"], "released");
    assert_eq!(released_escrow["settlement_network"], "solana:devnet");
    assert_eq!(released_escrow["settlement_commitment"], "finalized");
    assert_base58ish_signature(settlement_tx_signature(released_escrow));
}

pub(crate) fn assert_persisted_solana_signature_metadata(
    state_json: &Value,
    escrow_id: &str,
) {
    let persisted = &state_json["escrows"][escrow_id];
    assert_eq!(persisted["state"], "released");
    assert_eq!(persisted["settlement_network"], "solana:devnet");
    assert_eq!(persisted["settlement_commitment"], "finalized");
    assert_base58ish_signature(settlement_tx_signature(persisted));
}

pub(crate) fn settlement_tx_signature(payload: &Value) -> &str {
    payload["settlement_tx_signature"]
        .as_str()
        .expect("release payload must expose a Solana transaction signature")
}

pub(crate) fn cleanup_state_file(path: &std::path::Path) {
    let _ = fs::remove_file(path);
}

pub(crate) fn cleanup_solana_settlement_fixture(fixture: &SolanaSettlementFixture) {
    let _ = fs::remove_file(fixture.keypair_file.as_path());
}

impl Drop for LiveSolanaAssetMovementContext {
    fn drop(&mut self) {
        cleanup_solana_settlement_fixture(&self.fixture);
        cleanup_state_file(self.harness.state_file.as_path());
    }
}

fn assert_base58ish_signature(signature: &str) {
    let valid = !signature.is_empty()
        && signature.chars().all(
            |ch| matches!(ch, '1'..='9' | 'A'..='H' | 'J'..='N' | 'P'..='Z' | 'a'..='k' | 'm'..='z'),
        );
    assert!(valid, "expected a base58ish Solana signature, got: {signature}");
}
