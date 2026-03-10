use super::support::line_count;

const ROOT_MAX_LINES: usize = 200;
const EXTRACTED_MAX_LINES: usize = 200;
const EXTRACTED_FILES: &[&str] = &[
    "src/main_tests/service_api_endpoint_tests_split_contract/budget_contract_tests.rs",
    "src/main_tests/service_api_endpoint_tests_split_contract/root_layout_contract_tests.rs",
    "src/main_tests/service_api_endpoint_tests_split_contract/websocket_contract_tests.rs",
    "src/main_tests/service_api_endpoint_tests_split_contract/auth_scope_contract_tests.rs",
    "src/main_tests/service_api_endpoint_tests_split_contract/route_render_contract_tests.rs",
    "src/main_tests/service_api_endpoint_tests_split_contract/message_persistence_contract_tests.rs",
    "src/main_tests/service_api_endpoint_tests_split_contract/channel_agent_directory_contract_tests.rs",
    "src/main_tests/service_api_endpoint_tests_split_contract/task_escrow_persistence_contract_tests.rs",
    "src/main_tests/service_api_endpoint_tests_split_contract/content_lifecycle_restart_contract_tests.rs",
    "src/main_tests/service_api_endpoint_tests_split_contract/bridge_persistence_restart_contract_tests.rs",
    "src/main_tests/service_api_endpoint_tests_split_contract/mailbox_relay_delivery_contract_tests.rs",
    "src/main_tests/service_api_endpoint_tests_split_contract/ingress_guard_lifecycle_contract_tests.rs",
    "src/main_tests/service_api_endpoint_tests_split_contract/transport_surface_observability_contract_tests.rs",
    "src/main_tests/service_api_endpoint_tests_split_contract/shared_support_contract_tests.rs",
    "src/main_tests/service_api_endpoint_tests_split_contract/residual_root_contract_tests.rs",
    "src/main_tests/service_api_endpoint_tests_split_contract/support.rs",
];

#[test]
fn spec_c53_service_api_endpoint_split_contract_root_and_extracted_files_stay_within_budget() {
    assert!(
        line_count("src/main_tests/service_api_endpoint_tests_split_contract.rs") <= ROOT_MAX_LINES,
        "service_api_endpoint_tests_split_contract.rs should stay within {ROOT_MAX_LINES} lines"
    );
    for path in EXTRACTED_FILES {
        assert!(
            line_count(path) <= EXTRACTED_MAX_LINES,
            "{path} should stay within {EXTRACTED_MAX_LINES} lines"
        );
    }
}
