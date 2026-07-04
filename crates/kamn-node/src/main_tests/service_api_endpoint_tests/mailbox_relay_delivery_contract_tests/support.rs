use super::super::*;
pub(super) use super::request_support::{
    list_mailbox, query_message, relay_message, send_message, send_signed_request,
};
use crate::service_api_endpoint::ServiceApiSnapshot;
use std::path::{Path, PathBuf};

pub(super) fn build_mailbox_relay_snapshot(api_bind: &str) -> ServiceApiSnapshot {
    let parsed = parse_args(vec![
        "kamn-node".to_owned(),
        "--role".to_owned(),
        "processor".to_owned(),
        "--runtime-mode".to_owned(),
        "api".to_owned(),
        "--api-bind".to_owned(),
        api_bind.to_owned(),
    ])
    .expect("api args should parse");
    let report = execute(parsed).expect("api execution should succeed");
    build_service_api_snapshot(&report)
}

pub(super) fn unique_named_state_file(prefix: &str) -> PathBuf {
    unique_named_path(prefix, "json")
}

pub(super) fn unique_named_relay_spool_file(prefix: &str) -> PathBuf {
    unique_named_path(prefix, "ndjson")
}

fn unique_named_path(prefix: &str, extension: &str) -> PathBuf {
    std::env::temp_dir().join(format!(
        "{prefix}-{}-{}.{}",
        std::process::id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock should be monotonic")
            .as_nanos(),
        extension,
    ))
}

pub(super) fn set_state_file_env(path: &Path) -> EnvVarGuard {
    let path_text = path.to_string_lossy().to_string();
    EnvVarGuard::set("KAMN_SERVICE_API_STATE_FILE", Some(path_text.as_str()))
}

pub(super) fn set_relay_spool_env(path: &Path) -> EnvVarGuard {
    let path_text = path.to_string_lossy().to_string();
    EnvVarGuard::set(
        "KAMN_SERVICE_API_RELAY_SPOOL_FILE",
        Some(path_text.as_str()),
    )
}

pub(super) fn read_state_json(path: &Path) -> Value {
    let payload = fs::read_to_string(path).expect("state file should remain readable");
    serde_json::from_str(payload.as_str()).expect("state payload should parse")
}

pub(super) fn spawn_api_server(
    snapshot: &ServiceApiSnapshot,
    bind_addr: &str,
    max_requests: u64,
) -> thread::JoinHandle<Result<(), String>> {
    let endpoint_config = ServiceApiEndpointConfig {
        bind_addr: bind_addr.to_owned(),
        max_requests,
        idle_timeout_ms: 2_000,
        body_limit_bytes: DEFAULT_SERVICE_API_BODY_LIMIT_BYTES,
        concurrency_limit: DEFAULT_SERVICE_API_CONCURRENCY_LIMIT,
        rate_limit_per_second: DEFAULT_SERVICE_API_RATE_LIMIT_PER_SECOND,
    };
    let server_snapshot = snapshot.clone();
    let server =
        thread::spawn(move || serve_service_api_endpoint(&endpoint_config, &server_snapshot));
    wait_for_endpoint_ready(bind_addr);
    server
}

pub(super) fn assert_server_ok(server: thread::JoinHandle<Result<(), String>>, context: &str) {
    let result = server.join().expect("endpoint thread should complete");
    assert!(result.is_ok(), "{context}");
}

pub(super) fn state_hash(snapshot: &ServiceApiSnapshot) -> String {
    format!(
        "service-api:{}:{}",
        snapshot.chain_id.as_str(),
        snapshot.chain_version.as_str()
    )
}
