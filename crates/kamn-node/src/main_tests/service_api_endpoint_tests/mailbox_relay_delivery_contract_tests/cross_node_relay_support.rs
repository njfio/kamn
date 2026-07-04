use super::super::*;
use super::support::{
    build_mailbox_relay_snapshot, query_message, read_state_json, send_message,
    set_relay_spool_env, set_state_file_env, spawn_api_server, unique_named_relay_spool_file,
    unique_named_state_file,
};
use crate::service_api_endpoint::ServiceApiSnapshot;
use std::path::PathBuf;

pub(super) struct CrossNodeFiles {
    sender_state_file: PathBuf,
    sender_spool_file: PathBuf,
    recipient_state_file: PathBuf,
    recipient_spool_file: PathBuf,
}

pub(super) struct ServerContext {
    pub bind_addr: String,
    pub snapshot: ServiceApiSnapshot,
    pub server: thread::JoinHandle<Result<(), String>>,
    _state_guard: EnvVarGuard,
    _spool_guard: EnvVarGuard,
}

impl CrossNodeFiles {
    pub(super) fn new() -> Self {
        Self {
            sender_state_file: unique_named_state_file("kamn-cross-node-sender-state"),
            sender_spool_file: unique_named_relay_spool_file("kamn-cross-node-sender-spool"),
            recipient_state_file: unique_named_state_file("kamn-cross-node-recipient-state"),
            recipient_spool_file: unique_named_relay_spool_file("kamn-cross-node-recipient-spool"),
        }
    }
}

pub(super) fn boot_sender(files: &CrossNodeFiles) -> ServerContext {
    boot_cross_node_server(
        files.sender_state_file.as_path(),
        files.sender_spool_file.as_path(),
        "127.0.0.1:34115",
        1,
    )
}

pub(super) fn boot_recipient(files: &CrossNodeFiles) -> ServerContext {
    boot_cross_node_server(
        files.recipient_state_file.as_path(),
        files.recipient_spool_file.as_path(),
        "127.0.0.1:34116",
        3,
    )
}

fn boot_cross_node_server(
    state_file: &std::path::Path,
    spool_file: &std::path::Path,
    snapshot_addr: &str,
    max_requests: u64,
) -> ServerContext {
    let state_guard = set_state_file_env(state_file);
    let spool_guard = set_relay_spool_env(spool_file);
    let snapshot = build_mailbox_relay_snapshot(snapshot_addr);
    let bind_addr = reserve_loopback_addr();
    let server = spawn_api_server(&snapshot, bind_addr.as_str(), max_requests);
    ServerContext {
        bind_addr,
        snapshot,
        server,
        _state_guard: state_guard,
        _spool_guard: spool_guard,
    }
}

pub(super) fn assert_cross_node_send_phase(
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

pub(super) fn project_relay_to_recipient(
    files: &CrossNodeFiles,
    recipient_addr: &str,
    recipient_did: &str,
) {
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

pub(super) fn assert_cross_node_recipient_phase(
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

pub(super) fn assert_cross_node_persistence(files: &CrossNodeFiles, message_id: &str) {
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

pub(super) fn cleanup_cross_node_files(files: CrossNodeFiles) {
    let _ = fs::remove_file(files.sender_state_file);
    let _ = fs::remove_file(files.sender_spool_file);
    let _ = fs::remove_file(files.recipient_state_file);
    let _ = fs::remove_file(files.recipient_spool_file);
}

fn run_daemon_once() -> NodeBootstrapReport {
    let args: Vec<String> = [
        "kamn-node",
        "--role",
        "processor",
        "--runtime-mode",
        "daemon",
        "--daemon-max-ticks",
        "1",
        "--daemon-tick-interval-ms",
        "1",
        "--daemon-shutdown-signal-tick",
        "1",
        "--daemon-shutdown-drain-ticks",
        "1",
        "--daemon-shutdown-timeout-ticks",
        "1",
    ]
    .into_iter()
    .map(str::to_owned)
    .collect();
    execute(parse_args(args).expect("daemon args should parse"))
        .expect("daemon relay projection should succeed")
}
