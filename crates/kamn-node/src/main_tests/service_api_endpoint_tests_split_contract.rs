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
const MESSAGE_PERSISTENCE_MODULE_FILE: &str =
    "src/main_tests/service_api_endpoint_tests/message_persistence_contract_tests.rs";
const MESSAGE_RESTART_FILE: &str =
    "src/main_tests/service_api_endpoint_tests/message_persistence_contract_tests/message_restart_contract_tests.rs";
const MESSAGE_RUNTIME_EVIDENCE_FILE: &str =
    "src/main_tests/service_api_endpoint_tests/message_persistence_contract_tests/message_runtime_evidence_contract_tests.rs";
const CHANNEL_AGENT_DIRECTORY_MODULE_FILE: &str =
    "src/main_tests/service_api_endpoint_tests/channel_agent_directory_contract_tests.rs";
const CHANNEL_STATE_FILE: &str =
    "src/main_tests/service_api_endpoint_tests/channel_agent_directory_contract_tests/channel_state_contract_tests.rs";
const AGENT_PROFILE_FILE: &str =
    "src/main_tests/service_api_endpoint_tests/channel_agent_directory_contract_tests/agent_profile_contract_tests.rs";
const AGENT_REGISTRY_FILE: &str =
    "src/main_tests/service_api_endpoint_tests/channel_agent_directory_contract_tests/agent_registry_contract_tests.rs";
const TASK_ESCROW_PERSISTENCE_MODULE_FILE: &str =
    "src/main_tests/service_api_endpoint_tests/task_escrow_persistence_contract_tests.rs";
const TASK_ESCROW_ROUTES_FILE: &str =
    "src/main_tests/service_api_endpoint_tests/task_escrow_persistence_contract_tests/task_escrow_routes_contract_tests.rs";
const TASK_ESCROW_RESTART_FILE: &str =
    "src/main_tests/service_api_endpoint_tests/task_escrow_persistence_contract_tests/task_escrow_restart_contract_tests.rs";
const CONTENT_LIFECYCLE_RESTART_MODULE_FILE: &str =
    "src/main_tests/service_api_endpoint_tests/content_lifecycle_restart_contract_tests.rs";
const CONTENT_LIFECYCLE_RESTART_FILE: &str =
    "src/main_tests/service_api_endpoint_tests/content_lifecycle_restart_contract_tests/content_lifecycle_restart_contract_tests.rs";
const BRIDGE_PERSISTENCE_RESTART_MODULE_FILE: &str =
    "src/main_tests/service_api_endpoint_tests/bridge_persistence_restart_contract_tests.rs";
const BRIDGE_PERSISTENCE_RESTART_FILE: &str =
    "src/main_tests/service_api_endpoint_tests/bridge_persistence_restart_contract_tests/bridge_persistence_restart_contract_tests.rs";
const MAILBOX_RELAY_DELIVERY_MODULE_FILE: &str =
    "src/main_tests/service_api_endpoint_tests/mailbox_relay_delivery_contract_tests.rs";
const RECIPIENT_MAILBOX_FILE: &str =
    "src/main_tests/service_api_endpoint_tests/mailbox_relay_delivery_contract_tests/recipient_mailbox_contract_tests.rs";
const RELAY_DELIVERY_FILE: &str =
    "src/main_tests/service_api_endpoint_tests/mailbox_relay_delivery_contract_tests/relay_delivery_contract_tests.rs";
const RELAY_DID_REJECTION_FILE: &str =
    "src/main_tests/service_api_endpoint_tests/mailbox_relay_delivery_contract_tests/relay_did_rejection_contract_tests.rs";
const RELAY_STATUS_FILE: &str =
    "src/main_tests/service_api_endpoint_tests/mailbox_relay_delivery_contract_tests/relay_status_contract_tests.rs";
const MAILBOX_RELAY_SUPPORT_FILE: &str =
    "src/main_tests/service_api_endpoint_tests/mailbox_relay_delivery_contract_tests/support.rs";
const MAILBOX_RELAY_STATE_SUPPORT_FILE: &str =
    "src/main_tests/service_api_endpoint_tests/mailbox_relay_delivery_contract_tests/state_support.rs";
const INGRESS_GUARD_LIFECYCLE_MODULE_FILE: &str =
    "src/main_tests/service_api_endpoint_tests/ingress_guard_lifecycle_contract_tests.rs";
