use std::fs;
use std::path::{Path, PathBuf};

const ROOT_FILE: &str = "src/main_tests/service_api_endpoint_tests_split_contract.rs";
const ROOT_MAX_LINES: usize = 200;
const EXPECTED_MODULE_DECLARATIONS: &[&str] = &[
    "#[path = \"service_api_endpoint_tests_split_contract/budget_contract_tests.rs\"]",
    "mod budget_contract_tests;",
    "#[path = \"service_api_endpoint_tests_split_contract/root_layout_contract_tests.rs\"]",
    "mod root_layout_contract_tests;",
    "#[path = \"service_api_endpoint_tests_split_contract/websocket_contract_tests.rs\"]",
    "mod websocket_contract_tests;",
    "#[path = \"service_api_endpoint_tests_split_contract/auth_scope_contract_tests.rs\"]",
    "mod auth_scope_contract_tests;",
    "#[path = \"service_api_endpoint_tests_split_contract/route_render_contract_tests.rs\"]",
    "mod route_render_contract_tests;",
    "#[path = \"service_api_endpoint_tests_split_contract/message_persistence_contract_tests.rs\"]",
    "mod message_persistence_contract_tests;",
    "#[path = \"service_api_endpoint_tests_split_contract/channel_agent_directory_contract_tests.rs\"]",
    "mod channel_agent_directory_contract_tests;",
    "#[path = \"service_api_endpoint_tests_split_contract/task_escrow_persistence_contract_tests.rs\"]",
    "mod task_escrow_persistence_contract_tests;",
    "#[path = \"service_api_endpoint_tests_split_contract/content_lifecycle_restart_contract_tests.rs\"]",
    "mod content_lifecycle_restart_contract_tests;",
    "#[path = \"service_api_endpoint_tests_split_contract/bridge_persistence_restart_contract_tests.rs\"]",
    "mod bridge_persistence_restart_contract_tests;",
    "#[path = \"service_api_endpoint_tests_split_contract/mailbox_relay_delivery_contract_tests.rs\"]",
    "mod mailbox_relay_delivery_contract_tests;",
    "#[path = \"service_api_endpoint_tests_split_contract/ingress_guard_lifecycle_contract_tests.rs\"]",
    "mod ingress_guard_lifecycle_contract_tests;",
    "#[path = \"service_api_endpoint_tests_split_contract/transport_surface_observability_contract_tests.rs\"]",
    "mod transport_surface_observability_contract_tests;",
    "#[path = \"service_api_endpoint_tests_split_contract/shared_support_contract_tests.rs\"]",
    "mod shared_support_contract_tests;",
    "#[path = \"service_api_endpoint_tests_split_contract/residual_root_contract_tests.rs\"]",
    "mod residual_root_contract_tests;",
    "#[path = \"service_api_endpoint_tests_split_contract/support.rs\"]",
    "mod support;",
];
const EXPECTED_MODULE_FILES: &[&str] = &[
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

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).to_path_buf()
}

fn read_repo_file(path: &str) -> String {
    let full_path = repo_root().join(path);
    fs::read_to_string(&full_path)
        .unwrap_or_else(|error| panic!("failed to read {}: {error}", full_path.display()))
}

#[test]
fn regression_service_api_endpoint_split_contract_root_declares_extracted_submodules() {
    let root = read_repo_file(ROOT_FILE);
    for marker in EXPECTED_MODULE_DECLARATIONS {
        assert!(
            root.contains(marker),
            "service_api_endpoint_tests_split_contract.rs missing extracted-module marker: {marker}"
        );
    }
}

#[test]
fn regression_service_api_endpoint_split_contract_module_files_exist() {
    let root = repo_root();
    for path in EXPECTED_MODULE_FILES {
        let full_path = root.join(path);
        assert!(
            full_path.exists(),
            "expected service_api_endpoint split-contract module file missing: {}",
            full_path.display()
        );
    }
}

#[test]
fn regression_service_api_endpoint_split_contract_root_stays_within_shell_budget() {
    let line_count = read_repo_file(ROOT_FILE).lines().count();
    assert!(
        line_count <= ROOT_MAX_LINES,
        "service_api_endpoint_tests_split_contract.rs should stay within {ROOT_MAX_LINES} lines after extraction, found {line_count}"
    );
}
