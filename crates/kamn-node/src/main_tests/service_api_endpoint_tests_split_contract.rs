use std::fs;

fn read_repo_file(path: &str) -> String {
    let root = env!("CARGO_MANIFEST_DIR");
    let full_path = format!("{root}/{path}");
    fs::read_to_string(&full_path).unwrap_or_else(|error| {
        panic!("failed to read {path}: {error}");
    })
}

#[test]
fn spec_c01_service_api_endpoint_root_file_removes_websocket_helpers_and_tests() {
    let source = read_repo_file("src/main_tests/service_api_endpoint_tests.rs");
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
    let websocket = read_repo_file("src/main_tests/service_api_endpoint_tests/websocket_contract_tests.rs");
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
    let source = read_repo_file("src/main_tests/service_api_endpoint_tests.rs");
    assert!(
        source.contains("mod websocket_contract_tests;"),
        "service_api_endpoint_tests.rs should declare websocket submodule"
    );
}
