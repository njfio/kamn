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
