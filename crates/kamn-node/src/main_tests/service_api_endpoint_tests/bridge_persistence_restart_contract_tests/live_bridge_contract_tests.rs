use super::super::*;
use super::support::{
    build_bridge_snapshot, forward_bridge, query_bridge,
    set_default_live_solana_bridge_rpc_url_env, set_state_file_env, submit_bridge,
    unique_named_state_file,
};

#[test]
fn integration_service_api_endpoint_live_bridge_forward_path_rejects_placeholder_evidence() {
    let _env = acquire_service_api_test_env();
    let _live_rpc_guard = set_default_live_solana_bridge_rpc_url_env();
    let state_file = unique_named_state_file("kamn-node-service-api-bridge-live-evidence");
    let _state_file_guard = set_state_file_env(state_file.as_path());
    let caller_did = "kamn:did:agent:test-client-bridge-live-evidence";
    let (bridge_id, forwarded) = submit_and_forward_live_bridge(
        caller_did,
        "127.0.0.1:34117",
        201,
        202,
        r#"{"source_message_id":"msg-bridge-live-source"}"#,
    );
    assert_non_placeholder_bridge_evidence(bridge_id.as_str(), &forwarded);
    let _ = fs::remove_file(state_file);
}

#[test]
fn integration_service_api_endpoint_live_bridge_forward_evidence_survives_restart() {
    let _env = acquire_service_api_test_env();
    let _live_rpc_guard = set_default_live_solana_bridge_rpc_url_env();
    let state_file = unique_named_state_file("kamn-node-service-api-bridge-live-restart");
    let _state_file_guard = set_state_file_env(state_file.as_path());
    let caller_did = "kamn:did:agent:test-client-bridge-live-restart";
    let (bridge_id, queried) = submit_and_restart_live_bridge(
        caller_did,
        "127.0.0.1:34119",
        "127.0.0.1:34120",
        301,
        302,
        303,
        r#"{"source_message_id":"msg-bridge-live-restart-source"}"#,
    );
    assert_non_placeholder_bridge_payload(bridge_id.as_str(), &queried);
    let _ = fs::remove_file(state_file);
}

fn assert_non_placeholder_bridge_evidence(bridge_id: &str, forwarded: &Value) {
    let target_message_id = forwarded["target_message_id"]
        .as_str()
        .expect("target message id should be string");
    let forward_tx_hash = forwarded["forward_tx_hash"]
        .as_str()
        .expect("forward tx hash should be string");
    assert_ne!(
        target_message_id,
        format!("msg-bridge-target-{bridge_id}"),
        "live bridge path must not persist placeholder target ids"
    );
    assert_ne!(
        forward_tx_hash,
        format!("sha256:bridge-forwarded-{bridge_id}"),
        "live bridge path must not persist placeholder forward hashes"
    );
}

fn assert_non_placeholder_bridge_payload(bridge_id: &str, queried: &Value) {
    assert_ne!(
        queried["target_message_id"],
        Value::String(format!("msg-bridge-target-{bridge_id}")),
        "restart-visible live bridge target ids must not fall back to placeholders"
    );
    assert_ne!(
        queried["forward_tx_hash"],
        Value::String(format!("sha256:bridge-forwarded-{bridge_id}")),
        "restart-visible live bridge forward hashes must not fall back to placeholders"
    );
}

fn submit_and_forward_live_bridge(
    caller_did: &str,
    snapshot_addr: &str,
    submit_request_id: u64,
    forward_request_id: u64,
    request_body: &str,
) -> (String, Value) {
    let snapshot = build_bridge_snapshot(snapshot_addr);
    let bind_addr = reserve_loopback_addr();
    let submitted = submit_bridge(
        &snapshot,
        bind_addr.as_str(),
        caller_did,
        submit_request_id,
        request_body,
    );
    let bridge_id = submitted["bridge_id"]
        .as_str()
        .expect("bridge id should be string")
        .to_owned();
    let forwarded = forward_bridge(
        &snapshot,
        bind_addr.as_str(),
        caller_did,
        forward_request_id,
        bridge_id.as_str(),
    );
    (bridge_id, forwarded)
}

fn submit_and_restart_live_bridge(
    caller_did: &str,
    first_snapshot_addr: &str,
    restart_snapshot_addr: &str,
    submit_request_id: u64,
    forward_request_id: u64,
    restart_request_id: u64,
    request_body: &str,
) -> (String, Value) {
    let (bridge_id, _) = submit_and_forward_live_bridge(
        caller_did,
        first_snapshot_addr,
        submit_request_id,
        forward_request_id,
        request_body,
    );
    let queried = query_restarted_live_bridge(
        restart_snapshot_addr,
        caller_did,
        restart_request_id,
        bridge_id.as_str(),
    );
    (bridge_id, queried)
}

fn query_restarted_live_bridge(
    snapshot_addr: &str,
    caller_did: &str,
    request_id: u64,
    bridge_id: &str,
) -> Value {
    let restart_snapshot = build_bridge_snapshot(snapshot_addr);
    let restart_bind_addr = reserve_loopback_addr();
    query_bridge(
        &restart_snapshot,
        restart_bind_addr.as_str(),
        caller_did,
        request_id,
        bridge_id,
    )
}
