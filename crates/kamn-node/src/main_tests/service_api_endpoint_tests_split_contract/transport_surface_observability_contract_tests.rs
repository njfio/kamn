use super::support::*;

#[test]
fn spec_c41_service_api_endpoint_root_file_removes_moved_transport_surface_contracts() {
    let source = read_repo_file(ROOT_FILE);
    for marker in [
        "fn integration_service_api_endpoint_serves_required_http_routes()",
        "fn regression_service_api_runtime_observability_projects_live_metrics_under_traffic()",
        "fn integration_service_api_endpoint_async_runtime_handles_concurrent_http_routes()",
        "fn integration_service_api_endpoint_tls_mode_serves_required_https_routes()",
        "fn regression_service_api_endpoint_tls_mode_rejects_missing_cert_file()",
        "fn regression_service_api_endpoint_rejects_disabled_tls_for_non_loopback_api_runtime_path()",
        "fn integration_service_api_endpoint_http_response_bodies_match_serde_contracts()",
        "fn integration_service_api_endpoint_supports_keep_alive_requests_on_single_connection()",
        "fn functional_service_api_endpoint_emits_structured_ingress_correlation_markers()",
        "fn unit_service_api_endpoint_metrics_use_runtime_observability_when_present()",
    ] {
        assert!(
            !source.contains(marker),
            "service_api_endpoint_tests.rs should not keep moved transport-surface marker: {marker}"
        );
    }
}

#[test]
fn spec_c42_service_api_endpoint_transport_surface_module_exists_and_owns_moved_coverage() {
    let module_source = read_repo_file(TRANSPORT_SURFACE_OBSERVABILITY_MODULE_FILE);
    let route_tls = read_repo_file(ROUTE_TLS_FILE);
    let http_connection = read_repo_file(HTTP_CONNECTION_FILE);
    let observability = read_repo_file(OBSERVABILITY_FILE);
    let support = read_repo_file(TRANSPORT_SUPPORT_FILE);

    assert_transport_surface_module_declarations(module_source.as_str());
    assert_transport_surface_markers(
        route_tls.as_str(),
        http_connection.as_str(),
        observability.as_str(),
        support.as_str(),
    );
}

fn assert_transport_surface_module_declarations(module_source: &str) {
    for marker in [
        "mod route_tls_contract_tests;",
        "mod http_connection_contract_tests;",
        "mod observability_contract_tests;",
        "mod support;",
    ] {
        assert!(
            module_source.contains(marker),
            "transport_surface_observability_contract_tests.rs should declare submodule marker: {marker}"
        );
    }
}

fn assert_transport_surface_markers(
    route_tls: &str,
    http_connection: &str,
    observability: &str,
    support: &str,
) {
    assert_transport_route_tls_markers(route_tls);
    assert_transport_http_connection_markers(http_connection);
    assert_transport_observability_markers(observability);
    assert!(
        support.contains("fn build_transport_snapshot("),
        "transport support file should include shared transport snapshot helper"
    );
}

fn assert_transport_route_tls_markers(source: &str) {
    assert_transport_surface_file_markers(
        source,
        &[
            "fn integration_service_api_endpoint_serves_required_http_routes()",
            "fn integration_service_api_endpoint_async_runtime_handles_concurrent_http_routes()",
            "fn integration_service_api_endpoint_tls_mode_serves_required_https_routes()",
            "fn regression_service_api_endpoint_tls_mode_rejects_missing_cert_file()",
            "fn regression_service_api_endpoint_rejects_disabled_tls_for_non_loopback_api_runtime_path()",
            "fn integration_service_api_endpoint_http_response_bodies_match_serde_contracts()",
        ],
        "route/tls contract file",
    );
}

fn assert_transport_http_connection_markers(source: &str) {
    assert_transport_surface_file_markers(
        source,
        &["fn integration_service_api_endpoint_supports_keep_alive_requests_on_single_connection()"],
        "http connection contract file",
    );
}

fn assert_transport_observability_markers(source: &str) {
    assert_transport_surface_file_markers(
        source,
        &[
            "fn regression_service_api_runtime_observability_projects_live_metrics_under_traffic()",
            "fn functional_service_api_endpoint_emits_structured_ingress_correlation_markers()",
            "fn unit_service_api_endpoint_metrics_use_runtime_observability_when_present()",
        ],
        "observability contract file",
    );
}

fn assert_transport_surface_file_markers(source: &str, markers: &[&str], label: &str) {
    for marker in markers {
        assert!(
            source.contains(marker),
            "{label} should include moved marker: {marker}"
        );
    }
}

#[test]
fn spec_c43_service_api_endpoint_root_declares_transport_surface_submodule() {
    let source = read_repo_file(ROOT_FILE);
    assert!(
        source.contains("mod transport_surface_observability_contract_tests;"),
        "service_api_endpoint_tests.rs should declare transport-surface-observability submodule"
    );
}

#[test]
fn spec_c44_service_api_endpoint_transport_surface_split_files_stay_below_budget() {
    for path in [
        TRANSPORT_SURFACE_OBSERVABILITY_MODULE_FILE,
        ROUTE_TLS_FILE,
        HTTP_CONNECTION_FILE,
        OBSERVABILITY_FILE,
        TRANSPORT_SUPPORT_FILE,
    ] {
        let source = read_repo_file(path);
        let line_count = source.lines().count();
        assert!(
            line_count <= 200,
            "{path} should stay below 200 lines after extraction: line_count={line_count}"
        );
    }
}
