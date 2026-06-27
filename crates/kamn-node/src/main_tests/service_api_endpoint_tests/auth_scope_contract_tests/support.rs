use super::*;

pub(super) fn start_service_api_server(
    api_bind: &str,
    max_requests: u64,
) -> (
    crate::service_api_endpoint::ServiceApiSnapshot,
    String,
    thread::JoinHandle<Result<(), String>>,
    ServiceApiTestEnvGuards,
) {
    let env = acquire_service_api_test_env();
    let snapshot = build_service_api_test_snapshot(api_bind);
    let bind_addr = reserve_loopback_addr();
    let server = spawn_service_api_server(&snapshot, bind_addr.as_str(), max_requests);
    (snapshot, bind_addr, server, env)
}

fn build_service_api_test_snapshot(
    api_bind: &str,
) -> crate::service_api_endpoint::ServiceApiSnapshot {
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

fn spawn_service_api_server(
    snapshot: &crate::service_api_endpoint::ServiceApiSnapshot,
    bind_addr: &str,
    max_requests: u64,
) -> thread::JoinHandle<Result<(), String>> {
    let endpoint_config = ServiceApiEndpointConfig {
        bind_addr: bind_addr.to_owned(),
        max_requests,
        idle_timeout_ms: 2_500,
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

pub(super) fn join_service_api_server(
    server: thread::JoinHandle<Result<(), String>>,
    message: &str,
) {
    let server_result = server.join().expect("endpoint thread should complete");
    assert!(server_result.is_ok(), "{message}");
}

pub(super) fn service_api_request_state_hash(
    snapshot: &crate::service_api_endpoint::ServiceApiSnapshot,
) -> String {
    format!(
        "service-api:{}:{}",
        snapshot.chain_id.as_str(),
        snapshot.chain_version.as_str()
    )
}
