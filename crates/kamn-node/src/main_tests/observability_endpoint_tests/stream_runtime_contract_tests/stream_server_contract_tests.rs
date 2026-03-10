use super::super::support::*;
use super::super::*;

fn assert_ok_join(server: thread::JoinHandle<Result<(), String>>) {
    assert!(server
        .join()
        .expect("endpoint thread should complete")
        .is_ok());
}

#[test]
fn integration_runtime_observability_endpoint_serves_stream_path() {
    let snapshot = daemon_observability_snapshot();
    let (bind_addr, server) = spawn_observability_server(&snapshot, 2, 2_000);
    let stream_response = send_http_get(bind_addr.as_str(), "/metrics.stream");
    assert!(stream_response.contains("HTTP/1.1 200 OK"));
    assert!(stream_response.contains("application/x-ndjson"));
    assert_ok_join(server);
}

#[test]
fn integration_runtime_observability_endpoint_handles_concurrent_metrics_and_stream_requests() {
    let snapshot = daemon_observability_snapshot();
    let (bind_addr, server) = spawn_observability_server(&snapshot, 5, 2_000);
    let barrier = Arc::new(Barrier::new(3));
    let metrics_barrier = Arc::clone(&barrier);
    let metrics_addr = bind_addr.clone();
    let metrics_worker = thread::spawn(move || {
        metrics_barrier.wait();
        send_http_get(metrics_addr.as_str(), "/metrics")
    });
    let stream_barrier = Arc::clone(&barrier);
    let stream_addr = bind_addr.clone();
    let stream_worker = thread::spawn(move || {
        stream_barrier.wait();
        send_http_get(stream_addr.as_str(), "/metrics.stream")
    });
    barrier.wait();
    assert!(metrics_worker
        .join()
        .expect("metrics worker thread should complete")
        .contains("HTTP/1.1 200 OK"));
    assert!(stream_worker
        .join()
        .expect("stream worker thread should complete")
        .contains("HTTP/1.1 200 OK"));
    assert!(send_http_get(bind_addr.as_str(), "/healthz").contains("HTTP/1.1 200 OK"));
    assert!(send_http_get(bind_addr.as_str(), "/readyz").contains("HTTP/1.1 200 OK"));
    assert_ok_join(server);
}

#[test]
fn integration_runtime_observability_endpoint_enforces_queue_bound_request_budget() {
    let snapshot = sample_observability_snapshot();
    let bind_addr = reserve_loopback_addr();
    let endpoint_config = ObservabilityEndpointConfig {
        bind_addr: bind_addr.clone(),
        metrics_path: "/metrics".to_owned(),
        health_path: "/healthz".to_owned(),
        max_requests: 2,
        idle_timeout_ms: 2_000,
    };

    let server_snapshot = snapshot.clone();
    let server =
        thread::spawn(move || serve_observability_endpoint(&endpoint_config, &server_snapshot));
    wait_for_endpoint_ready(bind_addr.as_str());
    assert!(send_http_get(bind_addr.as_str(), "/metrics").contains("HTTP/1.1 200 OK"));
    assert!(server
        .join()
        .expect("endpoint thread should complete")
        .is_ok());
    assert!(try_send_http_get(bind_addr.as_str(), "/metrics").is_err());
}
