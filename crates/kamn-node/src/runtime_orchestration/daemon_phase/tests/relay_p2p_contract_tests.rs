use super::super::service_api_relay_p2p::{
    drain_daemon_service_api_relay_p2p_inbox_for_test,
    forward_service_api_relay_entry_via_p2p_for_test,
    resolve_daemon_service_api_relay_p2p_in_memory_context_from_json_for_test,
    set_daemon_service_api_relay_p2p_config_override_for_test,
    SERVICE_API_RELAY_P2P_DEFAULT_TOPIC_FOR_TEST,
};
use super::super::service_api_relay_tick_loop::{
    execute_daemon_service_api_relay_tick_loop, SERVICE_API_RELAY_RECIPIENT_ROUTE_MAP_ENV_FOR_TEST,
};
use super::support::{
    lock_daemon_phase_test_guard, relay_config_json, relay_fixture_paths, remove_relay_fixture,
    unique_p2p_listen_address, write_empty_state_fixture, write_relay_fixture, TestEnvGuard,
};
use std::fs;
use std::sync::Arc;

#[test]
fn unit_daemon_relay_p2p_config_omitted_topic_defaults_to_messages() {
    let _test_lock = lock_daemon_phase_test_guard();
    let _log_lock = crate::logging::lock_log_config_for_tests();
    let listen_address = unique_p2p_listen_address();
    let relay_config_json = relay_config_json(
        "daemon-p2p-default-topic",
        listen_address.as_str(),
        &[listen_address.as_str()],
        None,
        &[("kamn:did:agent:recipient", "daemon-p2p-default-topic")],
    );
    let shared_transport = Arc::new(kamn_core::InMemoryPeerLifecycleTransport::default());
    let relay_context = resolve_daemon_service_api_relay_p2p_in_memory_context_from_json_for_test(
        relay_config_json.as_str(),
        shared_transport,
    )
    .expect("relay p2p context should parse with default topic");
    assert_eq!(
        relay_context.topic.as_str(),
        SERVICE_API_RELAY_P2P_DEFAULT_TOPIC_FOR_TEST
    );
}

#[test]
fn integration_daemon_relay_p2p_forward_and_ingest_updates_recipient_state() {
    let _test_lock = lock_daemon_phase_test_guard();
    let _log_lock = crate::logging::lock_log_config_for_tests();
    let recipient_paths = relay_fixture_paths("kamn-node-daemon-phase-p2p-recipient");
    write_empty_state_fixture(recipient_paths.state_file.as_path());
    let ingested = forward_and_ingest_in_memory_relay(
        &recipient_paths,
        relay_entry_fixture(
            "msg-p2p-forward-unit-1",
            r#"{"message":"hello-p2p"}"#,
            1_700_001_000,
        ),
    );
    assert_eq!(ingested, 1);
    assert_message_relayed(&recipient_paths, "msg-p2p-forward-unit-1");
    remove_relay_fixture(&recipient_paths);
}

#[test]
fn regression_daemon_relay_tick_loop_p2p_unknown_recipient_requeues_with_error_counter() {
    let _test_lock = lock_daemon_phase_test_guard();
    let _log_lock = crate::logging::lock_log_config_for_tests();
    let paths = relay_fixture_paths("kamn-node-daemon-phase-p2p-unknown-recipient");
    seed_unknown_recipient_fixture(&paths);
    let _route_guard = TestEnvGuard::set(SERVICE_API_RELAY_RECIPIENT_ROUTE_MAP_ENV_FOR_TEST, None);
    let _p2p_guard = set_daemon_service_api_relay_p2p_config_override_for_test(Some(
        unknown_recipient_config_json().as_str(),
    ));
    let runtime_processing = run_unknown_recipient_tick_loop(&paths);
    assert_eq!(runtime_processing.relay_drained_count, 1);
    assert_eq!(runtime_processing.relay_projected_state_count, 0);
    assert_eq!(runtime_processing.processing_error_count, 1);
    assert_spool_contains_unknown_recipient(&paths);
    remove_relay_fixture(&paths);
}

fn forward_and_ingest_in_memory_relay(
    recipient_paths: &super::support::RelayFixturePaths,
    relay_entry: crate::service_api_endpoint::ServiceApiRelaySpoolEntry,
) -> usize {
    let (sender_relay_context, recipient_relay_context) = in_memory_p2p_context_pair();
    forward_service_api_relay_entry_via_p2p_for_test(&sender_relay_context, &relay_entry)
        .expect("p2p relay send should succeed with deterministic in-memory transport");
    drain_daemon_service_api_relay_p2p_inbox_for_test(
        &recipient_relay_context,
        Some(recipient_paths.state_file.to_string_lossy().as_ref()),
    )
    .expect("recipient inbox drain should succeed")
}