const INGRESS_BUDGET_FILE: &str =
    "src/main_tests/service_api_endpoint_tests/ingress_guard_lifecycle_contract_tests/ingress_budget_contract_tests.rs";
const SENDER_ANTI_SPAM_FILE: &str =
    "src/main_tests/service_api_endpoint_tests/ingress_guard_lifecycle_contract_tests/sender_anti_spam_contract_tests.rs";
const REPLAY_GUARD_FILE: &str =
    "src/main_tests/service_api_endpoint_tests/ingress_guard_lifecycle_contract_tests/replay_guard_contract_tests.rs";
const CONCURRENCY_GUARD_FILE: &str =
    "src/main_tests/service_api_endpoint_tests/ingress_guard_lifecycle_contract_tests/concurrency_guard_contract_tests.rs";
const LIFECYCLE_PROJECTION_FILE: &str =
    "src/main_tests/service_api_endpoint_tests/ingress_guard_lifecycle_contract_tests/lifecycle_projection_contract_tests.rs";
const ROOT_STAGED_MAX_LINES: usize = 2400;

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

#[test]
fn spec_c13_service_api_endpoint_root_file_removes_moved_message_persistence_contracts() {
    let source = read_repo_file(ROOT_FILE);
    for marker in [
        "fn integration_service_api_endpoint_persists_message_state_across_restart_without_explicit_state_file_env(",
        "fn integration_service_api_endpoint_persists_message_state_across_restart()",
        "fn integration_service_api_endpoint_send_path_persists_data_layer_runtime_evidence_for_m0_to_m11()",
        "let query_path = format!(\"/v1/messages/{}\", send_payload.message_id);",
        "data_layer_runtime_evidence",
    ] {
        assert!(
            !source.contains(marker),
            "service_api_endpoint_tests.rs should not keep moved message-persistence marker: {marker}"
        );
    }
}

#[test]
fn spec_c14_service_api_endpoint_message_persistence_module_exists_and_owns_moved_coverage() {
    let module_source = read_repo_file(MESSAGE_PERSISTENCE_MODULE_FILE);
    let restart_source = read_repo_file(MESSAGE_RESTART_FILE);
    let runtime_evidence_source = read_repo_file(MESSAGE_RUNTIME_EVIDENCE_FILE);

    assert!(
        module_source.contains("mod message_restart_contract_tests;"),
        "message_persistence_contract_tests.rs should declare restart submodule"
    );
    assert!(
        module_source.contains("mod message_runtime_evidence_contract_tests;"),
        "message_persistence_contract_tests.rs should declare runtime-evidence submodule"
    );

    for marker in [
        "fn integration_service_api_endpoint_persists_message_state_across_restart_without_explicit_state_file_env(",
        "fn integration_service_api_endpoint_persists_message_state_across_restart()",
        "let query_path = format!(\"/v1/messages/{}\", send_payload.message_id);",
        "let _ = fs::remove_file(state_file);",
    ] {
        assert!(
            restart_source.contains(marker),
            "message_restart_contract_tests.rs should include moved marker: {marker}"
        );
    }

    for marker in [
        "fn integration_service_api_endpoint_send_path_persists_data_layer_runtime_evidence_for_m0_to_m11()",
        "data_layer_runtime_evidence",
        "m11_decision",
    ] {
        assert!(
            runtime_evidence_source.contains(marker),
            "message_runtime_evidence_contract_tests.rs should include moved marker: {marker}"
        );
    }
}

#[test]
fn spec_c15_service_api_endpoint_root_declares_message_persistence_submodule() {
    let source = read_repo_file(ROOT_FILE);
    assert!(
        source.contains("mod message_persistence_contract_tests;"),
        "service_api_endpoint_tests.rs should declare message-persistence submodule"
    );
}

