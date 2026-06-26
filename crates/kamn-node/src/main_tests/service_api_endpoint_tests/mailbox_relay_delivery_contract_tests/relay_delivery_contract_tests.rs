use super::super::*;
use super::cross_node_relay_support::{
    assert_cross_node_persistence, assert_cross_node_recipient_phase, assert_cross_node_send_phase,
    boot_recipient, boot_sender, cleanup_cross_node_files, project_relay_to_recipient,
    CrossNodeFiles,
};
use super::state_support::read_first_spool_entry;
use super::support::{
    build_mailbox_relay_snapshot, send_message, set_relay_spool_env, set_state_file_env,
    spawn_api_server, unique_named_relay_spool_file, unique_named_state_file,
};

#[test]
fn integration_service_api_endpoint_cross_node_relay_delivery_contract() {
    let _env = acquire_service_api_test_env();
    let files = CrossNodeFiles::new();
    let recipient = boot_recipient(&files);
    let sender = boot_sender(&files);
    let sender_did = "kamn:did:agent:cross-node-sender";
    let recipient_did = test_service_api_sender_did("kamn:did:agent:cross-node-recipient");
    let payload = assert_cross_node_send_phase(sender, sender_did, recipient_did.as_str());
    project_relay_to_recipient(&files, recipient.bind_addr.as_str(), recipient_did.as_str());
    assert_cross_node_recipient_phase(recipient, recipient_did.as_str(), sender_did, &payload);
    assert_cross_node_persistence(&files, payload.message_id.as_str());
    cleanup_cross_node_files(files);
}

#[test]
fn integration_service_api_endpoint_enqueues_recipient_relays_to_durable_spool() {
    let _env = acquire_service_api_test_env();
    let state_file = unique_named_state_file("kamn-node-service-api-relay-spool-state");
    let spool_file = unique_named_relay_spool_file("kamn-node-service-api-relay-spool");
    let _state_guard = set_state_file_env(state_file.as_path());
    let _spool_guard = set_relay_spool_env(spool_file.as_path());
    let snapshot = build_mailbox_relay_snapshot("127.0.0.1:34108");
    let bind_addr = reserve_loopback_addr();
    let server = spawn_api_server(&snapshot, bind_addr.as_str(), 1);
    let sender_did = "kamn:did:agent:relay-spool-sender";
    let recipient_did = test_service_api_sender_did("kamn:did:agent:relay-spool-recipient");
    let body = format!(r#"{{"recipient_did":"{recipient_did}","message":"relay-me"}}"#);
    let payload = send_message(&snapshot, bind_addr.as_str(), sender_did, 41, body.as_str());
    super::support::assert_server_ok(
        server,
        "service api endpoint should stop cleanly after relay spool enqueue request",
    );
    let spool_entry = read_first_spool_entry(spool_file.as_path());
    assert_eq!(spool_entry["message_id"], payload.message_id);
    assert_eq!(
        spool_entry["sender_did"],
        test_service_api_sender_did(sender_did)
    );
    assert_eq!(spool_entry["recipient_did"], recipient_did);
    assert_eq!(spool_entry["body"], body);
    let _ = fs::remove_file(state_file);
    let _ = fs::remove_file(spool_file);
}
