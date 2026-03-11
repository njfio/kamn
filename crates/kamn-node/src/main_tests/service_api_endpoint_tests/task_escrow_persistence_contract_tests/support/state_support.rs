use super::super::super::*;
use crate::service_api_endpoint::{ServiceApiEndpointConfig, ServiceApiSnapshot};
use std::path::{Path, PathBuf};

pub(crate) fn build_task_escrow_snapshot(api_bind: &str) -> ServiceApiSnapshot {
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

pub(crate) fn unique_named_state_file(prefix: &str) -> PathBuf {
    std::env::temp_dir().join(format!(
        "{prefix}-{}-{}.json",
        std::process::id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock should be monotonic")
            .as_nanos()
    ))
}

pub(crate) fn read_state_json(path: &Path) -> Value {
    let payload = fs::read_to_string(path).expect("state file should remain readable");
    serde_json::from_str(payload.as_str()).expect("state payload should parse")
}

pub(crate) fn set_state_file_env(path: &Path) -> (String, EnvVarGuard) {
    let path_text = path.to_string_lossy().to_string();
    let guard = EnvVarGuard::set("KAMN_SERVICE_API_STATE_FILE", Some(path_text.as_str()));
    (path_text, guard)
}

pub(crate) fn with_api_server<T, F>(
    snapshot: &ServiceApiSnapshot,
    bind_addr: &str,
    max_requests: usize,
    request: F,
) -> T
where
    F: FnOnce(&str) -> T,
{
    let server = spawn_api_server(snapshot, bind_addr, max_requests);
    wait_for_endpoint_ready(bind_addr);
    let response = request(bind_addr);
    assert_api_server_stopped(server);
    response
}

pub(crate) fn state_hash(snapshot: &ServiceApiSnapshot) -> String {
    format!(
        "service-api:{}:{}",
        snapshot.chain_id.as_str(),
        snapshot.chain_version.as_str()
    )
}

fn spawn_api_server(
    snapshot: &ServiceApiSnapshot,
    bind_addr: &str,
    max_requests: usize,
) -> thread::JoinHandle<Result<(), String>> {
    let endpoint_config = ServiceApiEndpointConfig {
        bind_addr: bind_addr.to_owned(),
        max_requests: max_requests as u64,
        idle_timeout_ms: 2_000,
        body_limit_bytes: DEFAULT_SERVICE_API_BODY_LIMIT_BYTES,
        concurrency_limit: DEFAULT_SERVICE_API_CONCURRENCY_LIMIT,
        rate_limit_per_second: DEFAULT_SERVICE_API_RATE_LIMIT_PER_SECOND,
    };
    let server_snapshot = snapshot.clone();
    thread::spawn(move || serve_service_api_endpoint(&endpoint_config, &server_snapshot))
}

fn assert_api_server_stopped(server: thread::JoinHandle<Result<(), String>>) {
    let server_result = server.join().expect("endpoint thread should complete");
    assert!(
        server_result.is_ok(),
        "service api endpoint should stop cleanly"
    );
}
