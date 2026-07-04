use super::support::*;

const ROUTE_RENDER_SUBMODULE_MARKERS: &[&str] = &[
    "mod route_response_contract_tests;",
    "mod route_metrics_contract_tests;",
];
const ROUTE_RESPONSE_MARKERS: &[&str] = &[
    "fn functional_service_api_endpoint_renders_required_route_contracts()",
    "let send_response = render_service_api_endpoint_response(",
    "let bridge_query_response =",
    "let health_response = render_service_api_endpoint_response(&snapshot, \"GET\", \"/healthz\", \"\");",
];
const ROUTE_METRICS_MARKERS: &[&str] = &[
    "kamn_service_api_route_authz_matrix_total_route_count {}",
    "kamn_service_api_scope_policy_fixture_unique_allow_route_count",
    "kamn_service_api_websocket_reason_taxonomy_info",
    "service_api_websocket_upgrade_required",
];

#[test]
fn spec_c09_service_api_endpoint_root_file_removes_moved_route_rendering_contract() {
    let source = read_repo_file(ROOT_FILE);
    for marker in [
        "fn functional_service_api_endpoint_renders_required_route_contracts()",
        "let send_response = render_service_api_endpoint_response(",
        "let metrics_response = render_service_api_endpoint_response(&snapshot, \"GET\", \"/metrics\", \"\");",
        "kamn_service_api_route_authz_matrix_total_route_count {}",
        "service_api_websocket_upgrade_required",
    ] {
        assert!(
            !source.contains(marker),
            "service_api_endpoint_tests.rs should not keep moved route/rendering marker: {marker}"
        );
    }
}

#[test]
fn spec_c10_service_api_endpoint_route_render_module_exists_and_owns_moved_coverage() {
    let module_source = read_repo_file(ROUTE_RENDER_MODULE_FILE);
    let route_response = read_repo_file(ROUTE_RESPONSE_FILE);
    let route_metrics = read_repo_file(ROUTE_METRICS_FILE);

    assert_contains_markers(
        module_source.as_str(),
        ROUTE_RENDER_SUBMODULE_MARKERS,
        "route-render module",
    );
    assert_contains_markers(
        route_response.as_str(),
        ROUTE_RESPONSE_MARKERS,
        "route-response contract file",
    );
    assert_contains_markers(
        route_metrics.as_str(),
        ROUTE_METRICS_MARKERS,
        "route-metrics contract file",
    );
}

#[test]
fn spec_c11_service_api_endpoint_root_declares_route_render_submodule() {
    let source = read_repo_file(ROOT_FILE);
    assert!(
        source.contains("mod route_render_contract_tests;"),
        "service_api_endpoint_tests.rs should declare route/render submodule"
    );
}

#[test]
fn spec_c12_service_api_endpoint_route_render_split_files_stay_below_budget() {
    for path in [
        ROUTE_RENDER_MODULE_FILE,
        ROUTE_RESPONSE_FILE,
        ROUTE_METRICS_FILE,
    ] {
        let source = read_repo_file(path);
        let line_count = source.lines().count();
        assert!(
            line_count <= 200,
            "{path} should stay below 200 lines after extraction: line_count={line_count}"
        );
    }
}
