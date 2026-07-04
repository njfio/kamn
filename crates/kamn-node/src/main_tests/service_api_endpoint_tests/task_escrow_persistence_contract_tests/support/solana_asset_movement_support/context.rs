use super::super::super::super::*;
use super::super::{build_task_escrow_snapshot, set_live_solana_bridge_rpc_url_env};
use super::{
    build_asset_movement_harness, build_solana_settlement_fixture,
    cleanup_solana_settlement_fixture, cleanup_state_file, fund_live_escrow, release_escrow,
    AssetMovementHarness, LiveSolanaAssetMovementContext, SolanaSettlementFixture, Value,
};

struct LiveSolanaAssetMovementGuards {
    live_rpc_guard: EnvVarGuard,
    keypair_guard: EnvVarGuard,
    recipient_guard: EnvVarGuard,
    lamports_guard: EnvVarGuard,
}

pub(crate) struct LiveSolanaAssetMovementParams<'a> {
    pub(crate) state_file_prefix: &'a str,
    pub(crate) caller_did: &'static str,
    pub(crate) api_bind: &'a str,
    pub(crate) keypair_prefix: &'a str,
    pub(crate) keypair_env: &'static str,
    pub(crate) recipient_env: &'static str,
    pub(crate) lamports_env: &'static str,
    pub(crate) live_rpc_env: &'static str,
}

pub(crate) fn build_live_solana_asset_movement_context(
    params: LiveSolanaAssetMovementParams<'_>,
) -> LiveSolanaAssetMovementContext {
    let fixture = build_solana_settlement_fixture(params.keypair_prefix);
    let guards = build_live_solana_asset_movement_guards(
        &fixture,
        params.keypair_env,
        params.recipient_env,
        params.lamports_env,
        params.live_rpc_env,
    );
    let harness =
        build_asset_movement_harness(params.state_file_prefix, params.caller_did, params.api_bind);
    live_solana_asset_movement_context(harness, fixture, guards)
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
    let first = release_harness_escrow(harness, first_release_nonce, escrow_id.as_str());
    let second = release_restart_escrow(
        harness.caller_did,
        restart_bind,
        second_release_nonce,
        escrow_id.as_str(),
    );
    (first, second)
}

impl Drop for LiveSolanaAssetMovementContext {
    fn drop(&mut self) {
        cleanup_solana_settlement_fixture(&self.fixture);
        cleanup_state_file(self.harness.state_file.as_path());
    }
}

fn build_live_solana_asset_movement_guards(
    fixture: &SolanaSettlementFixture,
    keypair_env: &'static str,
    recipient_env: &'static str,
    lamports_env: &'static str,
    live_rpc_env: &'static str,
) -> LiveSolanaAssetMovementGuards {
    LiveSolanaAssetMovementGuards {
        live_rpc_guard: set_live_solana_bridge_rpc_url_env(Some(live_rpc_env)),
        keypair_guard: EnvVarGuard::set(keypair_env, Some(fixture.keypair_file_text.as_str())),
        recipient_guard: EnvVarGuard::set(recipient_env, Some(fixture.recipient_pubkey.as_str())),
        lamports_guard: EnvVarGuard::set(lamports_env, Some("1")),
    }
}

fn live_solana_asset_movement_context(
    harness: AssetMovementHarness,
    fixture: SolanaSettlementFixture,
    guards: LiveSolanaAssetMovementGuards,
) -> LiveSolanaAssetMovementContext {
    LiveSolanaAssetMovementContext {
        harness,
        _live_rpc_guard: guards.live_rpc_guard,
        _keypair_guard: guards.keypair_guard,
        _recipient_guard: guards.recipient_guard,
        _lamports_guard: guards.lamports_guard,
        fixture,
    }
}

fn release_harness_escrow(harness: &AssetMovementHarness, nonce: u64, escrow_id: &str) -> Value {
    release_escrow(
        &harness.snapshot,
        harness.bind_addr.as_str(),
        harness.caller_did,
        nonce,
        escrow_id,
    )
}

fn release_restart_escrow(
    caller_did: &'static str,
    restart_bind: &str,
    nonce: u64,
    escrow_id: &str,
) -> Value {
    let restart_snapshot = build_task_escrow_snapshot(restart_bind);
    release_escrow(
        &restart_snapshot,
        reserve_loopback_addr().as_str(),
        caller_did,
        nonce,
        escrow_id,
    )
}
