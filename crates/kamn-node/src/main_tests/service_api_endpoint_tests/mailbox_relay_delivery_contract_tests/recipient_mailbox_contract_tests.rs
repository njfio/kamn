use super::super::*;
use super::state_support::read_first_spool_entry;
use super::support::{
    assert_server_ok, build_mailbox_relay_snapshot, read_state_json, send_message,
    set_relay_spool_env, set_state_file_env, spawn_api_server, unique_named_relay_spool_file,
    unique_named_state_file,
};
use std::io::{ErrorKind, Read, Write};
use std::net::TcpListener;
use std::path::{Path, PathBuf};

struct DeliveryFiles {
    state_file: PathBuf,
    relay_spool_file: PathBuf,
}

struct DeliveryContext {
    body: String,
    effective_sender_did: String,
    message_id: String,
    recipient_did: String,
}

#[test]
fn integration_service_api_endpoint_recipient_mailbox_and_delivery_status_contract() {
    let _env = acquire_service_api_test_env();
    let files = DeliveryFiles::new();
    let _state_guard = set_state_file_env(files.state_file.as_path());
    let _spool_guard = set_relay_spool_env(files.relay_spool_file.as_path());
    let context = assert_send_phase();
    assert_initial_spool_entry(files.relay_spool_file.as_path(), &context);
    assert_daemon_projection_phase(
        files.state_file.as_path(),
        files.relay_spool_file.as_path(),
        &context,
    );
    assert_restart_delivery_phase(files.state_file.as_path(), &context);
    let _ = fs::remove_file(files.state_file);
    let _ = fs::remove_file(files.relay_spool_file);
}

impl DeliveryFiles {
    fn new() -> Self {
        Self {
            state_file: unique_named_state_file("kamn-node-service-api-recipient-delivery-state"),
            relay_spool_file: unique_named_relay_spool_file(
                "kamn-node-service-api-recipient-delivery-spool",
            ),
        }
    }
}

fn assert_send_phase() -> DeliveryContext {
    let snapshot = build_mailbox_relay_snapshot("127.0.0.1:34107");
    let sender_did = "kamn:did:agent:delivery-sender";
    let recipient_did = test_service_api_sender_did("kamn:did:agent:delivery-recipient");
    let body = format!(r#"{{"recipient_did":"{recipient_did}","message":"deliver-me"}}"#);
    let bind_addr = reserve_loopback_addr();
    let server = spawn_api_server(&snapshot, bind_addr.as_str(), 2);
    let payload = send_message(&snapshot, bind_addr.as_str(), sender_did, 31, body.as_str());
    let mailbox = super::support::list_mailbox(
        &snapshot,
        bind_addr.as_str(),
        recipient_did.as_str(),
        32,
        recipient_did.as_str(),
    );
    assert!(mailbox.messages.contains(&payload.message_id));
    assert_server_ok(
        server,
        "service api endpoint should stop cleanly after recipient mailbox projection flow",
    );
    DeliveryContext {
        body,
        effective_sender_did: test_service_api_sender_did(sender_did),
        message_id: payload.message_id,
        recipient_did,
    }
}

fn assert_initial_spool_entry(relay_spool_file: &Path, context: &DeliveryContext) {
    let spool_entry = read_first_spool_entry(relay_spool_file);
    assert_eq!(spool_entry["message_id"], context.message_id);
    assert_eq!(spool_entry["recipient_did"], context.recipient_did);
}

fn assert_daemon_projection_phase(
    state_file: &Path,
    relay_spool_file: &Path,
    context: &DeliveryContext,
) {
    let receiver = spawn_relay_receiver(context.recipient_did.as_str());
    let _route_guard = EnvVarGuard::set(
        "KAMN_SERVICE_API_RELAY_RECIPIENT_ROUTE_MAP_JSON",
        Some(receiver.route_map.as_str()),
    );
    let _key_guard = EnvVarGuard::set(
        "KAMN_SERVICE_API_AUTH_PRIVATE_KEY_HEX",
        Some(TEST_SERVICE_API_AUTH_PRIVATE_KEY_HEX),
    );
    let report = run_daemon_once();
    assert_eq!(report.runtime_mode, "daemon");
    assert!(report.daemon_observability_throughput_tps.unwrap_or(0) > 0);
    let request = receiver
        .handle
        .join()
        .expect("relay receiver thread should join");
    assert!(request.starts_with("POST /v1/messages/relay HTTP/1.1"));
    assert!(fs::read_to_string(relay_spool_file)
        .expect("relay spool should remain readable")
        .trim()
        .is_empty());
    assert_eq!(
        read_state_json(state_file)["messages"][context.message_id.as_str()]["status"],
        "relayed"
    );
}

fn assert_restart_delivery_phase(state_file: &Path, context: &DeliveryContext) {
    let snapshot = build_mailbox_relay_snapshot("127.0.0.1:34113");
    let bind_addr = reserve_loopback_addr();
    let server = spawn_api_server(&snapshot, bind_addr.as_str(), 1);
    let payload = super::support::query_message(
        &snapshot,
        bind_addr.as_str(),
        context.recipient_did.as_str(),
        33,
        context.message_id.as_str(),
    );
    assert_eq!(payload["message_id"], context.message_id);
    assert_eq!(payload["status"], "delivered");
    assert_eq!(payload["sender_did"], context.effective_sender_did);
    assert_eq!(payload["recipient_did"], context.recipient_did);
    assert_eq!(payload["body"], context.body);
    assert_server_ok(
        server,
        "service api endpoint should stop cleanly after recipient delivery contract flow",
    );
    assert_eq!(
        read_state_json(state_file)["messages"][context.message_id.as_str()]["status"],
        "delivered"
    );
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
        .expect("daemon args should parse for relay projection"),
    )
    .expect("daemon runtime should project relay status")
}

fn spawn_relay_receiver(recipient_did: &str) -> RelayReceiver {
    let listener = TcpListener::bind("127.0.0.1:0").expect("relay receiver listener should bind");
    let route_map = serde_json::json!({ recipient_did: listener.local_addr().expect("relay receiver addr should resolve").to_string(), }).to_string();
    let handle = thread::spawn(move || read_relay_request(listener));
    RelayReceiver { handle, route_map }
}

fn read_relay_request(listener: TcpListener) -> String {
    let (mut stream, _) = listener
        .accept()
        .expect("relay receiver should accept forwarding connection");
    stream
        .set_read_timeout(Some(Duration::from_secs(2)))
        .expect("relay receiver read timeout should configure");
    let mut request = String::new();
    let mut chunk = [0_u8; 1024];
    loop {
        match stream.read(&mut chunk) {
            Ok(0) => break,
            Ok(count) => request.push_str(
                std::str::from_utf8(&chunk[..count]).expect("relay request should be utf-8"),
            ),
            Err(error) if matches!(error.kind(), ErrorKind::WouldBlock | ErrorKind::TimedOut) => {
                break
            }
            Err(error) => panic!("relay receiver request read should succeed: {error}"),
        }
        if request.contains("\r\n\r\n") {
            break;
        }
    }
    stream
        .write_all(b"HTTP/1.1 202 Accepted\r\nContent-Length: 2\r\nConnection: close\r\n\r\n{}")
        .expect("relay receiver response should write");
    request
}

struct RelayReceiver {
    handle: thread::JoinHandle<String>,
    route_map: String,
}