fn assert_message_relayed(paths: &super::support::RelayFixturePaths, message_id: &str) {
    let recipient_state_payload = fs::read_to_string(paths.state_file.as_path())
        .expect("recipient state file should remain readable");
    let recipient_state_json: serde_json::Value =
        serde_json::from_str(recipient_state_payload.as_str())
            .expect("recipient state payload should parse");
    assert_eq!(
        recipient_state_json["messages"][message_id]["status"],
        "relayed"
    );
}

fn seed_unknown_recipient_fixture(paths: &super::support::RelayFixturePaths) {
    write_relay_fixture(
        paths,
        "msg-p2p-unknown-recipient-unit-1",
        r#"{"message":"p2p-unknown-recipient"}"#,
        1_700_001_100,
    );
}

fn run_unknown_recipient_tick_loop(
    paths: &super::support::RelayFixturePaths,
) -> crate::daemon_observability::DaemonRuntimeProcessingTelemetry {
    execute_daemon_service_api_relay_tick_loop(
        1,
        1,
        Some(paths.state_file.to_string_lossy().as_ref()),
        Some(paths.relay_spool_file.to_string_lossy().as_ref()),
        "service-api:kamn-devnet:v0.1.0",
    )
    .expect("daemon relay tick loop should complete")
}

fn assert_spool_contains_unknown_recipient(paths: &super::support::RelayFixturePaths) {
    let relay_payload = fs::read_to_string(paths.relay_spool_file.as_path())
        .expect("relay spool file should remain readable");
    assert!(relay_payload.contains("msg-p2p-unknown-recipient-unit-1"));
}

fn in_memory_p2p_context_pair() -> (
    super::super::service_api_relay_p2p::DaemonServiceApiRelayP2pContext,
    super::super::service_api_relay_p2p::DaemonServiceApiRelayP2pContext,
) {
    let sender_listen_address = unique_p2p_listen_address();
    let recipient_listen_address = unique_p2p_listen_address();
    let shared_transport = Arc::new(kamn_core::InMemoryPeerLifecycleTransport::default());
    let sender = resolve_daemon_service_api_relay_p2p_in_memory_context_from_json_for_test(
        sender_config_json(&sender_listen_address, &recipient_listen_address).as_str(),
        Arc::clone(&shared_transport),
    )
    .expect("sender relay p2p context should parse");
    let recipient = resolve_daemon_service_api_relay_p2p_in_memory_context_from_json_for_test(
        recipient_config_json(&sender_listen_address, &recipient_listen_address).as_str(),
        Arc::clone(&shared_transport),
    )
    .expect("recipient relay p2p context should parse");
    (sender, recipient)
}

fn sender_config_json(sender_listen_address: &str, recipient_listen_address: &str) -> String {
    relay_config_json(
        "daemon-p2p-sender",
        sender_listen_address,
        &[sender_listen_address, recipient_listen_address],
        Some("messages"),
        &[("kamn:did:agent:recipient", "daemon-p2p-recipient")],
    )
}

fn recipient_config_json(sender_listen_address: &str, recipient_listen_address: &str) -> String {
    relay_config_json(
        "daemon-p2p-recipient",
        recipient_listen_address,
        &[sender_listen_address, recipient_listen_address],
        Some("messages"),
        &[],
    )
}

fn relay_entry_fixture(
    message_id: &str,
    body: &str,
    queued_at_unix: u64,
) -> crate::service_api_endpoint::ServiceApiRelaySpoolEntry {
    crate::service_api_endpoint::ServiceApiRelaySpoolEntry {
        message_id: message_id.to_owned(),
        sender_did: Some("kamn:did:agent:sender".to_owned()),
        recipient_did: "kamn:did:agent:recipient".to_owned(),
        body: body.to_owned(),
        queued_at_unix,
    }
}

fn unknown_recipient_config_json() -> String {
    let listen_address = unique_p2p_listen_address();
    relay_config_json(
        "daemon-p2p-unknown-recipient-sender",
        listen_address.as_str(),
        &[listen_address.as_str()],
        Some("messages"),
        &[(
            "kamn:did:agent:recipient",
            "daemon-p2p-unknown-recipient-recipient",
        )],
    )
}