#[test]
fn spec_c16_service_api_endpoint_message_persistence_split_files_stay_below_budget() {
    for path in [
        MESSAGE_PERSISTENCE_MODULE_FILE,
        MESSAGE_RESTART_FILE,
        MESSAGE_RUNTIME_EVIDENCE_FILE,
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
fn spec_c17_service_api_endpoint_root_file_removes_moved_channel_agent_directory_contracts() {
    let source = read_repo_file(ROOT_FILE);
    for marker in [
        "fn integration_service_api_endpoint_lists_channel_messages_from_message_store()",
        "fn integration_service_api_endpoint_persists_channel_creation_state_across_restart()",
        "fn integration_service_api_endpoint_persists_agent_profile_query_state_across_restart()",
        "fn integration_service_api_endpoint_registers_agent_metadata_idempotently_and_conflicts_on_mismatch(",
        "fn integration_service_api_endpoint_searches_registered_agent_metadata()",
        "fn integration_service_api_endpoint_rejects_invalid_agent_search_payload()",
        "fn integration_service_api_endpoint_rejects_legacy_agent_profile_path_dids()",
    ] {
        assert!(
            !source.contains(marker),
            "service_api_endpoint_tests.rs should not keep moved channel/agent directory marker: {marker}"
        );
    }
}

#[test]
fn spec_c18_service_api_endpoint_channel_agent_directory_module_exists_and_owns_moved_coverage() {
    let module_source = read_repo_file(CHANNEL_AGENT_DIRECTORY_MODULE_FILE);
    let channel_state = read_repo_file(CHANNEL_STATE_FILE);
    let agent_profile = read_repo_file(AGENT_PROFILE_FILE);
    let agent_registry = read_repo_file(AGENT_REGISTRY_FILE);

    assert!(
        module_source.contains("mod channel_state_contract_tests;"),
        "channel_agent_directory_contract_tests.rs should declare channel-state submodule"
    );
    assert!(
        module_source.contains("mod agent_profile_contract_tests;"),
        "channel_agent_directory_contract_tests.rs should declare agent-profile submodule"
    );
    assert!(
        module_source.contains("mod agent_registry_contract_tests;"),
        "channel_agent_directory_contract_tests.rs should declare agent-registry submodule"
    );

    for marker in [
        "fn integration_service_api_endpoint_lists_channel_messages_from_message_store()",
        "fn integration_service_api_endpoint_persists_channel_creation_state_across_restart()",
    ] {
        assert!(
            channel_state.contains(marker),
            "channel_state_contract_tests.rs should include moved marker: {marker}"
        );
    }

    for marker in [
        "fn integration_service_api_endpoint_persists_agent_profile_query_state_across_restart()",
        "fn integration_service_api_endpoint_rejects_legacy_agent_profile_path_dids()",
    ] {
        assert!(
            agent_profile.contains(marker),
            "agent_profile_contract_tests.rs should include moved marker: {marker}"
        );
    }

    for marker in [
        "fn integration_service_api_endpoint_registers_agent_metadata_idempotently_and_conflicts_on_mismatch(",
        "fn integration_service_api_endpoint_searches_registered_agent_metadata()",
        "fn integration_service_api_endpoint_rejects_invalid_agent_search_payload()",
    ] {
        assert!(
            agent_registry.contains(marker),
            "agent_registry_contract_tests.rs should include moved marker: {marker}"
        );
    }
}

#[test]
fn spec_c19_service_api_endpoint_root_declares_channel_agent_directory_submodule() {
    let source = read_repo_file(ROOT_FILE);
    assert!(
        source.contains("mod channel_agent_directory_contract_tests;"),
        "service_api_endpoint_tests.rs should declare channel-agent-directory submodule"
    );
}

#[test]
fn spec_c20_service_api_endpoint_channel_agent_directory_split_files_stay_below_budget() {
    for path in [
        CHANNEL_AGENT_DIRECTORY_MODULE_FILE,
        CHANNEL_STATE_FILE,
        AGENT_PROFILE_FILE,
        AGENT_REGISTRY_FILE,
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
fn spec_c21_service_api_endpoint_root_file_removes_moved_task_escrow_persistence_contracts() {
    let source = read_repo_file(ROOT_FILE);
    for marker in [
        "fn integration_service_api_endpoint_persists_task_and_escrow_state_across_routes()",
        "fn integration_service_api_endpoint_persists_task_and_escrow_state_across_restart()",
    ] {
        assert!(
            !source.contains(marker),
            "service_api_endpoint_tests.rs should not keep moved task/escrow persistence marker: {marker}"
        );
    }
}

#[test]
fn spec_c22_service_api_endpoint_task_escrow_persistence_module_exists_and_owns_moved_coverage() {
    let module_source = read_repo_file(TASK_ESCROW_PERSISTENCE_MODULE_FILE);
    let routes_source = read_repo_file(TASK_ESCROW_ROUTES_FILE);
    let restart_source = read_repo_file(TASK_ESCROW_RESTART_FILE);

    assert!(
        module_source.contains("mod task_escrow_routes_contract_tests;"),
        "task_escrow_persistence_contract_tests.rs should declare routes submodule"
    );
    assert!(
        module_source.contains("mod task_escrow_restart_contract_tests;"),
        "task_escrow_persistence_contract_tests.rs should declare restart submodule"
    );

    assert!(
        routes_source.contains(
            "fn integration_service_api_endpoint_persists_task_and_escrow_state_across_routes()"
        ),
        "task_escrow_routes_contract_tests.rs should include moved routes marker"
    );
    assert!(
        restart_source.contains(
            "fn integration_service_api_endpoint_persists_task_and_escrow_state_across_restart()"
        ),
        "task_escrow_restart_contract_tests.rs should include moved restart marker"
    );
}

#[test]
fn spec_c23_service_api_endpoint_root_declares_task_escrow_persistence_submodule() {
    let source = read_repo_file(ROOT_FILE);
    assert!(
        source.contains("mod task_escrow_persistence_contract_tests;"),
        "service_api_endpoint_tests.rs should declare task-escrow-persistence submodule"
    );
}

#[test]
fn spec_c24_service_api_endpoint_task_escrow_persistence_split_files_stay_below_budget() {
    for path in [
        TASK_ESCROW_PERSISTENCE_MODULE_FILE,
        TASK_ESCROW_ROUTES_FILE,
        TASK_ESCROW_RESTART_FILE,
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
fn spec_c25_service_api_endpoint_root_file_removes_moved_content_lifecycle_restart_contract() {
    let source = read_repo_file(ROOT_FILE);
    let marker = "fn integration_service_api_endpoint_persists_content_lifecycle_state_across_restart()";
    assert!(
        !source.contains(marker),
        "service_api_endpoint_tests.rs should not keep moved content lifecycle restart marker: {marker}"
    );
}

#[test]
fn spec_c26_service_api_endpoint_content_lifecycle_restart_module_exists_and_owns_moved_coverage() {
    let module_source = read_repo_file(CONTENT_LIFECYCLE_RESTART_MODULE_FILE);
    let restart_source = read_repo_file(CONTENT_LIFECYCLE_RESTART_FILE);

    assert!(
        module_source.contains("mod content_lifecycle_restart_contract_tests;"),
        "content_lifecycle_restart_contract_tests.rs should declare restart submodule"
    );
    assert!(
        restart_source.contains(
            "fn integration_service_api_endpoint_persists_content_lifecycle_state_across_restart()"
        ),
        "content_lifecycle restart contract file should include moved restart marker"
    );
}

#[test]
fn spec_c27_service_api_endpoint_root_declares_content_lifecycle_restart_submodule() {
    let source = read_repo_file(ROOT_FILE);
    assert!(
        source.contains("mod content_lifecycle_restart_contract_tests;"),
        "service_api_endpoint_tests.rs should declare content-lifecycle-restart submodule"
    );
}

#[test]
fn spec_c28_service_api_endpoint_content_lifecycle_restart_split_files_stay_below_budget() {
    for path in [
        CONTENT_LIFECYCLE_RESTART_MODULE_FILE,
        CONTENT_LIFECYCLE_RESTART_FILE,
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
fn spec_c29_service_api_endpoint_root_file_removes_moved_bridge_persistence_restart_contract() {
    let source = read_repo_file(ROOT_FILE);
    let marker = "fn integration_service_api_endpoint_persists_bridge_state_across_restart()";
    assert!(
        !source.contains(marker),
        "service_api_endpoint_tests.rs should not keep moved bridge persistence restart marker: {marker}"
    );
}

#[test]
fn spec_c30_service_api_endpoint_bridge_persistence_restart_module_exists_and_owns_moved_coverage() {
    let module_source = read_repo_file(BRIDGE_PERSISTENCE_RESTART_MODULE_FILE);
    let restart_source = read_repo_file(BRIDGE_PERSISTENCE_RESTART_FILE);

    assert!(
        module_source.contains("mod bridge_persistence_restart_contract_tests;"),
        "bridge_persistence_restart_contract_tests.rs should declare restart submodule"
    );
    assert!(
        restart_source.contains(
            "fn integration_service_api_endpoint_persists_bridge_state_across_restart()"
        ),
        "bridge persistence restart contract file should include moved restart marker"
    );
}

#[test]
fn spec_c31_service_api_endpoint_root_declares_bridge_persistence_restart_submodule() {
    let source = read_repo_file(ROOT_FILE);
    assert!(
        source.contains("mod bridge_persistence_restart_contract_tests;"),
        "service_api_endpoint_tests.rs should declare bridge-persistence-restart submodule"
    );
}

#[test]
fn spec_c32_service_api_endpoint_bridge_persistence_restart_split_files_stay_below_budget() {
    for path in [
        BRIDGE_PERSISTENCE_RESTART_MODULE_FILE,
        BRIDGE_PERSISTENCE_RESTART_FILE,
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
fn spec_c33_service_api_endpoint_root_file_removes_moved_mailbox_relay_delivery_contracts() {
    let source = read_repo_file(ROOT_FILE);
    for marker in [
        "fn integration_service_api_endpoint_recipient_mailbox_and_delivery_status_contract()",
        "fn integration_service_api_endpoint_rejects_legacy_message_send_recipient_dids()",
        "fn integration_service_api_endpoint_cross_node_relay_delivery_contract()",
        "fn integration_service_api_endpoint_rejects_legacy_relay_ingest_dids()",
        "fn regression_service_api_endpoint_recipient_query_requires_relayed_state_before_delivery()",
        "fn integration_service_api_endpoint_recipient_query_promotes_relayed_to_delivered()",
        "fn regression_service_api_endpoint_non_recipient_query_keeps_relayed_status_across_restart()",
        "fn integration_service_api_endpoint_enqueues_recipient_relays_to_durable_spool()",
    ] {
        assert!(
            !source.contains(marker),
            "service_api_endpoint_tests.rs should not keep moved mailbox/relay marker: {marker}"
        );
    }
}

#[test]
fn spec_c34_service_api_endpoint_mailbox_relay_delivery_module_exists_and_owns_moved_coverage() {
    let module_source = read_repo_file(MAILBOX_RELAY_DELIVERY_MODULE_FILE);
    let recipient_mailbox = read_repo_file(RECIPIENT_MAILBOX_FILE);
    let relay_delivery = read_repo_file(RELAY_DELIVERY_FILE);
    let relay_did_rejection = read_repo_file(RELAY_DID_REJECTION_FILE);
    let relay_status = read_repo_file(RELAY_STATUS_FILE);

    assert_mailbox_relay_module_declarations(module_source.as_str());
    assert_mailbox_relay_markers(
        recipient_mailbox.as_str(),
        relay_delivery.as_str(),
        relay_did_rejection.as_str(),
        relay_status.as_str(),
    );
}

fn assert_mailbox_relay_module_declarations(module_source: &str) {
    for marker in [
        "mod recipient_mailbox_contract_tests;",
        "mod relay_delivery_contract_tests;",
        "mod relay_did_rejection_contract_tests;",
        "mod relay_status_contract_tests;",
        "mod support;",
        "mod state_support;",
    ] {
        assert!(
            module_source.contains(marker),
            "mailbox_relay_delivery_contract_tests.rs should declare submodule marker: {marker}"
        );
    }
}

fn assert_mailbox_relay_markers(
    recipient_mailbox: &str,
    relay_delivery: &str,
    relay_did_rejection: &str,
    relay_status: &str,
) {
    assert_mailbox_relay_delivery_markers(
        recipient_mailbox,
        relay_delivery,
        relay_did_rejection,
    );
    assert_mailbox_relay_status_markers(relay_status);
}

fn assert_mailbox_relay_delivery_markers(
    recipient_mailbox: &str,
    relay_delivery: &str,
    relay_did_rejection: &str,
) {
    assert_recipient_mailbox_markers(recipient_mailbox);
    assert_relay_delivery_markers(relay_delivery);
    assert_relay_did_rejection_markers(relay_did_rejection);
}

fn assert_recipient_mailbox_markers(source: &str) {
    assert_mailbox_relay_file_markers(
        source,
        &["fn integration_service_api_endpoint_recipient_mailbox_and_delivery_status_contract()"],
        "recipient mailbox contract file",
    );
}

fn assert_relay_delivery_markers(source: &str) {
    assert_mailbox_relay_file_markers(
        source,
        &[
            "fn integration_service_api_endpoint_cross_node_relay_delivery_contract()",
            "fn integration_service_api_endpoint_enqueues_recipient_relays_to_durable_spool()",
        ],
        "relay delivery contract file",
    );
}

fn assert_relay_did_rejection_markers(source: &str) {
    assert_mailbox_relay_file_markers(
        source,
        &[
            "fn integration_service_api_endpoint_rejects_legacy_message_send_recipient_dids()",
            "fn integration_service_api_endpoint_rejects_legacy_relay_ingest_dids()",
        ],
        "relay did rejection contract file",
    );
}

fn assert_mailbox_relay_status_markers(relay_status: &str) {
    assert_mailbox_relay_file_markers(
        relay_status,
        &[
            "fn regression_service_api_endpoint_recipient_query_requires_relayed_state_before_delivery()",
            "fn integration_service_api_endpoint_recipient_query_promotes_relayed_to_delivered()",
            "fn regression_service_api_endpoint_non_recipient_query_keeps_relayed_status_across_restart()",
        ],
        "relay status contract file",
    );
}

fn assert_mailbox_relay_file_markers(source: &str, markers: &[&str], label: &str) {
    for marker in markers {
        assert!(
            source.contains(marker),
            "{label} should include moved marker: {marker}"
        );
    }
}

#[test]
fn spec_c35_service_api_endpoint_root_declares_mailbox_relay_delivery_submodule() {
    let source = read_repo_file(ROOT_FILE);
    assert!(
        source.contains("mod mailbox_relay_delivery_contract_tests;"),
        "service_api_endpoint_tests.rs should declare mailbox-relay-delivery submodule"
    );
}

#[test]
fn spec_c36_service_api_endpoint_mailbox_relay_delivery_split_files_stay_below_budget() {
    for path in [
        MAILBOX_RELAY_DELIVERY_MODULE_FILE,
        RECIPIENT_MAILBOX_FILE,
        RELAY_DELIVERY_FILE,
        RELAY_DID_REJECTION_FILE,
        RELAY_STATUS_FILE,
        MAILBOX_RELAY_SUPPORT_FILE,
        MAILBOX_RELAY_STATE_SUPPORT_FILE,
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
fn spec_c37_service_api_endpoint_root_file_removes_moved_ingress_guard_contracts() {
    let source = read_repo_file(ROOT_FILE);
    for marker in [
        "fn functional_service_api_endpoint_rejects_when_rate_limit_is_exceeded()",
        "fn functional_service_api_endpoint_applies_sender_anti_spam_throttle_and_suspension()",
        "fn integration_service_api_endpoint_sender_anti_spam_burst_rounds_remain_deterministic()",
        "fn integration_service_api_endpoint_rejects_when_concurrency_limit_is_exceeded()",
        "fn regression_service_api_endpoint_oversized_payload_maps_body_limit_reason_code()",
        "fn regression_service_api_endpoint_rejects_replayed_request_nonce_for_sender()",
        "fn integration_service_api_endpoint_lifecycle_projection_matches_live_concurrency_rejection()",
        "fn regression_service_api_endpoint_returns_timeout_error_when_no_requests_arrive()",
    ] {
        assert!(
            !source.contains(marker),
            "service_api_endpoint_tests.rs should not keep moved ingress-guard marker: {marker}"
        );
    }
}

#[test]
fn spec_c38_service_api_endpoint_ingress_guard_module_exists_and_owns_moved_coverage() {
    let module_source = read_repo_file(INGRESS_GUARD_LIFECYCLE_MODULE_FILE);
    let ingress_budget = read_repo_file(INGRESS_BUDGET_FILE);
    let sender_anti_spam = read_repo_file(SENDER_ANTI_SPAM_FILE);
    let replay_guard = read_repo_file(REPLAY_GUARD_FILE);
    let concurrency_guard = read_repo_file(CONCURRENCY_GUARD_FILE);
    let lifecycle_projection = read_repo_file(LIFECYCLE_PROJECTION_FILE);

    assert_ingress_guard_module_declarations(module_source.as_str());
    assert_ingress_guard_markers(
        ingress_budget.as_str(),
        sender_anti_spam.as_str(),
        replay_guard.as_str(),
        concurrency_guard.as_str(),
        lifecycle_projection.as_str(),
    );
}

fn assert_ingress_guard_module_declarations(module_source: &str) {
    for marker in [
        "mod ingress_budget_contract_tests;",
        "mod sender_anti_spam_contract_tests;",
        "mod replay_guard_contract_tests;",
        "mod concurrency_guard_contract_tests;",
        "mod lifecycle_projection_contract_tests;",
    ] {
        assert!(
            module_source.contains(marker),
            "ingress_guard_lifecycle_contract_tests.rs should declare submodule marker: {marker}"
        );
    }
}

fn assert_ingress_guard_markers(
    ingress_budget: &str,
    sender_anti_spam: &str,
    replay_guard: &str,
    concurrency_guard: &str,
    lifecycle_projection: &str,
) {
    assert_ingress_guard_file_markers(
        ingress_budget,
        &[
            "fn functional_service_api_endpoint_rejects_when_rate_limit_is_exceeded()",
            "fn regression_service_api_endpoint_oversized_payload_maps_body_limit_reason_code()",
            "fn regression_service_api_endpoint_unauthorized_ingress_consumes_request_budget()",
            "fn regression_service_api_endpoint_returns_timeout_error_when_no_requests_arrive()",
        ],
        "ingress budget contract file",
    );
    assert_ingress_guard_file_markers(
        sender_anti_spam,
        &[
            "fn functional_service_api_endpoint_applies_sender_anti_spam_throttle_and_suspension()",
            "fn integration_service_api_endpoint_sender_anti_spam_burst_rounds_remain_deterministic()",
        ],
        "sender anti-spam contract file",
    );
    assert_ingress_guard_file_markers(
        replay_guard,
        &[
            "fn regression_service_api_endpoint_rejects_replayed_request_nonce_for_sender()",
            "fn integration_service_api_endpoint_replay_rejection_remains_stable_with_anti_spam_enforcement()",
            "fn regression_service_api_endpoint_replay_duplicate_sequence_reason_ordering_stays_stable()",
        ],
        "replay guard contract file",
    );
    assert_ingress_guard_file_markers(
        concurrency_guard,
        &[
            "fn integration_service_api_endpoint_rejects_when_concurrency_limit_is_exceeded()",
            "fn integration_service_api_endpoint_concurrency_rejection_reason_stays_stable_under_bounded_bursts()",
            "fn regression_service_api_endpoint_concurrency_limit_reason_code_stays_stable_across_rounds()",
        ],
        "concurrency guard contract file",
    );
    assert_ingress_guard_file_markers(
        lifecycle_projection,
        &[
            "fn unit_service_api_endpoint_lifecycle_rejection_projection_is_deterministic()",
            "fn functional_service_api_endpoint_lifecycle_rejection_projection_maps_limiter_classes()",
            "fn functional_service_api_endpoint_backpressure_projection_covers_reason_codes()",
            "fn integration_service_api_endpoint_lifecycle_projection_matches_live_concurrency_rejection()",
            "fn regression_service_api_endpoint_lifecycle_projection_sender_suspension_class_stays_stable()",
            "fn performance_service_api_endpoint_lifecycle_projection_loop_stays_within_local_budget()",
        ],
        "lifecycle projection contract file",
    );
}

fn assert_ingress_guard_file_markers(source: &str, markers: &[&str], label: &str) {
    for marker in markers {
        assert!(
            source.contains(marker),
            "{label} should include moved marker: {marker}"
        );
    }
}

#[test]
fn spec_c39_service_api_endpoint_root_declares_ingress_guard_submodule() {
    let source = read_repo_file(ROOT_FILE);
    assert!(
        source.contains("mod ingress_guard_lifecycle_contract_tests;"),
        "service_api_endpoint_tests.rs should declare ingress-guard-lifecycle submodule"
    );
}

#[test]
fn spec_c40_service_api_endpoint_ingress_guard_split_files_stay_below_budget() {
    for path in [
        INGRESS_GUARD_LIFECYCLE_MODULE_FILE,
        INGRESS_BUDGET_FILE,
        SENDER_ANTI_SPAM_FILE,
        REPLAY_GUARD_FILE,
        CONCURRENCY_GUARD_FILE,
        LIFECYCLE_PROJECTION_FILE,
    ] {
        let source = read_repo_file(path);
        let line_count = source.lines().count();
        assert!(
            line_count <= 200,
            "{path} should stay below 200 lines after extraction: line_count={line_count}"
        );
    }
}
