use super::support::*;
use super::*;

#[path = "tls_contract_tests/tls_mode_regression_contract_tests.rs"]
mod tls_mode_regression_contract_tests;

#[test]
fn integration_runtime_observability_endpoint_tls_mode_serves_required_https_routes() {
    let (cert_file, key_file) =
        super::super::service_api_endpoint_tests::write_test_service_api_tls_materials();
    let snapshot = sample_observability_snapshot();
    let bind_addr = reserve_loopback_addr();
    let endpoint_config = ObservabilityEndpointConfig {
        bind_addr: bind_addr.clone(),
        metrics_path: "/metrics".to_owned(),
        health_path: "/healthz".to_owned(),
        max_requests: 4,
        idle_timeout_ms: 2_000,
    };

    let server_cert_file = cert_file.clone();
    let server_key_file = key_file.clone();
    let server_snapshot = snapshot.clone();
    let server = thread::spawn(move || {
        set_observability_endpoint_tls_mode_override_for_current_thread_for_tests(Some(
            ObservabilityEndpointTlsModeOverride::Require {
                cert_file: server_cert_file,
                key_file: server_key_file,
            },
        ));
        let result = serve_observability_endpoint(&endpoint_config, &server_snapshot);
        set_observability_endpoint_tls_mode_override_for_current_thread_for_tests(None);
        result
    });
    wait_for_https_endpoint_ready(bind_addr.as_str());

    assert!(send_https_get(bind_addr.as_str(), "/metrics").contains("HTTP/1.1 200 OK"));
    assert!(send_https_get(bind_addr.as_str(), "/healthz").contains("HTTP/1.1 200 OK"));
    assert!(send_https_get(bind_addr.as_str(), "/readyz").contains("HTTP/1.1 200 OK"));

    let server_result = server.join().expect("endpoint thread should complete");
    assert!(
        server_result.is_ok(),
        "tls endpoint server should stop cleanly after configured request budget"
    );
}

#[test]
fn integration_runtime_observability_endpoint_tls_mode_rejects_plain_http_handshake() {
    let (cert_file, key_file) =
        super::super::service_api_endpoint_tests::write_test_service_api_tls_materials();
    let snapshot = sample_observability_snapshot();
    let bind_addr = reserve_loopback_addr();
    let endpoint_config = ObservabilityEndpointConfig {
        bind_addr: bind_addr.clone(),
        metrics_path: "/metrics".to_owned(),
        health_path: "/healthz".to_owned(),
        max_requests: 2,
        idle_timeout_ms: 2_000,
    };

    let server = thread::spawn(move || {
        set_observability_endpoint_tls_mode_override_for_current_thread_for_tests(Some(
            ObservabilityEndpointTlsModeOverride::Require {
                cert_file,
                key_file,
            },
        ));
        let result = serve_observability_endpoint(&endpoint_config, &snapshot);
        set_observability_endpoint_tls_mode_override_for_current_thread_for_tests(None);
        result
    });
    wait_for_https_endpoint_ready(bind_addr.as_str());

    if let Ok(response) = try_send_http_get(bind_addr.as_str(), "/metrics") {
        assert!(!response.contains("HTTP/1.1 200 OK"));
    }
    assert!(send_https_get(bind_addr.as_str(), "/metrics").contains("HTTP/1.1 200 OK"));

    let server_result = server.join().expect("endpoint thread should complete");
    assert!(
        server_result.is_ok(),
        "tls endpoint server should stop cleanly after handshake-rejection contract checks"
    );
}
