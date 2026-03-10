mod payloads;

use payloads::{
    empty_state_payload, message_state_payload, spool_entry_payload, write_json_fixture,
};
use serde_json::json;
use std::env;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU16, Ordering};
use std::sync::{Mutex, OnceLock};

static NEXT_TEST_P2P_PORT: AtomicU16 = AtomicU16::new(24_000);

pub(super) fn lock_daemon_phase_test_guard() -> std::sync::MutexGuard<'static, ()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| Mutex::new(()))
        .lock()
        .unwrap_or_else(|error| error.into_inner())
}

pub(super) struct TestEnvGuard {
    key: &'static str,
    previous: Option<String>,
}

impl TestEnvGuard {
    pub(super) fn set(key: &'static str, value: Option<&str>) -> Self {
        let previous = env::var(key).ok();
        match value {
            Some(value) => env::set_var(key, value),
            None => env::remove_var(key),
        }
        Self { key, previous }
    }
}

impl Drop for TestEnvGuard {
    fn drop(&mut self) {
        if let Some(previous) = self.previous.as_deref() {
            env::set_var(self.key, previous);
        } else {
            env::remove_var(self.key);
        }
    }
}

pub(super) fn unique_p2p_listen_address() -> String {
    let next_port = NEXT_TEST_P2P_PORT.fetch_add(1, Ordering::Relaxed);
    format!("/ip4/127.0.0.1/tcp/{next_port}")
}

pub(super) fn unique_suffix() -> String {
    format!(
        "{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("clock should be monotonic")
            .as_nanos()
    )
}

pub(super) fn temp_json_path(prefix: &str) -> PathBuf {
    std::env::temp_dir().join(format!("{prefix}-{}.json", unique_suffix()))
}

pub(super) fn temp_ndjson_path(prefix: &str) -> PathBuf {
    std::env::temp_dir().join(format!("{prefix}-{}.ndjson", unique_suffix()))
}

pub(super) struct RelayFixturePaths {
    pub(super) state_file: PathBuf,
    pub(super) relay_spool_file: PathBuf,
}

pub(super) fn relay_fixture_paths(prefix: &str) -> RelayFixturePaths {
    RelayFixturePaths {
        state_file: temp_json_path(&format!("{prefix}-state")),
        relay_spool_file: temp_ndjson_path(&format!("{prefix}-spool")),
    }
}

pub(super) fn write_message_state_fixture(
    path: &Path,
    message_id: &str,
    status: &str,
    sender_did: &str,
    recipient_did: &str,
    body: &str,
) {
    write_json_fixture(
        path,
        message_state_payload(message_id, status, sender_did, recipient_did, body),
        "state fixture json",
        "state file fixture should write",
    );
}

pub(super) fn write_empty_state_fixture(path: &Path) {
    write_json_fixture(
        path,
        empty_state_payload(),
        "empty state fixture json",
        "state file fixture should write",
    );
}

pub(super) fn write_spool_entry_fixture(
    path: &Path,
    message_id: &str,
    sender_did: &str,
    recipient_did: &str,
    body: &str,
    queued_at_unix: u64,
) {
    let payload = spool_entry_payload(message_id, sender_did, recipient_did, body, queued_at_unix);
    std::fs::write(
        path,
        format!("{}\n", serde_json::to_string(&payload).expect("spool json")),
    )
    .expect("relay spool fixture should write");
}

pub(super) fn relay_config_json(
    local_peer_id: &str,
    listen_address: &str,
    bootstrap_peers: &[&str],
    topic: Option<&str>,
    recipient_peers_by_did: &[(&str, &str)],
) -> String {
    let recipient_map = recipient_peers_by_did
        .iter()
        .map(|(did, peer_id)| ((*did).to_owned(), (*peer_id).to_owned()))
        .collect::<std::collections::BTreeMap<String, String>>();
    let payload = json!({
        "local_peer_id": local_peer_id,
        "listen_address": listen_address,
        "bootstrap_peers": bootstrap_peers,
        "topic": topic.unwrap_or("messages"),
        "recipient_peers_by_did": recipient_map,
    });
    serde_json::to_string_pretty(&payload).expect("relay config json")
}

pub(super) fn write_relay_fixture(
    paths: &RelayFixturePaths,
    message_id: &str,
    body: &str,
    queued_at_unix: u64,
) {
    write_message_state_fixture(
        paths.state_file.as_path(),
        message_id,
        "created",
        "kamn:did:agent:sender",
        "kamn:did:agent:recipient",
        body,
    );
    write_spool_entry_fixture(
        paths.relay_spool_file.as_path(),
        message_id,
        "kamn:did:agent:sender",
        "kamn:did:agent:recipient",
        body,
        queued_at_unix,
    );
}

pub(super) fn assert_spool_contains(paths: &RelayFixturePaths, message_id: &str) {
    let relay_payload = std::fs::read_to_string(paths.relay_spool_file.as_path())
        .expect("relay spool file should remain readable");
    assert!(relay_payload.contains(message_id));
}

pub(super) fn remove_relay_fixture(paths: &RelayFixturePaths) {
    let _ = std::fs::remove_file(paths.state_file.as_path());
    let _ = std::fs::remove_file(paths.relay_spool_file.as_path());
}
