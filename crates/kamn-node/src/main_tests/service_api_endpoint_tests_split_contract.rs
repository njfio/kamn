use std::fs;

const ROOT_FILE: &str = "src/main_tests/service_api_endpoint_tests.rs";
const WEBSOCKET_FILE: &str =
    "src/main_tests/service_api_endpoint_tests/websocket_contract_tests.rs";
const AUTH_SCOPE_MODULE_FILE: &str =
    "src/main_tests/service_api_endpoint_tests/auth_scope_contract_tests.rs";
const AUTH_BINDING_FILE: &str =
    "src/main_tests/service_api_endpoint_tests/auth_scope_contract_tests/auth_binding_contract_tests.rs";
const ROUTE_SCOPE_POLICY_FILE: &str =
    "src/main_tests/service_api_endpoint_tests/auth_scope_contract_tests/route_scope_policy_contract_tests.rs";
const LEGACY_SIGNATURE_FILE: &str =
    "src/main_tests/service_api_endpoint_tests/auth_scope_contract_tests/legacy_signature_contract_tests.rs";
const ROUTE_RENDER_MODULE_FILE: &str =
    "src/main_tests/service_api_endpoint_tests/route_render_contract_tests.rs";
const ROUTE_RESPONSE_FILE: &str =
    "src/main_tests/service_api_endpoint_tests/route_render_contract_tests/route_response_contract_tests.rs";
const ROUTE_METRICS_FILE: &str =
    "src/main_tests/service_api_endpoint_tests/route_render_contract_tests/route_metrics_contract_tests.rs";
const ROOT_STAGED_MAX_LINES: usize = 7800;

fn read_repo_file(path: &str) -> String {
    let root = env!("CARGO_MANIFEST_DIR");
    let full_path = format!("{root}/{path}");
    fs::read_to_string(&full_path).unwrap_or_else(|error| {
        panic!("failed to read {path}: {error}");
    })
}

#[test]
fn spec_c01_service_api_endpoint_root_file_removes_websocket_helpers_and_tests() {
    let source = read_repo_file(ROOT_FILE);
    for marker in [
        "fn send_websocket_upgrade_request(addr: &str, path: &str, headers: &[(&str, &str)]) -> Vec<u8>",
        "fn send_websocket_upgrade_request_with_version(",
        "fn send_websocket_upgrade_request_with_version_close_observation(",
        "fn parse_websocket_response_frames(response: &[u8]) -> (String, Vec<String>)",
        "fn parse_websocket_response(response: &[u8]) -> (String, String)",
        "fn integration_service_api_endpoint_websocket_upgrade_streams_state_transition_event()",
        "fn integration_service_api_endpoint_websocket_upgrade_keeps_connection_open_after_initial_event()",
        "fn regression_service_api_endpoint_websocket_stream_delivers_live_message_event_after_upgrade()",
        "fn integration_service_api_endpoint_websocket_presence_mode_streams_bridge_projection_event()",
        "fn regression_service_api_endpoint_websocket_presence_mode_rejects_unsupported_mode()",
        "fn regression_service_api_endpoint_websocket_presence_mode_rejects_missing_owner_header()",
        "fn regression_service_api_endpoint_websocket_presence_mode_rejects_cross_owner_scope()",
        "fn regression_service_api_endpoint_websocket_route_rejects_missing_upgrade_headers()",
        "fn regression_service_api_endpoint_websocket_rejects_invalid_version_header()",
    ] {
        assert!(
            !source.contains(marker),
            "service_api_endpoint_tests.rs should not keep moved websocket marker: {marker}"
        );
    }
}

#[test]
fn spec_c02_service_api_endpoint_websocket_module_exists_and_owns_moved_coverage() {
    let websocket = read_repo_file(WEBSOCKET_FILE);
    for marker in [
        "fn send_websocket_upgrade_request(addr: &str, path: &str, headers: &[(&str, &str)]) -> Vec<u8>",
        "fn send_websocket_upgrade_request_with_version(",
        "fn send_websocket_upgrade_request_with_version_close_observation(",
        "fn parse_websocket_response_frames(response: &[u8]) -> (String, Vec<String>)",
        "fn parse_websocket_response(response: &[u8]) -> (String, String)",
        "fn integration_service_api_endpoint_websocket_upgrade_streams_state_transition_event()",
        "fn integration_service_api_endpoint_websocket_upgrade_keeps_connection_open_after_initial_event()",
        "fn regression_service_api_endpoint_websocket_stream_delivers_live_message_event_after_upgrade()",
        "fn integration_service_api_endpoint_websocket_presence_mode_streams_bridge_projection_event()",
        "fn regression_service_api_endpoint_websocket_presence_mode_rejects_unsupported_mode()",
        "fn regression_service_api_endpoint_websocket_presence_mode_rejects_missing_owner_header()",
        "fn regression_service_api_endpoint_websocket_presence_mode_rejects_cross_owner_scope()",
        "fn regression_service_api_endpoint_websocket_route_rejects_missing_upgrade_headers()",
        "fn regression_service_api_endpoint_websocket_rejects_invalid_version_header()",
    ] {
        assert!(
            websocket.contains(marker),
            "websocket_contract_tests.rs should include moved marker: {marker}"
        );
    }
}

