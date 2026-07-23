use super::super::super::*;
use super::{
    accept_task, build_task_escrow_snapshot, complete_task, create_task, fund_escrow,
    release_escrow, set_state_file_env, unique_named_state_file,
};
use crate::service_api_endpoint::ServiceApiSnapshot;
use solana_sdk::signer::keypair::{write_keypair_file, Keypair};
use solana_sdk::signer::Signer;

#[path = "solana_asset_movement_support/assertions.rs"]
mod assertions;
#[path = "solana_asset_movement_support/context.rs"]
mod context;

pub(crate) use assertions::{
    assert_persisted_solana_signature_metadata, assert_released_escrow_has_durable_authority,
    assert_released_escrow_has_solana_signature_metadata, cleanup_solana_settlement_fixture,
    cleanup_state_file, settlement_tx_signature,
};
pub(crate) use context::{
    build_live_solana_asset_movement_context, release_live_escrow_across_restart,
    LiveSolanaAssetMovementParams,
};

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

pub(crate) fn fund_live_escrow(harness: &AssetMovementHarness, nonce: u64, amount: u64) -> String {
    let task = create_task(
        &harness.snapshot,
        harness.bind_addr.as_str(),
        harness.caller_did,
        nonce - 3,
        r#"{"description":"solana asset movement task"}"#,
    );
    accept_task(
        &harness.snapshot,
        harness.bind_addr.as_str(),
        harness.caller_did,
        nonce - 2,
        task.task_id.as_str(),
    );
    let payload = format!(r#"{{"task_id":"{}","amount":{amount}}}"#, task.task_id);
    let funded_escrow = fund_escrow(
        &harness.snapshot,
        harness.bind_addr.as_str(),
        harness.caller_did,
        nonce,
        payload.as_str(),
    );
    complete_task(
        &harness.snapshot,
        harness.bind_addr.as_str(),
        harness.caller_did,
        nonce + 1,
        task.task_id.as_str(),
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
