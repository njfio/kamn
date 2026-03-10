use super::support::*;

#[path = "tls_contract_tests/tls_mode_regression_contract_tests.rs"]
mod tls_mode_regression_contract_tests;

fn test_tls_materials() -> (String, String) {
    super::super::service_api_endpoint_tests::write_test_service_api_tls_materials()
}

#[test]
fn integration_runtime_observability_endpoint_tls_mode_serves_required_https_routes() {
    let (cert_file, key_file) = test_tls_materials();
    let snapshot = sample_observability_snapshot();
    let (bind_addr, server) =
        spawn_tls_observability_server(&snapshot, 4, 2_000, cert_file, key_file);

    assert!(send_https_get(bind_addr.as_str(), "/metrics").contains("HTTP/1.1 200 OK"));
    assert!(send_https_get(bind_addr.as_str(), "/healthz").contains("HTTP/1.1 200 OK"));
    assert!(send_https_get(bind_addr.as_str(), "/readyz").contains("HTTP/1.1 200 OK"));
    assert!(server
        .join()
        .expect("endpoint thread should complete")
        .is_ok());
}

#[test]
fn integration_runtime_observability_endpoint_tls_mode_rejects_plain_http_handshake() {
    let (cert_file, key_file) = test_tls_materials();
    let snapshot = sample_observability_snapshot();
    let (bind_addr, server) =
        spawn_tls_observability_server(&snapshot, 2, 2_000, cert_file, key_file);

    if let Ok(response) = try_send_http_get(bind_addr.as_str(), "/metrics") {
        assert!(!response.contains("HTTP/1.1 200 OK"));
    }
    assert!(send_https_get(bind_addr.as_str(), "/metrics").contains("HTTP/1.1 200 OK"));
    assert!(server
        .join()
        .expect("endpoint thread should complete")
        .is_ok());
}
