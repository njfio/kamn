use super::support::*;

#[test]
fn unit_service_api_client_rejects_invalid_endpoint_scheme() {
    assert_eq!(
        ServiceApiClient::connect("tcp://127.0.0.1:35001"),
        Err(SdkError::InvalidInput {
            field: "service.endpoint",
            reason: "must start with http:// or https://",
        })
    );
}

#[test]
fn spec_c01_service_api_client_executes_https_health_route_with_trusted_ca() {
    let _tls_env_lock = tls_env_lock()
        .lock()
        .expect("tls env lock should not be poisoned");
    let mut server = spawn_https_single_request_server(
        200,
        r#"{"status":"ok","runtime_mode":"api","role":"processor","observability_source":"unknown","observability_health":"unknown"}"#,
    );
    let ca_file = server.ca_cert_path.to_string_lossy().to_string();
    let _ca_guard = EnvVarGuard::set(SERVICE_TLS_CA_FILE_ENV, Some(ca_file.as_str()));
    let client =
        ServiceApiClient::connect(server.base_url.as_str()).expect("https client should construct");
    let health = client
        .health()
        .expect("trusted CA should allow https service route request");
    assert_eq!(health.status, "ok");
    assert_eq!(health.runtime_mode, "api");
    server.wait_for_exit();
}

#[test]
fn spec_c02_service_api_client_rejects_untrusted_https_certificate_chain() {
    let _tls_env_lock = tls_env_lock()
        .lock()
        .expect("tls env lock should not be poisoned");
    let server = spawn_https_single_request_server(
        200,
        r#"{"status":"ok","runtime_mode":"api","role":"processor","observability_source":"unknown","observability_health":"unknown"}"#,
    );
    let _ca_guard = EnvVarGuard::set(SERVICE_TLS_CA_FILE_ENV, None);
    let client =
        ServiceApiClient::connect(server.base_url.as_str()).expect("https client should construct");
    let error = client
        .health()
        .expect_err("untrusted cert chain must fail closed");
    assert_eq!(
        error,
        SdkError::TransportFailure("service tls certificate verification failed")
    );
}

#[test]
fn spec_c02_service_api_client_rejects_missing_tls_ca_bundle_path() {
    let _tls_env_lock = tls_env_lock()
        .lock()
        .expect("tls env lock should not be poisoned");
    let server = spawn_https_single_request_server(
        200,
        r#"{"status":"ok","runtime_mode":"api","role":"processor","observability_source":"unknown","observability_health":"unknown"}"#,
    );
    let missing_ca_file = server
        .temp_dir
        .join("missing-ca.pem")
        .to_string_lossy()
        .to_string();
    let _ca_guard = EnvVarGuard::set(SERVICE_TLS_CA_FILE_ENV, Some(missing_ca_file.as_str()));
    let client =
        ServiceApiClient::connect(server.base_url.as_str()).expect("https client should construct");
    let error = client
        .health()
        .expect_err("missing TLS CA bundle path must fail closed");
    assert_eq!(
        error,
        SdkError::TransportFailure("service tls ca file read failed")
    );
}
