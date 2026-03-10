use super::support::*;

#[test]
fn spec_c45_service_api_endpoint_root_file_removes_moved_shared_helper_surface() {
    let source = read_repo_file(ROOT_FILE);
    for marker in [
        "const TEST_SERVICE_API_TLS_CERT_PEM: &str =",
        "struct ServiceApiErrorEnvelope {",
        "struct ServiceApiRouteAuthzMatrixRow {",
        "struct ServiceApiScopePolicyFixtureRow {",
        "struct TestSkipServerVerification(",
        "struct ServiceApiTestEnvGuards {",
        "fn service_api_route_authz_matrix_rows() -> Vec<ServiceApiRouteAuthzMatrixRow> {",
        "fn parse_service_api_scope_policy_fixture(",
        "fn required_scope_for_test_route(method: &str, path: &str) -> Option<&'static str> {",
        "fn test_service_api_auth_public_key_hex() -> String {",
        "fn send_http_request_with_headers_raw(",
        "async fn send_http_request_with_headers_async(",
        "fn send_https_request_with_headers_raw(",
        "fn parse_http_content_length(response_head: &str) -> usize {",
        "fn acquire_service_api_test_env() -> ServiceApiTestEnvGuards {",
    ] {
        assert!(
            !source.contains(marker),
            "service_api_endpoint_tests.rs should not keep moved shared helper marker: {marker}"
        );
    }
}

#[test]
fn spec_c46_service_api_endpoint_shared_support_module_exists_and_owns_helper_surface() {
    let module_source = read_repo_file(SHARED_SUPPORT_MODULE_FILE);
    let auth_support = read_repo_file(AUTH_FIXTURE_SUPPORT_FILE);
    let route_scope = read_repo_file(ROUTE_SCOPE_SUPPORT_FILE);
    let http_transport = read_repo_file(HTTP_TRANSPORT_SUPPORT_FILE);
    let tls_transport = read_repo_file(TLS_TRANSPORT_SUPPORT_FILE);
    let response_support = read_repo_file(RESPONSE_SUPPORT_FILE);
    let env_support = read_repo_file(ENV_SUPPORT_FILE);

    assert_shared_support_modules_declared(module_source.as_str());
    assert_auth_fixture_support_markers(auth_support.as_str());
    assert_route_scope_support_markers(route_scope.as_str());
    assert_http_transport_support_markers(http_transport.as_str());
    assert_tls_transport_support_markers(tls_transport.as_str());
    assert_response_support_markers(response_support.as_str());
    assert_env_support_markers(env_support.as_str());
}

fn assert_shared_support_modules_declared(source: &str) {
    assert_file_markers(
        source,
        &[
            "mod auth_fixture_support;",
            "mod route_scope_support;",
            "mod http_transport_support;",
            "mod tls_transport_support;",
            "mod response_support;",
            "mod env_support;",
        ],
        "shared_support.rs",
    );
}

fn assert_auth_fixture_support_markers(source: &str) {
    assert_file_markers(
        source,
        &[
            "struct ServiceApiErrorEnvelope {",
            "const SERVICE_API_AUTH_MISSING_HEADER_REASON_CODE: &str =",
            "fn test_service_api_auth_public_key_hex() -> String {",
            "fn test_service_api_sender_did(sender: &str) -> String {",
            "fn service_api_request_signature_for_fields(",
        ],
        "auth fixture support file",
    );
}

fn assert_route_scope_support_markers(source: &str) {
    assert_file_markers(
        source,
        &[
            "struct ServiceApiRouteAuthzMatrixRow {",
            "struct ServiceApiScopePolicyFixtureRow {",
            "fn service_api_route_authz_matrix_rows() -> Vec<ServiceApiRouteAuthzMatrixRow> {",
            "fn parse_service_api_scope_policy_fixture(",
            "fn required_scope_for_test_route(method: &str, path: &str) -> Option<&'static str> {",
            "fn enrich_signed_headers_with_scope(",
        ],
        "route/scope support file",
    );
}

fn assert_http_transport_support_markers(source: &str) {
    assert_file_markers(
        source,
        &[
            "fn reserve_loopback_addr() -> String {",
            "fn send_http_request(addr: &str, method: &str, path: &str, body: &str) -> String {",
            "fn send_http_request_with_headers(",
            "fn send_http_request_with_headers_raw(",
            "async fn send_http_request_with_headers_async(",
        ],
        "http transport support file",
    );
}

fn assert_tls_transport_support_markers(source: &str) {
    assert_file_markers(
        source,
        &[
            "const TEST_SERVICE_API_TLS_CERT_PEM: &str =",
            "const TEST_SERVICE_API_TLS_KEY_PEM: &str =",
            "struct TestSkipServerVerification(",
            "fn send_https_request_with_headers(",
            "fn send_https_request_with_headers_raw(",
            "fn write_test_service_api_tls_materials() -> (String, String) {",
        ],
        "tls transport support file",
    );
}

fn assert_response_support_markers(source: &str) {
    assert_file_markers(
        source,
        &[
            "fn parse_http_content_length(response_head: &str) -> usize {",
            "fn extract_http_response_body(response: &str) -> &str {",
            "fn parse_error_envelope(body: &str) -> ServiceApiErrorEnvelope {",
            "fn parse_error_envelope_from_http_response(response: &str) -> ServiceApiErrorEnvelope {",
            "fn parse_scalar_metric_value(response: &str, metric_name: &str) -> Option<u64> {",
            "fn read_single_http_response(stream: &mut TcpStream) -> String {",
            "fn wait_for_endpoint_ready(addr: &str) {",
        ],
        "response support file",
    );
}

fn assert_env_support_markers(source: &str) {
    assert_file_markers(
        source,
        &[
            "struct ServiceApiTestEnvGuards {",
            "fn unique_service_api_test_state_file_path() -> String {",
            "fn acquire_service_api_test_env() -> ServiceApiTestEnvGuards {",
        ],
        "env support file",
    );
}

#[test]
fn spec_c47_service_api_endpoint_root_declares_shared_support_submodule() {
    let source = read_repo_file(ROOT_FILE);
    assert!(
        source.contains("mod shared_support;"),
        "service_api_endpoint_tests.rs should declare shared-support submodule"
    );
}

#[test]
fn spec_c48_service_api_endpoint_shared_support_files_stay_below_budget() {
    for path in [
        SHARED_SUPPORT_MODULE_FILE,
        AUTH_FIXTURE_SUPPORT_FILE,
        ROUTE_SCOPE_SUPPORT_FILE,
        HTTP_TRANSPORT_SUPPORT_FILE,
        TLS_TRANSPORT_SUPPORT_FILE,
        RESPONSE_SUPPORT_FILE,
        ENV_SUPPORT_FILE,
    ] {
        let source = read_repo_file(path);
        let line_count = source.lines().count();
        assert!(
            line_count <= 200,
            "{path} should stay below 200 lines after extraction: line_count={line_count}"
        );
    }
}
