#[path = "support/authorization_fixture.rs"]
mod authorization_fixture;
#[path = "support/env_support.rs"]
mod env_support;
#[path = "support/request_support.rs"]
mod request_support;
#[path = "support/solana_asset_movement_support.rs"]
mod solana_asset_movement_support;
#[path = "support/state_support.rs"]
mod state_support;

pub(super) use env_support::{
    default_audit_export_file, read_audit_export_json, read_state_json, set_audit_export_file_env,
    set_live_solana_bridge_rpc_url_env,
};
pub(super) use request_support::{
    accept_task, authorized_signed_request, complete_task, create_task, fund_escrow, query_task,
    raw_signed_request, register_agent_profile, release_escrow, release_escrow_response,
    release_escrow_response_with_key, SignedRequest,
};
pub(super) use solana_asset_movement_support::{
    assert_persisted_solana_signature_metadata,
    assert_released_escrow_has_solana_signature_metadata, build_live_solana_asset_movement_context,
    fund_and_release_live_escrow, fund_live_escrow, release_live_escrow_across_restart,
    release_live_escrow_twice, settlement_tx_signature, LiveSolanaAssetMovementParams,
};
pub(super) use state_support::{
    build_task_escrow_snapshot, set_state_file_env, state_hash, unique_named_state_file,
    with_api_server,
};

use crate::service_api_endpoint::ServiceApiSnapshot;

pub(super) fn raw_create_task_response(
    snapshot: &ServiceApiSnapshot,
    bind_addr: &str,
    caller_did: &str,
    nonce: u64,
    payload: &str,
) -> String {
    request_support::authorized_signed_request(
        snapshot,
        bind_addr,
        request_support::SignedRequest {
            max_requests: 1,
            method: "POST",
            path: "/v1/tasks/create",
            caller_did,
            nonce,
            body: payload,
            extra_headers: &[],
        },
    )
}
