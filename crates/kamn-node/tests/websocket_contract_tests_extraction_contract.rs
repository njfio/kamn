use std::fs;
use std::path::{Path, PathBuf};

const ROOT_FILE: &str = "src/main_tests/service_api_endpoint_tests/websocket_contract_tests.rs";
const ROOT_MAX_LINES: usize = 200;
const EXPECTED_ROOT_MARKERS: &[&str] = &[
    "#[path = \"websocket_contract_tests/support.rs\"]",
    "mod support;",
    "#[path = \"websocket_contract_tests/upgrade_delivery_contract_tests.rs\"]",
    "mod upgrade_delivery_contract_tests;",
    "#[path = \"websocket_contract_tests/presence_projection_contract_tests.rs\"]",
    "mod presence_projection_contract_tests;",
    "#[path = \"websocket_contract_tests/presence_validation_contract_tests.rs\"]",
    "mod presence_validation_contract_tests;",
    "#[path = \"websocket_contract_tests/presence_legacy_header_contract_tests.rs\"]",
    "mod presence_legacy_header_contract_tests;",
    "#[path = \"websocket_contract_tests/route_rejection_contract_tests.rs\"]",
    "mod route_rejection_contract_tests;",
];
const EXPECTED_MODULE_FILES: &[&str] = &[
    "src/main_tests/service_api_endpoint_tests/websocket_contract_tests/support.rs",
    "src/main_tests/service_api_endpoint_tests/websocket_contract_tests/upgrade_delivery_contract_tests.rs",
    "src/main_tests/service_api_endpoint_tests/websocket_contract_tests/presence_projection_contract_tests.rs",
    "src/main_tests/service_api_endpoint_tests/websocket_contract_tests/presence_validation_contract_tests.rs",
    "src/main_tests/service_api_endpoint_tests/websocket_contract_tests/presence_legacy_header_contract_tests.rs",
    "src/main_tests/service_api_endpoint_tests/websocket_contract_tests/route_rejection_contract_tests.rs",
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
fn regression_websocket_contract_root_declares_extracted_submodules() {
    let root = read_repo_file(ROOT_FILE);
    for marker in EXPECTED_ROOT_MARKERS {
        assert!(
            root.contains(marker),
            "websocket_contract_tests.rs missing extracted-module marker: {marker}"
        );
    }
}

#[test]
fn regression_websocket_contract_module_files_exist() {
    let root = repo_root();
    for path in EXPECTED_MODULE_FILES {
        let full_path = root.join(path);
        assert!(
            full_path.exists(),
            "expected websocket contract module file missing: {}",
            full_path.display()
        );
    }
}

#[test]
fn regression_websocket_contract_root_stays_within_shell_budget() {
    let line_count = read_repo_file(ROOT_FILE).lines().count();
    assert!(
        line_count <= ROOT_MAX_LINES,
        "websocket_contract_tests.rs should stay within {ROOT_MAX_LINES} lines after extraction, found {line_count}"
    );
}
