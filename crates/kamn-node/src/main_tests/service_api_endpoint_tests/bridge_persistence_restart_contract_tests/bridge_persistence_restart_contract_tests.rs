use super::super::*;
use super::support::{
    build_bridge_snapshot, forward_bridge, query_bridge, query_missing_bridge, read_state_json,
    set_state_file_env, submit_bridge, unique_named_state_file,
};

#[test]
fn integration_service_api_endpoint_persists_bridge_state_across_restart() {
    let _env = acquire_service_api_test_env();
    let state_file = unique_named_state_file("kamn-node-service-api-bridge-restart-state");
    let _state_file_guard = set_state_file_env(state_file.as_path());
    let caller_did = "kamn:did:agent:test-client-bridge-restart";
    let bridge_id = assert_bridge_submit_phase(caller_did);
    assert_bridge_restart_phase(caller_did, bridge_id, state_file.as_path());
    let _ = fs::remove_file(state_file);
}

fn assert_bridge_submit_phase(caller_did: &str) -> String {
    let first_snapshot = build_bridge_snapshot("127.0.0.1:34115");
    let bind_addr = reserve_loopback_addr();
    let submitted = submit_bridge(
        &first_snapshot,
        bind_addr.as_str(),
        caller_did,
        101,
        r#"{"source_message_id":"msg-bridge-restart-source"}"#,
    );
    let bridge_id = submitted["bridge_id"].as_str().expect("bridge id should be string");
    assert_eq!(submitted["bridge_status"], "submitted");
    bridge_id.to_owned()
}

fn assert_bridge_restart_phase(caller_did: &str, bridge_id: String, state_file: &std::path::Path) {
    let restart_snapshot = build_bridge_snapshot("127.0.0.1:34116");
    let restart_bind_addr = reserve_loopback_addr();
    let forwarded = forward_bridge(&restart_snapshot, restart_bind_addr.as_str(), caller_did, 102, bridge_id.as_str());
    let queried = query_bridge(&restart_snapshot, restart_bind_addr.as_str(), caller_did, 103, bridge_id.as_str());
    let missing = query_missing_bridge(
        &restart_snapshot,
        restart_bind_addr.as_str(),
        "kamn:did:agent:test-client-bridge-missing",
        104,
        "bridge-missing-104",
    );
    let state_json = read_state_json(state_file);
    assert_bridge_restart_payloads(&forwarded, &queried, &missing, &state_json, bridge_id.as_str());
}

fn assert_bridge_restart_payloads(
    forwarded: &Value,
    queried: &Value,
    missing: &ServiceApiErrorEnvelope,
    state_json: &Value,
    bridge_id: &str,
) {
    assert_eq!(forwarded["bridge_id"], bridge_id);
    assert_eq!(forwarded["bridge_status"], "forwarded");
    assert_eq!(forwarded["target_message_id"], format!("msg-bridge-target-{bridge_id}"));
    assert_eq!(queried["bridge_id"], bridge_id);
    assert_eq!(queried["bridge_status"], "forwarded");
    assert_eq!(queried["target_message_id"], format!("msg-bridge-target-{bridge_id}"));
    assert_eq!(missing.error, "not-found");
    assert_eq!(missing.reason_code, "service_api_route_not_found");
    assert_eq!(state_json["bridges"][bridge_id]["bridge_status"], "forwarded");
    assert_eq!(state_json["bridges"][bridge_id]["target_message_id"], format!("msg-bridge-target-{bridge_id}"));
    assert_eq!(state_json["bridges"][bridge_id]["forward_tx_hash"], format!("sha256:bridge-forwarded-{bridge_id}"));
}
