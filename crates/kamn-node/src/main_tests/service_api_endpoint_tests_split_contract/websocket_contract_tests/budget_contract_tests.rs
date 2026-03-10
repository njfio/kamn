use super::super::support::*;

const WEBSOCKET_SPLIT_PATHS: &[&str] = &[
    WEBSOCKET_FILE,
    "src/main_tests/service_api_endpoint_tests/websocket_contract_tests/support.rs",
    "src/main_tests/service_api_endpoint_tests/websocket_contract_tests/support/request_support.rs",
    "src/main_tests/service_api_endpoint_tests/websocket_contract_tests/support/frame_support.rs",
    "src/main_tests/service_api_endpoint_tests/websocket_contract_tests/upgrade_delivery_contract_tests.rs",
    "src/main_tests/service_api_endpoint_tests/websocket_contract_tests/upgrade_delivery_contract_tests/reason_taxonomy_contract_tests.rs",
    "src/main_tests/service_api_endpoint_tests/websocket_contract_tests/upgrade_delivery_contract_tests/upgrade_flow_contract_tests.rs",
    "src/main_tests/service_api_endpoint_tests/websocket_contract_tests/upgrade_delivery_contract_tests/live_delivery_contract_tests.rs",
    "src/main_tests/service_api_endpoint_tests/websocket_contract_tests/presence_projection_contract_tests.rs",
    "src/main_tests/service_api_endpoint_tests/websocket_contract_tests/presence_validation_contract_tests.rs",
    "src/main_tests/service_api_endpoint_tests/websocket_contract_tests/presence_validation_contract_tests/unsupported_mode_contract_tests.rs",
    "src/main_tests/service_api_endpoint_tests/websocket_contract_tests/presence_validation_contract_tests/missing_owner_contract_tests.rs",
    "src/main_tests/service_api_endpoint_tests/websocket_contract_tests/presence_validation_contract_tests/cross_owner_scope_contract_tests.rs",
    "src/main_tests/service_api_endpoint_tests/websocket_contract_tests/presence_legacy_header_contract_tests.rs",
    "src/main_tests/service_api_endpoint_tests/websocket_contract_tests/presence_legacy_header_contract_tests/legacy_owner_contract_tests.rs",
    "src/main_tests/service_api_endpoint_tests/websocket_contract_tests/presence_legacy_header_contract_tests/legacy_target_owner_contract_tests.rs",
    "src/main_tests/service_api_endpoint_tests/websocket_contract_tests/presence_legacy_header_contract_tests/legacy_target_agent_contract_tests.rs",
    "src/main_tests/service_api_endpoint_tests/websocket_contract_tests/route_rejection_contract_tests.rs",
    "src/main_tests/service_api_endpoint_tests/websocket_contract_tests/route_rejection_contract_tests/upgrade_header_contract_tests.rs",
    "src/main_tests/service_api_endpoint_tests/websocket_contract_tests/route_rejection_contract_tests/version_header_contract_tests.rs",
];

#[test]
fn spec_c04_websocket_split_files_stay_below_budget() {
    for path in WEBSOCKET_SPLIT_PATHS {
        assert_file_within_budget(path);
    }
}

fn assert_file_within_budget(path: &str) {
    let source = read_repo_file(path);
    let line_count = source.lines().count();
    assert!(
        line_count <= 200,
        "{path} should stay below 200 lines after extraction: line_count={line_count}"
    );
}
