use super::support::*;
use super::*;
use std::thread;

#[path = "async_regression_contract_tests/negative_path_contract_tests.rs"]
mod negative_path_contract_tests;

fn observability_endpoint_source_bundle() -> String {
    [
        include_str!("../../observability_endpoint.rs"),
        include_str!("../../observability_endpoint/endpoint_server.rs"),
        include_str!("../../observability_endpoint/payload_contract.rs"),
        include_str!("../../observability_endpoint/payload_render.rs"),
        include_str!("../../observability_endpoint/tls_mode.rs"),
    ]
    .join("\n")
}

#[test]
fn integration_runtime_observability_endpoint_returns_not_found_for_unknown_path() {
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
    let unknown_response = send_http_get(bind_addr.as_str(), "/unknown");
    assert!(unknown_response.contains("HTTP/1.1 404 Not Found"));
    assert!(server
        .join()
        .expect("endpoint thread should complete")
        .is_ok());
}

#[test]
fn regression_observability_endpoint_export_keeps_bootstrap_report_rendering_unchanged() {
    let parsed = parse_args_with_clean_daemon_env(vec![
        "kamn-node".to_owned(),
        "--role".to_owned(),
        "processor".to_owned(),
        "--runtime-mode".to_owned(),
        "daemon".to_owned(),
        "--daemon-max-ticks".to_owned(),
        "3".to_owned(),
        "--daemon-tick-interval-ms".to_owned(),
        "25".to_owned(),
    ])
    .expect("daemon args should parse");
    let report = execute(parsed).expect("daemon execution should succeed");
    let before = render_bootstrap_report(&report, OutputMode::json());
    let snapshot =
        build_runtime_observability_snapshot(&report).expect("daemon report should map snapshot");
    let _metrics = render_observability_endpoint_response(&snapshot, "/metrics");
    let _health = render_observability_endpoint_response(&snapshot, "/healthz");
    let after = render_bootstrap_report(&report, OutputMode::json());
    assert_eq!(before, after);
}

#[test]
fn regression_observability_endpoint_uses_async_listener_serving_path() {
    let source = observability_endpoint_source_bundle();
    assert!(source.contains("tokio::net::TcpListener::bind("));
    assert!(source.contains("async fn serve_observability_endpoint_async("));
    assert!(
        source.contains("runtime.block_on(serve_observability_endpoint_async(")
            || source
                .contains("runtime.block_on(endpoint_server::serve_observability_endpoint_async(")
    );
}

#[test]
fn regression_observability_endpoint_uses_axum_route_composition() {
    let source = observability_endpoint_source_bundle();
    assert!(source.contains("fn build_observability_endpoint_router("));
    assert!(source.contains("Router::new()"));
    assert!(source.contains(".route(\"/\", any(handle_observability_http_route))"));
    assert!(source.contains(".route(\"/{*path}\", any(handle_observability_http_route))"));
}

#[test]
fn regression_observability_endpoint_keeps_async_negative_matrix_contracts() {
    let source = observability_endpoint_source_bundle();
    assert!(source.contains("handle_observability_not_found_path().await"));
    assert!(source.contains("if method != Method::GET"));
    assert!(source.contains("request_has_non_empty_body(&headers)"));
    assert!(source.contains("observability endpoint timed out after {} ms waiting for requests"));
}
