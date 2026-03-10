use super::super::support::*;
use super::super::*;

fn spawn_server(
    snapshot: RuntimeObservabilitySnapshot,
    max_requests: u64,
) -> (String, thread::JoinHandle<Result<(), String>>) {
    let bind_addr = reserve_loopback_addr();
    let endpoint_config = ObservabilityEndpointConfig {
        bind_addr: bind_addr.clone(),
        metrics_path: "/metrics".to_owned(),
        health_path: "/healthz".to_owned(),
        max_requests,
        idle_timeout_ms: 2_000,
    };
    let server = thread::spawn(move || serve_observability_endpoint(&endpoint_config, &snapshot));
    wait_for_endpoint_ready(bind_addr.as_str());
    (bind_addr, server)
}

#[test]
fn integration_runtime_observability_endpoint_returns_not_found_for_malformed_request_method() {
    let (bind_addr, server) = spawn_server(sample_observability_snapshot(), 2);
    let malformed_response = send_raw_http_request(
        bind_addr.as_str(),
        "POST /metrics HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n",
    );
    assert!(malformed_response.contains("HTTP/1.1 404 Not Found"));
    assert!(server
        .join()
        .expect("endpoint thread should complete")
        .is_ok());
}

#[test]
fn integration_runtime_observability_endpoint_returns_not_found_for_get_with_non_empty_body() {
    let (bind_addr, server) = spawn_server(sample_observability_snapshot(), 2);
    let malformed_response = send_raw_http_request(
        bind_addr.as_str(),
        "GET /metrics HTTP/1.1\r\nHost: localhost\r\nContent-Length: 4\r\nConnection: close\r\n\r\nnope",
    );
    assert!(malformed_response.contains("HTTP/1.1 404 Not Found"));
    assert!(server
        .join()
        .expect("endpoint thread should complete")
        .is_ok());
}

#[test]
fn integration_runtime_observability_endpoint_fails_closed_on_idle_timeout() {
    let snapshot = sample_observability_snapshot();
    let endpoint_config = ObservabilityEndpointConfig {
        bind_addr: reserve_loopback_addr(),
        metrics_path: "/metrics".to_owned(),
        health_path: "/healthz".to_owned(),
        max_requests: 1,
        idle_timeout_ms: 200,
    };
    let server = thread::spawn(move || serve_observability_endpoint(&endpoint_config, &snapshot));
    let error = server
        .join()
        .expect("endpoint thread should complete")
        .expect_err("idle timeout should fail closed when no requests arrive");
    assert_eq!(
        error,
        "observability endpoint timed out after 200 ms waiting for requests"
    );
}
