const SERVICE_ROOT_SOURCE: &str = include_str!("../src/service.rs");
const SERVICE_RS_MAX_LINES: usize = 1700;
const EXTRACTED_HELPER_SIGNATURE_SNIPPETS: &[&str] = &[
    "fn write_and_flush_request<",
    "fn parse_host_port(",
    "fn normalize_route_segment(",
    "fn validate_http_header_value(",
    "fn validate_endpoint_host(",
    "fn validate_request_method(",
    "fn validate_request_path(",
    "fn render_auth_headers(",
    "fn read_response_bytes<",
    "fn read_response_text<",
];
const EXTRACTED_ENDPOINT_TRANSPORT_SNIPPETS: &[&str] = &[
    "enum ServiceScheme {",
    "enum ServiceStream {",
    "struct ServiceEndpoint {",
    "fn resolve_tls_client_config(",
    "fn resolve_request_timeout_seconds(",
    "fn resolve_tls_server_name(",
];
const EXTRACTED_AUTH_CRYPTO_SNIPPETS: &[&str] = &[
    "pub fn service_signature_for_fields(",
    "pub fn service_signer_public_key_for_fields(",
    "pub fn service_signature_for_state_hash_with_private_key(",
    "pub fn service_public_key_for_private_key(",
    "pub fn service_verify_signature_with_public_key(",
    "fn map_service_auth_error_to_sdk(",
];
const EXTRACTED_REQUEST_AUTH_SNIPPETS: &[&str] = &[
    "pub struct ServiceRequestAuth {",
    "impl ServiceRequestAuth {",
];
const EXTRACTED_SERVICE_MODEL_SNIPPETS: &[&str] = &[
    "pub struct ServiceMessageReceipt {",
    "pub struct ServiceMessageStatus {",
    "pub struct ServiceChannelReceipt {",
    "pub struct ServiceChannelMessages {",
    "pub struct ServiceTaskReceipt {",
    "pub struct ServiceTaskStatus {",
    "pub struct ServiceEscrowStatus {",
    "pub struct ServiceContentRegistration {",
    "pub struct ServiceContentStatus {",
    "pub struct ServiceBridgeSubmission {",
    "pub struct ServiceBridgeStatus {",
    "pub struct ServiceAgentProfile {",
    "pub struct ServiceHealthStatus {",
    "pub struct ServiceRouteEvent {",
];
const EXTRACTED_SERVICE_CLIENT_SNIPPETS: &[&str] = &[
    "struct HttpResponse {",
    "pub struct ServiceApiClient {",
    "impl ServiceApiClient {",
];

#[test]
fn contract_issue_6305_service_root_respects_line_budget() {
    let line_count = SERVICE_ROOT_SOURCE.lines().count();
    assert!(
        line_count <= SERVICE_RS_MAX_LINES,
        "service.rs line budget exceeded: actual={line_count}, max={SERVICE_RS_MAX_LINES}"
    );
}

#[test]
fn contract_issue_6305_service_root_wires_external_test_module() {
    assert!(
        SERVICE_ROOT_SOURCE.contains("#[cfg(test)]")
            && SERVICE_ROOT_SOURCE.contains("#[path = \"service_tests.rs\"]")
            && SERVICE_ROOT_SOURCE.contains("mod tests;"),
        "service.rs must wire #[cfg(test)] #[path = \"service_tests.rs\"] mod tests;"
    );
}

#[test]
fn contract_issue_6325_service_root_declares_http_io_helper_module() {
    assert!(
        SERVICE_ROOT_SOURCE.contains("#[path = \"service_http_io.rs\"]")
            && SERVICE_ROOT_SOURCE.contains("mod service_http_io;"),
        "service.rs must declare #[path = \"service_http_io.rs\"] mod service_http_io;"
    );
}

#[test]
fn contract_issue_6325_service_root_removes_inline_http_io_helper_impls() {
    for helper_signature_snippet in EXTRACTED_HELPER_SIGNATURE_SNIPPETS {
        assert!(
            !SERVICE_ROOT_SOURCE.contains(helper_signature_snippet),
            "service.rs must not retain inline helper impl: {helper_signature_snippet}",
        );
    }
}

