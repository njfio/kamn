use super::service_api_policy_support::assert_remediation_marker_in_docs_and_ops;
use super::*;

#[test]
fn doc_contains_service_api_request_path_authz_docs_parity_markers() {
    assert_request_path_authz_doc_headers();
    assert_request_path_authz_doc_paths();
    assert_request_path_authz_doc_commands();
    assert!(DOC.contains("Regression: #4057"));
}

#[test]
fn doc_enforces_service_api_request_path_authz_docs_parity_matches_source_taxonomy() {
    assert_request_path_authz_source_markers();
    assert_request_path_authz_strategy_markers();
    assert_request_path_authz_ops_markers();
}

#[test]
fn doc_enforces_service_api_request_path_authz_remediation_markers_cover_reason_codes() {
    for reason_code in service_api_request_path_authz_reason_codes() {
        assert_request_path_authz_remediation_markers(reason_code);
    }
}

fn assert_request_path_authz_doc_headers() {
    assert!(DOC.contains("### Service API Request-Path Authz Matrix and Docs Parity Contract"));
    assert!(DOC.contains(&format!(
        "service_api_request_path_authz_reason_taxonomy_version={SERVICE_API_REQUEST_PATH_AUTHZ_REASON_TAXONOMY_VERSION}"
    )));
    assert!(DOC.contains(&format!(
        "service_api_request_path_authz_reason_codes_csv={SERVICE_API_REQUEST_PATH_AUTHZ_REASON_CODES_CSV}"
    )));
}

fn assert_request_path_authz_doc_paths() {
    assert!(DOC.contains(&format!(
        "service_api_request_path_authz_public_routes_csv={SERVICE_API_REQUEST_PATH_AUTHZ_PUBLIC_ROUTES_CSV}"
    )));
    assert!(DOC.contains(&format!(
        "service_api_request_path_authz_protected_routes_csv={SERVICE_API_REQUEST_PATH_AUTHZ_PROTECTED_ROUTES_CSV}"
    )));
    assert!(DOC.contains(&format!(
        "service_api_request_path_authz_missing_header_reason_code={SERVICE_API_REQUEST_PATH_AUTHZ_MISSING_HEADER_REASON_CODE}"
    )));
    assert!(DOC.contains("service_api_request_path_authz_ops_doc_path=docs/ops/configuration.md"));
    assert!(DOC.contains("service_api_request_path_authz_strategy_doc_path=docs/ci/strategy.md"));
    assert!(DOC.contains("service_api_request_path_authz_remediation_map_version=v1"));
}

fn assert_request_path_authz_doc_commands() {
    assert!(DOC.contains(
        "cargo test -p kamn-node main_tests::service_api_endpoint_tests::unit_service_api_route_authz_matrix_matches_protected_and_public_paths -- --exact"
    ));
    assert!(DOC.contains(
        "cargo test -p kamn-node main_tests::service_api_endpoint_tests::integration_service_api_endpoint_route_authz_matrix_rejects_protected_paths_without_headers -- --exact"
    ));
    assert!(DOC.contains(
        "cargo test -p kamn-core --test ci_strategy_docs doc_enforces_service_api_request_path_authz_docs_parity_matches_source_taxonomy -- --exact"
    ));
    assert!(DOC.contains(
        "cargo test -p kamn-core --test ci_strategy_docs doc_enforces_service_api_request_path_authz_remediation_markers_cover_reason_codes -- --exact"
    ));
}

fn assert_request_path_authz_source_markers() {
    assert!(SERVICE_API_ENDPOINT_SOURCE
        .contains("pub(crate) const SERVICE_API_AUTH_REASON_TAXONOMY_VERSION: &str ="));
    assert!(SERVICE_API_ENDPOINT_SOURCE
        .contains("pub(crate) const SERVICE_API_AUTH_REASON_CODES_CSV: &str ="));
    assert!(SERVICE_API_ENDPOINT_SOURCE
        .contains(SERVICE_API_REQUEST_PATH_AUTHZ_REASON_TAXONOMY_VERSION));
    assert!(SERVICE_API_ENDPOINT_SOURCE.contains(SERVICE_API_REQUEST_PATH_AUTHZ_REASON_CODES_CSV));
}

fn assert_request_path_authz_strategy_markers() {
    assert!(DOC.contains(&format!(
        "service_api_request_path_authz_reason_taxonomy_version={SERVICE_API_REQUEST_PATH_AUTHZ_REASON_TAXONOMY_VERSION}"
    )));
    assert!(DOC.contains(&format!(
        "service_api_request_path_authz_reason_codes_csv={SERVICE_API_REQUEST_PATH_AUTHZ_REASON_CODES_CSV}"
    )));
    assert!(DOC.contains(&format!(
        "service_api_request_path_authz_public_routes_csv={SERVICE_API_REQUEST_PATH_AUTHZ_PUBLIC_ROUTES_CSV}"
    )));
    assert!(DOC.contains(&format!(
        "service_api_request_path_authz_protected_routes_csv={SERVICE_API_REQUEST_PATH_AUTHZ_PROTECTED_ROUTES_CSV}"
    )));
    assert!(DOC.contains(&format!(
        "service_api_request_path_authz_missing_header_reason_code={SERVICE_API_REQUEST_PATH_AUTHZ_MISSING_HEADER_REASON_CODE}"
    )));
}

fn assert_request_path_authz_ops_markers() {
    assert!(OPS_DOC.contains(&format!(
        "service_api_request_path_authz_reason_taxonomy_version={SERVICE_API_REQUEST_PATH_AUTHZ_REASON_TAXONOMY_VERSION}"
    )));
    assert!(OPS_DOC.contains(&format!(
        "service_api_request_path_authz_reason_codes_csv={SERVICE_API_REQUEST_PATH_AUTHZ_REASON_CODES_CSV}"
    )));
    assert!(OPS_DOC.contains(&format!(
        "service_api_request_path_authz_public_routes_csv={SERVICE_API_REQUEST_PATH_AUTHZ_PUBLIC_ROUTES_CSV}"
    )));
    assert!(OPS_DOC.contains(&format!(
        "service_api_request_path_authz_protected_routes_csv={SERVICE_API_REQUEST_PATH_AUTHZ_PROTECTED_ROUTES_CSV}"
    )));
    assert!(OPS_DOC.contains(&format!(
        "service_api_request_path_authz_missing_header_reason_code={SERVICE_API_REQUEST_PATH_AUTHZ_MISSING_HEADER_REASON_CODE}"
    )));
}

fn assert_request_path_authz_remediation_markers(reason_code: &str) {
    assert_remediation_marker_in_docs_and_ops(
        "service_api_request_path_authz_remediation",
        reason_code,
        "request-path authz",
    );
}
