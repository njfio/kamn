use super::super::*;
use super::support::{
    build_directory_snapshot, create_channel, list_channel_messages, send_channel_message,
    unique_named_state_file,
};

#[test]
fn integration_service_api_endpoint_lists_channel_messages_from_message_store() {
    let _env = acquire_service_api_test_env();
    let state_file = unique_named_state_file("kamn-node-service-api-channel-state");
    let state_file_text = state_file.to_string_lossy().to_string();
    let _state_file_guard = EnvVarGuard::set("KAMN_SERVICE_API_STATE_FILE", Some(state_file_text.as_str()));
    let snapshot = build_directory_snapshot("127.0.0.1:34081");
    let sender_did = "kamn:did:agent:test-client-channel";
    let bind_addr = reserve_loopback_addr();

    let sent = send_channel_message(
        &snapshot,
        bind_addr.as_str(),
        sender_did,
        11,
        r#"{"channel_id":"channel-contract-42","message":"hello"}"#,
    );
    let listed = list_channel_messages(&snapshot, bind_addr.as_str(), sender_did, 12, "channel-contract-42");

    assert_eq!(listed.channel_id, "channel-contract-42");
    assert!(listed.messages.contains(&sent.message_id));
    let _ = fs::remove_file(state_file);
}

#[test]
fn integration_service_api_endpoint_persists_channel_creation_state_across_restart() {
    let _env = acquire_service_api_test_env();
    let state_file = unique_named_state_file("kamn-node-service-api-channel-create-restart-state");
    let state_file_text = state_file.to_string_lossy().to_string();
    let _state_file_guard = EnvVarGuard::set("KAMN_SERVICE_API_STATE_FILE", Some(state_file_text.as_str()));
    let snapshot = build_directory_snapshot("127.0.0.1:34117");
    let caller_did = "kamn:did:agent:test-client-channel-create-restart";
    let created = create_channel(&snapshot, reserve_loopback_addr().as_str(), caller_did, 111, r#"{"name":"channel-restart-contract"}"#);

    let phase_one_state_json = super::support::read_state_json(state_file.as_path());
    let persisted_channel = phase_one_state_json["channel_messages"]
        .get(created.channel_id.as_str())
        .and_then(serde_json::Value::as_array);
    assert_eq!(created.status, "created");
    assert!(persisted_channel.is_some());
    assert_eq!(persisted_channel.map(std::vec::Vec::len), Some(0));

    let restart_snapshot = build_directory_snapshot("127.0.0.1:34118");
    let listed = list_channel_messages(
        &restart_snapshot,
        reserve_loopback_addr().as_str(),
        caller_did,
        112,
        created.channel_id.as_str(),
    );
    assert_eq!(listed.channel_id, created.channel_id);
    assert!(listed.messages.is_empty());
    let _ = fs::remove_file(state_file);
}
