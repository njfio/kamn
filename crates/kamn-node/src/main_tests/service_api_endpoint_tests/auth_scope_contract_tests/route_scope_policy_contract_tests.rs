const MOVED_ROUTE_SCOPE_POLICY_MARKERS: &[&str] = &[
    "fn unit_service_api_route_authz_matrix_matches_protected_and_public_paths()",
    "fn integration_service_api_endpoint_route_authz_matrix_rejects_protected_paths_without_headers()",
    "fn unit_service_api_scope_policy_fixture_parser_contract()",
    "fn functional_service_api_scope_policy_fixture_rows_match_route_scope_mapping()",
    "fn integration_service_api_endpoint_scope_policy_rejects_missing_invalid_and_mismatched_scopes()",
    "fn integration_service_api_endpoint_rejects_missing_request_auth_headers()",
];

#[allow(dead_code)]
fn moved_route_scope_policy_markers() -> &'static [&'static str] {
    MOVED_ROUTE_SCOPE_POLICY_MARKERS
}

#[path = "route_scope_policy_contract_tests/route_authz_contract_tests.rs"]
mod route_authz_contract_tests;
#[path = "route_scope_policy_contract_tests/scope_policy_fixture_contract_tests.rs"]
mod scope_policy_fixture_contract_tests;
#[path = "route_scope_policy_contract_tests/scope_policy_rejection_contract_tests.rs"]
mod scope_policy_rejection_contract_tests;
