use super::super::support::*;

const SUPPORT_FILE: &str =
    "src/main_tests/service_api_endpoint_tests/websocket_contract_tests/support.rs";
const REQUEST_SUPPORT_FILE: &str =
    "src/main_tests/service_api_endpoint_tests/websocket_contract_tests/support/request_support.rs";
const FRAME_SUPPORT_FILE: &str =
    "src/main_tests/service_api_endpoint_tests/websocket_contract_tests/support/frame_support.rs";
const UPGRADE_ROOT_FILE: &str =
    "src/main_tests/service_api_endpoint_tests/websocket_contract_tests/upgrade_delivery_contract_tests.rs";
const PRESENCE_VALIDATION_ROOT_FILE: &str =
    "src/main_tests/service_api_endpoint_tests/websocket_contract_tests/presence_validation_contract_tests.rs";
const PRESENCE_LEGACY_ROOT_FILE: &str =
    "src/main_tests/service_api_endpoint_tests/websocket_contract_tests/presence_legacy_header_contract_tests.rs";
const ROUTE_REJECTION_ROOT_FILE: &str =
    "src/main_tests/service_api_endpoint_tests/websocket_contract_tests/route_rejection_contract_tests.rs";

#[test]
fn spec_c02_service_api_endpoint_websocket_module_exists_and_owns_structure() {
    assert_websocket_root_structure();
    assert_support_root_markers();
    assert_support_leaf_markers();
    assert_nested_root_structure();
}

fn assert_websocket_root_structure() {
    let websocket = read_repo_file(WEBSOCKET_FILE);
    assert_contains_markers(
        websocket.as_str(),
        &[
            "mod support;",
            "mod upgrade_delivery_contract_tests;",
            "mod presence_projection_contract_tests;",
            "mod presence_validation_contract_tests;",
            "mod presence_legacy_header_contract_tests;",
            "mod route_rejection_contract_tests;",
        ],
        "websocket contract root",
    );
}

fn assert_support_root_markers() {
    let support = read_repo_file(SUPPORT_FILE);
    assert_contains_markers(
        support.as_str(),
        &["mod request_support;", "mod frame_support;"],
        "websocket support root",
    );
}

fn assert_support_leaf_markers() {
    assert_request_support_markers();
    assert_frame_support_markers();
}

fn assert_request_support_markers() {
    let request_support = read_repo_file(REQUEST_SUPPORT_FILE);
    assert_contains_markers(
        request_support.as_str(),
        &[
            "fn send_websocket_upgrade_request(",
            "fn send_websocket_upgrade_request_with_version(",
            "fn send_websocket_upgrade_request_with_version_close_observation(",
        ],
        "websocket request support",
    );
}

fn assert_frame_support_markers() {
    let frame_support = read_repo_file(FRAME_SUPPORT_FILE);
    assert_contains_markers(
        frame_support.as_str(),
        &[
            "fn parse_websocket_response_frames(",
            "fn parse_websocket_response(",
        ],
        "websocket frame support",
    );
}

fn assert_nested_root_structure() {
    assert_upgrade_structure();
    assert_presence_structure();
    assert_route_rejection_structure();
}

fn assert_upgrade_structure() {
    let upgrade_delivery = read_repo_file(UPGRADE_ROOT_FILE);
    assert_contains_markers(
        upgrade_delivery.as_str(),
        &[
            "mod reason_taxonomy_contract_tests;",
            "mod upgrade_flow_contract_tests;",
            "mod live_delivery_contract_tests;",
        ],
        "websocket upgrade delivery root",
    );
}

fn assert_presence_structure() {
    let validation = read_repo_file(PRESENCE_VALIDATION_ROOT_FILE);
    let legacy = read_repo_file(PRESENCE_LEGACY_ROOT_FILE);
    assert_contains_markers(
        validation.as_str(),
        &[
            "mod unsupported_mode_contract_tests;",
            "mod missing_owner_contract_tests;",
            "mod cross_owner_scope_contract_tests;",
        ],
        "websocket presence validation root",
    );
    assert_contains_markers(
        legacy.as_str(),
        &[
            "mod legacy_owner_contract_tests;",
            "mod legacy_target_owner_contract_tests;",
            "mod legacy_target_agent_contract_tests;",
        ],
        "websocket legacy-header root",
    );
}

fn assert_route_rejection_structure() {
    let route_rejection = read_repo_file(ROUTE_REJECTION_ROOT_FILE);
    assert_contains_markers(
        route_rejection.as_str(),
        &[
            "mod upgrade_header_contract_tests;",
            "mod version_header_contract_tests;",
        ],
        "websocket route rejection root",
    );
}