#[test]
fn contract_issue_6327_service_root_declares_endpoint_transport_module() {
    assert!(
        SERVICE_ROOT_SOURCE.contains("#[path = \"service_endpoint.rs\"]")
            && SERVICE_ROOT_SOURCE.contains("mod service_endpoint;"),
        "service.rs must declare #[path = \"service_endpoint.rs\"] mod service_endpoint;"
    );
}

#[test]
fn contract_issue_6327_service_root_removes_inline_endpoint_transport_impls() {
    for endpoint_transport_snippet in EXTRACTED_ENDPOINT_TRANSPORT_SNIPPETS {
        assert!(
            !SERVICE_ROOT_SOURCE.contains(endpoint_transport_snippet),
            "service.rs must not retain inline endpoint/transport impl: {endpoint_transport_snippet}",
        );
    }
}

#[test]
fn contract_issue_6329_service_root_declares_auth_crypto_module() {
    assert!(
        SERVICE_ROOT_SOURCE.contains("#[path = \"service_auth_crypto.rs\"]")
            && SERVICE_ROOT_SOURCE.contains("mod service_auth_crypto;"),
        "service.rs must declare #[path = \"service_auth_crypto.rs\"] mod service_auth_crypto;"
    );
}

#[test]
fn contract_issue_6329_service_root_removes_inline_auth_crypto_impls() {
    for auth_crypto_snippet in EXTRACTED_AUTH_CRYPTO_SNIPPETS {
        assert!(
            !SERVICE_ROOT_SOURCE.contains(auth_crypto_snippet),
            "service.rs must not retain inline auth crypto impl: {auth_crypto_snippet}",
        );
    }
}

#[test]
fn contract_issue_6331_service_root_declares_request_auth_module() {
    assert!(
        SERVICE_ROOT_SOURCE.contains("#[path = \"service_request_auth.rs\"]")
            && SERVICE_ROOT_SOURCE.contains("mod service_request_auth;"),
        "service.rs must declare #[path = \"service_request_auth.rs\"] mod service_request_auth;"
    );
}

#[test]
fn contract_issue_6331_service_root_removes_inline_request_auth_impls() {
    for request_auth_snippet in EXTRACTED_REQUEST_AUTH_SNIPPETS {
        assert!(
            !SERVICE_ROOT_SOURCE.contains(request_auth_snippet),
            "service.rs must not retain inline request-auth impl: {request_auth_snippet}",
        );
    }
}

#[test]
fn contract_issue_6333_service_root_declares_models_module() {
    assert!(
        SERVICE_ROOT_SOURCE.contains("#[path = \"service_models.rs\"]")
            && SERVICE_ROOT_SOURCE.contains("mod service_models;"),
        "service.rs must declare #[path = \"service_models.rs\"] mod service_models;"
    );
}

#[test]
fn contract_issue_6333_service_root_removes_inline_model_structs() {
    for model_snippet in EXTRACTED_SERVICE_MODEL_SNIPPETS {
        assert!(
            !SERVICE_ROOT_SOURCE.contains(model_snippet),
            "service.rs must not retain inline service model struct: {model_snippet}",
        );
    }
}

#[test]
fn contract_issue_6335_service_root_declares_client_module() {
    assert!(
        SERVICE_ROOT_SOURCE.contains("#[path = \"service_client.rs\"]")
            && SERVICE_ROOT_SOURCE.contains("mod service_client;"),
        "service.rs must declare #[path = \"service_client.rs\"] mod service_client;"
    );
}

#[test]
fn contract_issue_6335_service_root_removes_inline_client_orchestration_defs() {
    for client_snippet in EXTRACTED_SERVICE_CLIENT_SNIPPETS {
        assert!(
            !SERVICE_ROOT_SOURCE.contains(client_snippet),
            "service.rs must not retain inline client orchestration definition: {client_snippet}",
        );
    }
}
