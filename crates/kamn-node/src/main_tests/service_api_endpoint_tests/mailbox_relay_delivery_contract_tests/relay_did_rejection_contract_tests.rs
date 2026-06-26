use super::super::*;
use super::state_support::read_spool_lines;
use super::support::{
    build_mailbox_relay_snapshot, read_state_json, relay_message, send_message,
    set_relay_spool_env, set_state_file_env, spawn_api_server, unique_named_relay_spool_file,
    unique_named_state_file,
};
use crate::service_api_endpoint::ServiceApiSnapshot;
use std::path::Path;

#[test]
fn integration_service_api_endpoint_rejects_legacy_message_send_recipient_dids() {
    let _env = acquire_service_api_test_env();
    let state_file = unique_named_state_file("kamn-node-service-api-legacy-recipient-state");
    let spool_file = unique_named_relay_spool_file("kamn-node-service-api-legacy-recipient-spool");
    let _state_guard = set_state_file_env(state_file.as_path());
    let _spool_guard = set_relay_spool_env(spool_file.as_path());
    let snapshot = build_mailbox_relay_snapshot("127.0.0.1:34122");
    let bind_addr = reserve_loopback_addr();
    let server = spawn_api_server(&snapshot, bind_addr.as_str(), 2);
    let sender_did = "kamn:did:agent:legacy-recipient-sender";
    let canonical_recipient_did = "kamn:did:agent:legacy-recipient-target";
    assert_legacy_message_send_rejection(&snapshot, bind_addr.as_str(), sender_did);
    let payload = assert_canonical_message_send(
        &snapshot,
        bind_addr.as_str(),
        sender_did,
        canonical_recipient_did,
    );
    super::support::assert_server_ok(
        server,
        "service api endpoint should stop cleanly after legacy recipient rejection flow",
    );
    assert_legacy_message_send_persistence(
        state_file.as_path(),
        spool_file.as_path(),
        canonical_recipient_did,
        &payload.message_id,
        sender_did,
    );
    let _ = fs::remove_file(state_file);
    let _ = fs::remove_file(spool_file);
}

#[test]
fn integration_service_api_endpoint_rejects_legacy_relay_ingest_dids() {
    let _env = acquire_service_api_test_env();
    let state_file = unique_named_state_file("kamn-node-service-api-legacy-relay-state");
    let _state_guard = set_state_file_env(state_file.as_path());
    let snapshot = build_mailbox_relay_snapshot("127.0.0.1:34123");
    let bind_addr = reserve_loopback_addr();
    let server = spawn_api_server(&snapshot, bind_addr.as_str(), 3);
    let caller_did = "kamn:did:agent:relay-ingest-caller";
    assert_legacy_relay_rejection(
        &snapshot,
        bind_addr.as_str(),
        caller_did,
        51,
        legacy_recipient_body(),
    );
    assert_legacy_relay_rejection(
        &snapshot,
        bind_addr.as_str(),
        caller_did,
        52,
        legacy_sender_body(),
    );
    let response = relay_message(
        &snapshot,
        bind_addr.as_str(),
        caller_did,
        53,
        canonical_relay_body(),
    );
    assert!(response.contains("HTTP/1.1 202 Accepted"));
    super::support::assert_server_ok(
        server,
        "service api endpoint should stop cleanly after relay did rejection flow",
    );
    assert_legacy_relay_persistence(state_file.as_path());
    let _ = fs::remove_file(state_file);
}

fn assert_legacy_message_send_rejection(
    snapshot: &ServiceApiSnapshot,
    bind_addr: &str,
    sender_did: &str,
) {
    let response = super::support::send_signed_request(
        snapshot,
        bind_addr,
        "POST",
        "/v1/messages/send",
        sender_did,
        41,
        r#"{"recipient_did":"did:kamn:agent:legacy-alpha","message":"reject-me"}"#,
    );
    let payload = parse_error_envelope_from_http_response(response.as_str());
    assert!(response.contains("HTTP/1.1 400 Bad Request"));
    assert_eq!(
        payload.reason_code,
        SERVICE_API_MESSAGE_RECIPIENT_DID_INVALID_REASON_CODE
    );
    assert!(payload.message.contains("invalid recipient did"));
}

fn assert_canonical_message_send(
    snapshot: &ServiceApiSnapshot,
    bind_addr: &str,
    sender_did: &str,
    recipient_did: &str,
) -> ServiceApiMessageCreateBody {
    let body = format!(r#"{{"recipient_did":"{recipient_did}","message":"accept-me"}}"#);
    send_message(snapshot, bind_addr, sender_did, 42, body.as_str())
}

fn assert_legacy_message_send_persistence(
    state_file: &Path,
    spool_file: &Path,
    recipient_did: &str,
    message_id: &str,
    sender_did: &str,
) {
    let state_json = read_state_json(state_file);
    let messages = state_json["messages"]
        .as_object()
        .expect("messages snapshot should be an object");
    let persisted = messages
        .values()
        .next()
        .expect("canonical send should persist exactly one message");
    assert_eq!(messages.len(), 1);
    assert_eq!(persisted["message_id"], message_id);
    assert_eq!(persisted["recipient_did"], recipient_did);
    assert_eq!(
        persisted["sender_did"],
        test_service_api_sender_did(sender_did)
    );
    let spool_lines = read_spool_lines(spool_file);
    let spool_entry: Value = serde_json::from_str(spool_lines[0].as_str())
        .expect("relay spool entry should deserialize");
    assert_eq!(spool_lines.len(), 1);
    assert_eq!(spool_entry["message_id"], message_id);
    assert_eq!(spool_entry["recipient_did"], recipient_did);
}

fn assert_legacy_relay_rejection(
    snapshot: &ServiceApiSnapshot,
    bind_addr: &str,
    caller_did: &str,
    nonce: u64,
    body: &str,
) {
    let response = relay_message(snapshot, bind_addr, caller_did, nonce, body);
    let payload = parse_error_envelope_from_http_response(response.as_str());
    assert!(response.contains("HTTP/1.1 400 Bad Request"));
    assert_eq!(
        payload.reason_code,
        SERVICE_API_RELAY_DID_INVALID_REASON_CODE
    );
}

fn assert_legacy_relay_persistence(state_file: &Path) {
    let messages = read_state_json(state_file)["messages"]
        .as_object()
        .expect("messages snapshot should be an object")
        .clone();
    assert_eq!(messages.len(), 1);
    assert!(messages.contains_key("msg-relay-canonical"));
    assert!(!messages.contains_key("msg-relay-legacy-recipient"));
    assert!(!messages.contains_key("msg-relay-legacy-sender"));
}

fn legacy_recipient_body() -> &'static str {
    r#"{
      "message_id":"msg-relay-legacy-recipient",
      "sender_did":"kamn:did:agent:relay-sender",
      "recipient_did":"did:kamn:agent:legacy-recipient",
      "body":"{\"message\":\"relay-recipient\"}"
    }"#
}

fn legacy_sender_body() -> &'static str {
    r#"{
      "message_id":"msg-relay-legacy-sender",
      "sender_did":"did:kamn:agent:legacy-sender",
      "recipient_did":"kamn:did:agent:relay-recipient",
      "body":"{\"message\":\"relay-sender\"}"
    }"#
}

fn canonical_relay_body() -> &'static str {
    r#"{
      "message_id":"msg-relay-canonical",
      "sender_did":"kamn:did:agent:relay-sender",
      "recipient_did":"kamn:did:agent:relay-recipient",
      "body":"{\"message\":\"relay-canonical\"}"
    }"#
}
