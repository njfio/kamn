use super::support::read_repo_file;

const ROOT_FILE: &str = "src/main_tests/service_api_endpoint_tests_split_contract.rs";
const ROOT_MARKERS: &[&str] = &[
    r##"#[path = "service_api_endpoint_tests_split_contract/budget_contract_tests.rs"]"##,
    "mod budget_contract_tests;",
    r##"#[path = "service_api_endpoint_tests_split_contract/root_layout_contract_tests.rs"]"##,
    "mod root_layout_contract_tests;",
    r##"#[path = "service_api_endpoint_tests_split_contract/websocket_contract_tests.rs"]"##,
    "mod websocket_contract_tests;",
    r##"#[path = "service_api_endpoint_tests_split_contract/auth_scope_contract_tests.rs"]"##,
    "mod auth_scope_contract_tests;",
    r##"#[path = "service_api_endpoint_tests_split_contract/route_render_contract_tests.rs"]"##,
    "mod route_render_contract_tests;",
    r##"#[path = "service_api_endpoint_tests_split_contract/message_persistence_contract_tests.rs"]"##,
    "mod message_persistence_contract_tests;",
    r##"#[path = "service_api_endpoint_tests_split_contract/channel_agent_directory_contract_tests.rs"]"##,
    "mod channel_agent_directory_contract_tests;",
    r##"#[path = "service_api_endpoint_tests_split_contract/task_escrow_persistence_contract_tests.rs"]"##,
    "mod task_escrow_persistence_contract_tests;",
    r##"#[path = "service_api_endpoint_tests_split_contract/content_lifecycle_restart_contract_tests.rs"]"##,
    "mod content_lifecycle_restart_contract_tests;",
    r##"#[path = "service_api_endpoint_tests_split_contract/bridge_persistence_restart_contract_tests.rs"]"##,
    "mod bridge_persistence_restart_contract_tests;",
    r##"#[path = "service_api_endpoint_tests_split_contract/mailbox_relay_delivery_contract_tests.rs"]"##,
    "mod mailbox_relay_delivery_contract_tests;",
    r##"#[path = "service_api_endpoint_tests_split_contract/ingress_guard_lifecycle_contract_tests.rs"]"##,
    "mod ingress_guard_lifecycle_contract_tests;",
    r##"#[path = "service_api_endpoint_tests_split_contract/transport_surface_observability_contract_tests.rs"]"##,
    "mod transport_surface_observability_contract_tests;",
    r##"#[path = "service_api_endpoint_tests_split_contract/shared_support_contract_tests.rs"]"##,
    "mod shared_support_contract_tests;",
    r##"#[path = "service_api_endpoint_tests_split_contract/residual_root_contract_tests.rs"]"##,
    "mod residual_root_contract_tests;",
    r##"#[path = "service_api_endpoint_tests_split_contract/support.rs"]"##,
    "mod support;",
];

#[test]
fn spec_c54_service_api_endpoint_split_contract_root_declares_extracted_submodules() {
    let root = read_repo_file(ROOT_FILE);
    for marker in ROOT_MARKERS {
        assert!(
            root.contains(marker),
            "service_api_endpoint_tests_split_contract.rs should declare extracted marker: {marker}"
        );
    }
}