#[test]
fn spec_c03_service_api_endpoint_root_declares_websocket_submodule() {
    let source = read_repo_file(ROOT_FILE);
    assert!(
        source.contains("mod websocket_contract_tests;"),
        "service_api_endpoint_tests.rs should declare websocket submodule"
    );
}

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

    assert!(
        auth_scope_module.contains("mod auth_binding_contract_tests;"),
        "auth_scope_contract_tests.rs should declare auth-binding submodule"
    );
    assert!(
        auth_scope_module.contains("mod route_scope_policy_contract_tests;"),
        "auth_scope_contract_tests.rs should declare route/scope-policy submodule"
    );
    assert!(
        auth_scope_module.contains("mod legacy_signature_contract_tests;"),
        "auth_scope_contract_tests.rs should declare legacy-signature submodule"
    );

    for marker in [
        "fn integration_service_api_endpoint_accepts_case_variant_self_certifying_sender_did_binding()",
        "fn regression_service_api_endpoint_rejects_legacy_sender_binding_without_signer_public_key_header()",
    ] {
        assert!(
            auth_binding.contains(marker),
            "auth-binding contract file should include moved marker: {marker}"
        );
    }

    for marker in [
        "fn unit_service_api_route_authz_matrix_matches_protected_and_public_paths()",
        "fn integration_service_api_endpoint_route_authz_matrix_rejects_protected_paths_without_headers()",
        "fn unit_service_api_scope_policy_fixture_parser_contract()",
        "fn functional_service_api_scope_policy_fixture_rows_match_route_scope_mapping()",
        "fn integration_service_api_endpoint_scope_policy_rejects_missing_invalid_and_mismatched_scopes()",
        "fn integration_service_api_endpoint_rejects_missing_request_auth_headers()",
    ] {
        assert!(
            route_scope_policy.contains(marker),
            "route/scope-policy contract file should include moved marker: {marker}"
        );
    }

    for marker in [
        "fn integration_service_api_endpoint_rejects_legacy_deterministic_signature_profile()",
        "fn regression_service_api_endpoint_rejects_legacy_signature_when_toggle_env_is_true()",
    ] {
        assert!(
            legacy_signature.contains(marker),
            "legacy-signature contract file should include moved marker: {marker}"
        );
    }
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

    assert!(
        module_source.contains("mod route_response_contract_tests;"),
        "route_render_contract_tests.rs should declare route-response submodule"
    );
    assert!(
        module_source.contains("mod route_metrics_contract_tests;"),
        "route_render_contract_tests.rs should declare route-metrics submodule"
    );

    for marker in [
        "fn functional_service_api_endpoint_renders_required_route_contracts()",
        "let send_response = render_service_api_endpoint_response(",
        "let bridge_query_response =",
        "let health_response = render_service_api_endpoint_response(&snapshot, \"GET\", \"/healthz\", \"\");",
    ] {
        assert!(
            route_response.contains(marker),
            "route_response_contract_tests.rs should include moved marker: {marker}"
        );
    }

    for marker in [
        "kamn_service_api_route_authz_matrix_total_route_count {}",
        "kamn_service_api_scope_policy_fixture_unique_allow_route_count",
        "kamn_service_api_websocket_reason_taxonomy_info",
        "service_api_websocket_upgrade_required",
    ] {
        assert!(
            route_metrics.contains(marker),
            "route_metrics_contract_tests.rs should include moved marker: {marker}"
        );
    }
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
    for path in [ROUTE_RENDER_MODULE_FILE, ROUTE_RESPONSE_FILE, ROUTE_METRICS_FILE] {
        let source = read_repo_file(path);
        let line_count = source.lines().count();
        assert!(
            line_count <= 200,
            "{path} should stay below 200 lines after extraction: line_count={line_count}"
        );
    }
}
