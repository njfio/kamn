use super::super::super::*;
use super::super::support::{
    authorized_signed_request, build_live_solana_asset_movement_context,
    build_task_escrow_snapshot, raw_signed_request, register_agent_profile, AssetMovementHarness,
    LiveSolanaAssetMovementContext, LiveSolanaAssetMovementParams, SignedRequest,
};
use serde_json::{json, Value};

#[path = "task_escrow_bridge_authority_seed.rs"]
mod seed;
pub(super) use seed::{
    clone_replay_target, seed_finalized_bridge_receipt, seed_finalized_bridge_receipt_with,
};

pub(super) const ACTOR: &str = "kamn:did:agent:bridge-authority";
pub(super) const VERIFIER: &str = "kamn:did:agent:bridge-verifier";
pub(super) const RECIPIENT_ENV: &str = "KAMN_SERVICE_API_LIVE_SOLANA_SETTLEMENT_RECIPIENT_PUBKEY";
const RPC_URL: &str = "https://api.devnet.solana.com";
const KEYPAIR_ENV: &str = "KAMN_SERVICE_API_LIVE_SOLANA_SETTLEMENT_KEYPAIR_FILE";
const LAMPORTS_ENV: &str = "KAMN_SERVICE_API_LIVE_SOLANA_SETTLEMENT_LAMPORTS";

pub(super) fn bridge_context(
    label: &str,
    api_bind: &str,
    amount_lamports: u64,
) -> LiveSolanaAssetMovementContext {
    build_live_solana_asset_movement_context(LiveSolanaAssetMovementParams {
        state_file_prefix: label,
        caller_did: ACTOR,
        api_bind,
        keypair_prefix: label,
        keypair_env: KEYPAIR_ENV,
        recipient_env: RECIPIENT_ENV,
        lamports_env: LAMPORTS_ENV,
        live_rpc_env: RPC_URL,
        amount_lamports,
    })
}

pub(super) fn register_verifier(harness: &AssetMovementHarness, nonce: u64) {
    register_agent_profile(
        &harness.snapshot,
        harness.bind_addr.as_str(),
        VERIFIER,
        nonce,
        r#"{"agent_type":"verifier","model_family":"test","capabilities":["tasks"]}"#,
    );
}

pub(super) fn release_with_bridge_authority(
    harness: &AssetMovementHarness,
    nonce: u64,
    escrow_id: &str,
    bridge_id: &str,
) -> Value {
    let response = bridge_release_response(
        harness,
        nonce,
        escrow_id,
        bridge_id,
        format!("bridge-release-{nonce}").as_str(),
    );
    assert!(response.contains("HTTP/1.1 200 OK"), "{response}");
    parse_service_api_payload(extract_http_response_body(response.as_str()))
        .expect("bridge-authorized release should deserialize")
}

pub(super) fn bridge_release_response(
    harness: &AssetMovementHarness,
    nonce: u64,
    escrow_id: &str,
    bridge_id: &str,
    idempotency_key: &str,
) -> String {
    let body = json!({
        "idempotency_key": idempotency_key,
        "authority_mode": "bridge-receipt",
        "bridge_id": bridge_id,
    })
    .to_string();
    authorized_signed_request(
        &harness.snapshot,
        harness.bind_addr.as_str(),
        SignedRequest {
            max_requests: 1,
            method: "POST",
            path: format!("/v1/escrow/{escrow_id}/release").as_str(),
            caller_did: harness.caller_did,
            nonce,
            body: body.as_str(),
            extra_headers: &[],
        },
    )
}

pub(super) fn restart_release_with_bridge_authority(
    api_bind: &str,
    nonce: u64,
    escrow_id: &str,
    bridge_id: &str,
    idempotency_key: &str,
) -> Value {
    let snapshot = build_task_escrow_snapshot(api_bind);
    let harness = AssetMovementHarnessRef {
        snapshot: &snapshot,
        bind_addr: reserve_loopback_addr(),
    };
    let response = bridge_release_for_snapshot(
        harness.snapshot,
        harness.bind_addr.as_str(),
        nonce,
        escrow_id,
        bridge_id,
        idempotency_key,
    );
    assert!(response.contains("HTTP/1.1 200 OK"), "{response}");
    parse_service_api_payload(extract_http_response_body(response.as_str()))
        .expect("restart bridge release should deserialize")
}

fn bridge_release_for_snapshot(
    snapshot: &crate::service_api_endpoint::ServiceApiSnapshot,
    bind_addr: &str,
    nonce: u64,
    escrow_id: &str,
    bridge_id: &str,
    idempotency_key: &str,
) -> String {
    let body = json!({
        "idempotency_key": idempotency_key,
        "authority_mode": "bridge-receipt",
        "bridge_id": bridge_id,
    })
    .to_string();
    authorized_signed_request(
        snapshot,
        bind_addr,
        SignedRequest {
            max_requests: 1,
            method: "POST",
            path: format!("/v1/escrow/{escrow_id}/release").as_str(),
            caller_did: ACTOR,
            nonce,
            body: body.as_str(),
            extra_headers: &[],
        },
    )
}

struct AssetMovementHarnessRef<'a> {
    snapshot: &'a crate::service_api_endpoint::ServiceApiSnapshot,
    bind_addr: String,
}

pub(super) fn query_projection(
    harness: &AssetMovementHarness,
    caller_did: &str,
    nonce: u64,
    task_id: &str,
    view: &str,
) -> Value {
    let response = raw_signed_request(
        &harness.snapshot,
        harness.bind_addr.as_str(),
        SignedRequest {
            max_requests: 1,
            method: "GET",
            path: format!("/v1/tasks/{task_id}/{view}").as_str(),
            caller_did,
            nonce,
            body: "",
            extra_headers: &[("X-KAMN-Authz-Scope", "tasks:read")],
        },
    );
    assert!(response.contains("HTTP/1.1 200 OK"), "{response}");
    parse_service_api_payload(extract_http_response_body(response.as_str()))
        .expect("projection should deserialize")
}
