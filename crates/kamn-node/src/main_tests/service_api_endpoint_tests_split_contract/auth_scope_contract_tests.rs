use super::support::*;

const AUTH_SCOPE_SUBMODULE_MARKERS: &[&str] = &[
    "mod auth_binding_contract_tests;",
    "mod route_scope_policy_contract_tests;",
    "mod legacy_signature_contract_tests;",
];
const AUTH_BINDING_MARKERS: &[&str] = &[
    "fn integration_service_api_endpoint_accepts_case_variant_self_certifying_sender_did_binding()",
    "fn regression_service_api_endpoint_rejects_legacy_sender_binding_without_signer_public_key_header()",
];
const ROUTE_SCOPE_POLICY_MARKERS: &[&str] = &[
    "fn unit_service_api_route_authz_matrix_matches_protected_and_public_paths()",
    "fn integration_service_api_endpoint_route_authz_matrix_rejects_protected_paths_without_headers()",
    "fn unit_service_api_scope_policy_fixture_parser_contract()",
    "fn functional_service_api_scope_policy_fixture_rows_match_route_scope_mapping()",
    "fn integration_service_api_endpoint_scope_policy_rejects_missing_invalid_and_mismatched_scopes()",
    "fn integration_service_api_endpoint_rejects_missing_request_auth_headers()",
];
const LEGACY_SIGNATURE_MARKERS: &[&str] = &[
    "fn integration_service_api_endpoint_rejects_legacy_deterministic_signature_profile()",
    "fn regression_service_api_endpoint_rejects_legacy_signature_when_toggle_env_is_true()",
];

#[test]
fn spec_c04_service_api_endpoint_root_file_removes_moved_auth_scope_tests() {
    let source = read_repo_file(ROOT_FILE);
    for marker in [
        "fn integration_service_api_endpoint_accepts_case_variant_self_certifying_sender_did_binding()",
        "fn regression_service_api_endpoint_rejects_legacy_sender_binding_without_signer_public_key_header()",
        "fn unit_service_api_route_authz_matrix_matches_protected_and_public_paths()",
        "fn integration_service_api_endpoint_route_authz_matrix_rejects_protected_paths_without_headers()",
        "fn unit_service_api_scope_policy_fixture_parser_contract()",
        "fn functional_service_api_scope_policy_fixture_rows_match_route_scope_mapping()",
        "fn integration_service_api_endpoint_scope_policy_rejects_missing_invalid_and_mismatched_scopes()",
        "fn integration_service_api_endpoint_rejects_missing_request_auth_headers()",
        "fn integration_service_api_endpoint_rejects_legacy_deterministic_signature_profile()",
        "fn regression_service_api_endpoint_rejects_legacy_signature_when_toggle_env_is_true()",
    ] {
        assert!(
            !source.contains(marker),
            "service_api_endpoint_tests.rs should not keep moved auth/scope marker: {marker}"
        );
    }
}

#[test]
fn spec_c05_service_api_endpoint_auth_scope_module_exists_and_owns_moved_coverage() {
    let auth_scope_module = read_repo_file(AUTH_SCOPE_MODULE_FILE);
    let auth_binding = read_repo_file(AUTH_BINDING_FILE);
    let route_scope_policy = read_repo_file(ROUTE_SCOPE_POLICY_FILE);
    let legacy_signature = read_repo_file(LEGACY_SIGNATURE_FILE);

    assert_contains_markers(
        auth_scope_module.as_str(),
        AUTH_SCOPE_SUBMODULE_MARKERS,
        "auth-scope module",
    );
    assert_contains_markers(
        auth_binding.as_str(),
        AUTH_BINDING_MARKERS,
        "auth-binding contract file",
    );
    assert_contains_markers(
        route_scope_policy.as_str(),
        ROUTE_SCOPE_POLICY_MARKERS,
        "route/scope-policy contract file",
    );
    assert_contains_markers(
        legacy_signature.as_str(),
        LEGACY_SIGNATURE_MARKERS,
        "legacy-signature contract file",
    );
}

#[test]
fn spec_c06_service_api_endpoint_root_declares_auth_scope_submodule() {
    let source = read_repo_file(ROOT_FILE);
    assert!(
        source.contains("mod auth_scope_contract_tests;"),
        "service_api_endpoint_tests.rs should declare auth/scope submodule"
    );
}

#[test]
fn spec_c07_service_api_endpoint_root_file_is_below_staged_threshold_after_auth_scope_split() {
    let source = read_repo_file(ROOT_FILE);
    let line_count = source.lines().count();
    assert!(
        line_count <= ROOT_STAGED_MAX_LINES,
        "service_api_endpoint_tests.rs staged threshold exceeded: line_count={line_count} max={ROOT_STAGED_MAX_LINES}"
    );
}

#[test]
fn spec_c08_service_api_endpoint_auth_scope_split_files_stay_below_budget() {
    for path in [
        AUTH_SCOPE_MODULE_FILE,
        AUTH_BINDING_FILE,
        ROUTE_SCOPE_POLICY_FILE,
        LEGACY_SIGNATURE_FILE,
    ] {
        let source = read_repo_file(path);
        let line_count = source.lines().count();
        assert!(
            line_count <= 200,
            "{path} should stay below 200 lines after extraction: line_count={line_count}"
        );
    }
}
