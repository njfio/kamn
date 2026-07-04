use super::super::super::*;
use crate::service_api_endpoint::{ServiceApiEndpointConfig, ServiceApiSnapshot};
use std::path::{Path, PathBuf};

const TASK_ESCROW_TEST_IDLE_TIMEOUT_MS: u64 = 5_000;

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
    let process_id = std::process::id();
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock should be monotonic")
        .as_nanos();
    std::env::temp_dir().join(format!("{prefix}-{process_id}-{nanos}.json"))
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
    let chain_id = snapshot.chain_id.as_str();
    let chain_version = snapshot.chain_version.as_str();
    format!("service-api:{chain_id}:{chain_version}")
}

fn spawn_api_server(
    snapshot: &ServiceApiSnapshot,
    bind_addr: &str,
    max_requests: usize,
) -> thread::JoinHandle<Result<(), String>> {
    let endpoint_config = ServiceApiEndpointConfig {
        bind_addr: bind_addr.to_owned(),
        max_requests: max_requests as u64,
        idle_timeout_ms: TASK_ESCROW_TEST_IDLE_TIMEOUT_MS,
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
        "service api endpoint should stop cleanly: {server_result:?}"
    );
}
