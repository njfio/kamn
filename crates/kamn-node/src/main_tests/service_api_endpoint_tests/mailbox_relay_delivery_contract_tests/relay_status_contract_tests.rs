use super::super::*;
use super::state_support::write_relayed_message_fixture;
use super::support::{
    assert_server_ok, build_mailbox_relay_snapshot, query_message, read_state_json, send_message,
    set_relay_spool_env, set_state_file_env, spawn_api_server, unique_named_relay_spool_file,
    unique_named_state_file,
};
use std::path::Path;

#[test]
fn regression_service_api_endpoint_recipient_query_requires_relayed_state_before_delivery() {
    let _env = acquire_service_api_test_env();
    let state_file = unique_named_state_file("kamn-node-service-api-delivery-gate-state");
    let spool_file = unique_named_relay_spool_file("kamn-node-service-api-delivery-gate-spool");
    let _state_guard = set_state_file_env(state_file.as_path());
    let _spool_guard = set_relay_spool_env(spool_file.as_path());
    let snapshot = build_mailbox_relay_snapshot("127.0.0.1:34114");
    let bind_addr = reserve_loopback_addr();
    let server = spawn_api_server(&snapshot, bind_addr.as_str(), 2);
    let payload = send_message(
        &snapshot,
        bind_addr.as_str(),
        "kamn:did:agent:delivery-gate-sender",
        71,
        r#"{"recipient_did":"kamn:did:agent:delivery-gate-recipient","message":"deliver-gate"}"#,
    );
    let query = query_message(
        &snapshot,
        bind_addr.as_str(),
        "kamn:did:agent:delivery-gate-recipient",
        72,
        payload.message_id.as_str(),
    );
    assert_eq!(query["message_id"], payload.message_id);
    assert_eq!(query["status"], "created");
    assert_server_ok(
        server,
        "service api endpoint should stop cleanly after delivery gate regression flow",
    );
    assert_eq!(
        read_state_json(state_file.as_path())["messages"][payload.message_id.as_str()]["status"],
        "created"
    );
    let _ = fs::remove_file(state_file);
    let _ = fs::remove_file(spool_file);
}

#[test]
fn integration_service_api_endpoint_recipient_query_promotes_relayed_to_delivered() {
    let _env = acquire_service_api_test_env();
    let state_file = unique_named_state_file("kamn-node-service-api-relayed-to-delivered-state");
    let recipient_did = test_service_api_sender_did("kamn:did:agent:recipient-relayed");
    let observer_did = test_service_api_sender_did("kamn:did:agent:recipient-relayed-observer");
    write_relayed_message_fixture(
        state_file.as_path(),
        "msg-relayed-to-delivered-1",
        "kamn:did:agent:sender-relayed",
        recipient_did.as_str(),
        &format!(r#"{{\"recipient_did\":\"{recipient_did}\",\"message\":\"relay-complete\"}}"#),
    );
    let _state_guard = set_state_file_env(state_file.as_path());
    let snapshot = build_mailbox_relay_snapshot(reserve_loopback_addr().as_str());
    let bind_addr = reserve_loopback_addr();
    let server = spawn_api_server(&snapshot, bind_addr.as_str(), 2);
    assert_eq!(
        query_message(
            &snapshot,
            bind_addr.as_str(),
            observer_did.as_str(),
            50,
            "msg-relayed-to-delivered-1"
        )["status"],
        "relayed"
    );
    assert_eq!(
        query_message(
            &snapshot,
            bind_addr.as_str(),
            recipient_did.as_str(),
            51,
            "msg-relayed-to-delivered-1"
        )["status"],
        "delivered"
    );
    assert_server_ok(
        server,
        "service api endpoint should stop cleanly after relayed recipient query flow",
    );
    let _ = fs::remove_file(state_file);
}

#[test]
fn regression_service_api_endpoint_non_recipient_query_keeps_relayed_status_across_restart() {
    let _env = acquire_service_api_test_env();
    let state_file =
        unique_named_state_file("kamn-node-service-api-relayed-non-recipient-restart-state");
    write_relayed_message_fixture(
        state_file.as_path(),
        "msg-relayed-non-recipient-restart-1",
        "kamn:did:agent:sender-relayed-restart",
        "kamn:did:agent:recipient-relayed-restart",
        r#"{\"recipient_did\":\"kamn:did:agent:recipient-relayed-restart\",\"message\":\"relay-restart\"}"#,
    );
    let _state_guard = set_state_file_env(state_file.as_path());
    assert_non_recipient_restart_phase(81, state_file.as_path());
    assert_non_recipient_restart_phase(82, state_file.as_path());
    assert_eq!(
        read_state_json(state_file.as_path())["messages"]["msg-relayed-non-recipient-restart-1"]
            ["status"],
        "relayed"
    );
    let _ = fs::remove_file(state_file);
}

fn assert_non_recipient_restart_phase(nonce: u64, state_file: &Path) {
    let snapshot = build_mailbox_relay_snapshot(reserve_loopback_addr().as_str());
    let bind_addr = reserve_loopback_addr();
    let server = spawn_api_server(&snapshot, bind_addr.as_str(), 1);
    let payload = query_message(
        &snapshot,
        bind_addr.as_str(),
        "kamn:did:agent:recipient-relayed-restart-observer",
        nonce,
        "msg-relayed-non-recipient-restart-1",
    );
    assert_eq!(payload["message_id"], "msg-relayed-non-recipient-restart-1");
    assert_eq!(payload["status"], "relayed");
    assert_server_ok(
        server,
        "service api endpoint should stop cleanly after non-recipient relay query",
    );
    assert_eq!(
        read_state_json(state_file)["messages"]["msg-relayed-non-recipient-restart-1"]["status"],
        "relayed"
    );
}
