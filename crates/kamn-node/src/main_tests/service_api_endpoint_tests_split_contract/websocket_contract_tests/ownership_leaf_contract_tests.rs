use super::super::support::*;

#[test]
fn spec_c02_service_api_endpoint_websocket_module_exists_and_owns_leaf_coverage() {
    assert_upgrade_leaves();
    assert_presence_projection_leaf();
    assert_presence_validation_leaves();
    assert_presence_legacy_leaves();
    assert_rejection_leaves();
}

fn assert_upgrade_leaves() {
    assert_single_leaf(
        "src/main_tests/service_api_endpoint_tests/websocket_contract_tests/upgrade_delivery_contract_tests/reason_taxonomy_contract_tests.rs",
        &["fn regression_service_api_endpoint_websocket_reason_taxonomy_includes_presence_did_invalid_headers()"],
        "websocket reason taxonomy contract file",
    );
    assert_single_leaf(
        "src/main_tests/service_api_endpoint_tests/websocket_contract_tests/upgrade_delivery_contract_tests/upgrade_flow_contract_tests.rs",
        &[
            "fn integration_service_api_endpoint_websocket_upgrade_streams_state_transition_event()",
            "fn integration_service_api_endpoint_websocket_upgrade_keeps_connection_open_after_initial_event()",
        ],
        "websocket upgrade flow contract file",
    );
    assert_single_leaf(
        "src/main_tests/service_api_endpoint_tests/websocket_contract_tests/upgrade_delivery_contract_tests/live_delivery_contract_tests.rs",
        &["fn regression_service_api_endpoint_websocket_stream_delivers_live_message_event_after_upgrade()"],
        "websocket live delivery contract file",
    );
}

fn assert_presence_projection_leaf() {
    assert_single_leaf(
        "src/main_tests/service_api_endpoint_tests/websocket_contract_tests/presence_projection_contract_tests.rs",
        &["fn integration_service_api_endpoint_websocket_presence_mode_streams_bridge_projection_event()"],
        "websocket presence projection contract file",
    );
}

fn assert_presence_validation_leaves() {
    assert_single_leaf(
        "src/main_tests/service_api_endpoint_tests/websocket_contract_tests/presence_validation_contract_tests/unsupported_mode_contract_tests.rs",
        &["fn regression_service_api_endpoint_websocket_presence_mode_rejects_unsupported_mode()"],
        "websocket unsupported-mode contract file",
    );
    assert_single_leaf(
        "src/main_tests/service_api_endpoint_tests/websocket_contract_tests/presence_validation_contract_tests/missing_owner_contract_tests.rs",
        &["fn regression_service_api_endpoint_websocket_presence_mode_rejects_missing_owner_header()"],
        "websocket missing-owner contract file",
    );
    assert_single_leaf(
        "src/main_tests/service_api_endpoint_tests/websocket_contract_tests/presence_validation_contract_tests/cross_owner_scope_contract_tests.rs",
        &["fn regression_service_api_endpoint_websocket_presence_mode_rejects_cross_owner_scope()"],
        "websocket cross-owner contract file",
    );
}

fn assert_presence_legacy_leaves() {
    assert_single_leaf(
        "src/main_tests/service_api_endpoint_tests/websocket_contract_tests/presence_legacy_header_contract_tests/legacy_owner_contract_tests.rs",
        &["fn regression_service_api_endpoint_websocket_presence_mode_rejects_legacy_owner_did_header()"],
        "websocket legacy-owner contract file",
    );
    assert_single_leaf(
        "src/main_tests/service_api_endpoint_tests/websocket_contract_tests/presence_legacy_header_contract_tests/legacy_target_owner_contract_tests.rs",
        &["fn regression_service_api_endpoint_websocket_presence_mode_rejects_legacy_target_owner_did_header()"],
        "websocket legacy-target-owner contract file",
    );
    assert_single_leaf(
        "src/main_tests/service_api_endpoint_tests/websocket_contract_tests/presence_legacy_header_contract_tests/legacy_target_agent_contract_tests.rs",
        &["fn regression_service_api_endpoint_websocket_presence_mode_rejects_legacy_target_agent_did_header()"],
        "websocket legacy-target-agent contract file",
    );
}

fn assert_rejection_leaves() {
    assert_single_leaf(
        "src/main_tests/service_api_endpoint_tests/websocket_contract_tests/route_rejection_contract_tests/upgrade_header_contract_tests.rs",
        &["fn regression_service_api_endpoint_websocket_route_rejects_missing_upgrade_headers()"],
        "websocket upgrade-header contract file",
    );
    assert_single_leaf(
        "src/main_tests/service_api_endpoint_tests/websocket_contract_tests/route_rejection_contract_tests/version_header_contract_tests.rs",
        &["fn regression_service_api_endpoint_websocket_rejects_invalid_version_header()"],
        "websocket version-header contract file",
    );
}

fn assert_single_leaf(path: &str, markers: &[&str], label: &str) {
    let source = read_repo_file(path);
    assert_contains_markers(source.as_str(), markers, label);
}
