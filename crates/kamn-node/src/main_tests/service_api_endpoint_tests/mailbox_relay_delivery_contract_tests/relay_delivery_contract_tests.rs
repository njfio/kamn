use super::super::*;
use super::state_support::read_first_spool_entry;
use super::support::{
    build_mailbox_relay_snapshot, query_message, read_state_json, send_message,
    set_relay_spool_env, set_state_file_env, spawn_api_server, unique_named_relay_spool_file,
    unique_named_state_file,
};
use crate::service_api_endpoint::ServiceApiSnapshot;
use std::path::PathBuf;

struct CrossNodeFiles {
    sender_state_file: PathBuf,
    sender_spool_file: PathBuf,
    recipient_state_file: PathBuf,
    recipient_spool_file: PathBuf,
}

struct ServerContext {
    bind_addr: String,
    snapshot: ServiceApiSnapshot,
    server: thread::JoinHandle<Result<(), String>>,
    _state_guard: EnvVarGuard,
    _spool_guard: EnvVarGuard,
}

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

impl CrossNodeFiles {
    fn new() -> Self {
        Self {
            sender_state_file: unique_named_state_file(
                "kamn-node-service-api-cross-node-sender-state",
            ),
            sender_spool_file: unique_named_relay_spool_file(
                "kamn-node-service-api-cross-node-sender-spool",
            ),
            recipient_state_file: unique_named_state_file(
                "kamn-node-service-api-cross-node-recipient-state",
            ),
            recipient_spool_file: unique_named_relay_spool_file(
                "kamn-node-service-api-cross-node-recipient-spool",
            ),
        }
    }
}

fn boot_sender(files: &CrossNodeFiles) -> ServerContext {
    let state_guard = set_state_file_env(files.sender_state_file.as_path());
    let spool_guard = set_relay_spool_env(files.sender_spool_file.as_path());
    let snapshot = build_mailbox_relay_snapshot("127.0.0.1:34115");
    let bind_addr = reserve_loopback_addr();
    let server = spawn_api_server(&snapshot, bind_addr.as_str(), 1);
    ServerContext {
        bind_addr,
        snapshot,
        server,
        _state_guard: state_guard,
        _spool_guard: spool_guard,
    }
}

fn boot_recipient(files: &CrossNodeFiles) -> ServerContext {
    let state_guard = set_state_file_env(files.recipient_state_file.as_path());
    let spool_guard = set_relay_spool_env(files.recipient_spool_file.as_path());
    let snapshot = build_mailbox_relay_snapshot("127.0.0.1:34116");
    let bind_addr = reserve_loopback_addr();
    let server = spawn_api_server(&snapshot, bind_addr.as_str(), 3);
    ServerContext {
        bind_addr,
        snapshot,
        server,
        _state_guard: state_guard,
        _spool_guard: spool_guard,
    }
}

fn assert_cross_node_send_phase(
    sender: ServerContext,
    sender_did: &str,
    recipient_did: &str,
) -> ServiceApiMessageCreateBody {
    let body = format!(r#"{{"recipient_did":"{recipient_did}","message":"cross-node"}}"#);
    let payload = send_message(
        &sender.snapshot,
        sender.bind_addr.as_str(),
        sender_did,
        81,
        body.as_str(),
    );
    super::support::assert_server_ok(
        sender.server,
        "sender endpoint should stop cleanly after send request",
    );
    payload
}

fn project_relay_to_recipient(files: &CrossNodeFiles, recipient_addr: &str, recipient_did: &str) {
    let _state_guard = set_state_file_env(files.sender_state_file.as_path());
    let _spool_guard = set_relay_spool_env(files.sender_spool_file.as_path());
    let route_map = format!(r#"{{"{recipient_did}":"{recipient_addr}"}}"#);
    let _route_guard = EnvVarGuard::set(
        "KAMN_SERVICE_API_RELAY_RECIPIENT_ROUTE_MAP_JSON",
        Some(route_map.as_str()),
    );
    let _key_guard = EnvVarGuard::set(
        "KAMN_SERVICE_API_AUTH_PRIVATE_KEY_HEX",
        Some(TEST_SERVICE_API_AUTH_PRIVATE_KEY_HEX),
    );
    let report = run_daemon_once();
    assert_eq!(report.runtime_mode, "daemon");
}

fn assert_cross_node_recipient_phase(
    recipient: ServerContext,
    recipient_did: &str,
    sender_did: &str,
    payload: &ServiceApiMessageCreateBody,
) {
    let mailbox = super::support::list_mailbox(
        &recipient.snapshot,
        recipient.bind_addr.as_str(),
        recipient_did,
        82,
        recipient_did,
    );
    let message = query_message(
        &recipient.snapshot,
        recipient.bind_addr.as_str(),
        recipient_did,
        83,
        payload.message_id.as_str(),
    );
    assert_eq!(
        mailbox
            .messages
            .iter()
            .filter(|id| *id == &payload.message_id)
            .count(),
        1
    );
    assert_eq!(message["message_id"], payload.message_id);
    assert_eq!(message["status"], "delivered");
    assert_eq!(
        message["sender_did"],
        test_service_api_sender_did(sender_did)
    );
    assert_eq!(message["recipient_did"], recipient_did);
    super::support::assert_server_ok(
        recipient.server,
        "recipient endpoint should stop cleanly after relay delivery checks",
    );
}

fn assert_cross_node_persistence(files: &CrossNodeFiles, message_id: &str) {
    let state_json = read_state_json(files.sender_state_file.as_path());
    let status = state_json["messages"][message_id]["status"]
        .as_str()
        .expect("sender state status should be a string");
    assert!(matches!(status, "relayed" | "delivered"));
    assert!(fs::read_to_string(files.sender_spool_file.as_path())
        .expect("sender relay spool file should remain readable")
        .trim()
        .is_empty());
}

fn cleanup_cross_node_files(files: CrossNodeFiles) {
    let _ = fs::remove_file(files.sender_state_file);
    let _ = fs::remove_file(files.sender_spool_file);
    let _ = fs::remove_file(files.recipient_state_file);
    let _ = fs::remove_file(files.recipient_spool_file);
}

fn run_daemon_once() -> NodeBootstrapReport {
    execute(
        parse_args(vec![
            "kamn-node".to_owned(),
            "--role".to_owned(),
            "processor".to_owned(),
            "--runtime-mode".to_owned(),
            "daemon".to_owned(),
            "--daemon-max-ticks".to_owned(),
            "1".to_owned(),
            "--daemon-tick-interval-ms".to_owned(),
            "1".to_owned(),
            "--daemon-shutdown-signal-tick".to_owned(),
            "1".to_owned(),
            "--daemon-shutdown-drain-ticks".to_owned(),
            "1".to_owned(),
            "--daemon-shutdown-timeout-ticks".to_owned(),
            "1".to_owned(),
        ])
        .expect("daemon args should parse"),
    )
    .expect("daemon relay projection should succeed")
}
