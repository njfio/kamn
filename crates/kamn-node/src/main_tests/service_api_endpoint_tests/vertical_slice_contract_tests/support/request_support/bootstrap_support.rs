use super::super::super::super::*;
use crate::service_api_endpoint::{ServiceApiEndpointConfig, ServiceApiSnapshot};

pub(crate) fn boot_snapshot(api_bind: &str) -> ServiceApiSnapshot {
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

pub(crate) fn spawn_api_server(
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
    thread::spawn(move || serve_service_api_endpoint(&endpoint_config, &server_snapshot))
}

pub(crate) fn assert_server_ok(server: thread::JoinHandle<Result<(), String>>, context: &str) {
    let result = server.join().expect("endpoint thread should complete");
    assert!(result.is_ok(), "{context}");
}
