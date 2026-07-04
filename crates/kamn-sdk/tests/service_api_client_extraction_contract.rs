use std::fs;
use std::path::{Path, PathBuf};

const ROOT: &str = "tests/service_api_client.rs";
const ROOT_CAP: usize = 180;
const MODULE_CAP: usize = 200;
const MODULE_FILES: &[&str] = &[
    "tests/service_api_client/support.rs",
    "tests/service_api_client/tls_contract_tests.rs",
    "tests/service_api_client/input_validation_contract_tests.rs",
    "tests/service_api_client/signed_http_route_contract_tests.rs",
    "tests/service_api_client/websocket_contract_tests.rs",
    "tests/service_api_client/route_family_contract_tests.rs",
];
const REQUIRED_MARKERS: &[&str] = &[
    "mod support;",
    "mod tls_contract_tests;",
    "mod input_validation_contract_tests;",
    "mod signed_http_route_contract_tests;",
    "mod websocket_contract_tests;",
    "mod route_family_contract_tests;",
];
const MOVED_MARKERS: &[&str] = &[
    "fn ensure_test_service_auth_private_key()",
    "fn generate_test_ca_signed_certificate_chain(temp_dir: &Path)",
    "fn run_service_contract_server(bind_addr: String, max_requests: u64)",
    "fn spec_c01_service_api_client_executes_https_health_route_with_trusted_ca()",
    "fn regression_service_api_client_rejects_crlf_route_identifier_payload()",
    "fn functional_service_api_client_executes_signed_http_route_contracts()",
    "fn integration_service_api_client_reads_websocket_event_frame()",
    "fn spec_c02_service_api_client_executes_task_transition_and_escrow_route_contracts()",
    "fn spec_c03_service_api_client_executes_bridge_route_contracts()",
];

#[test]
fn service_api_client_root_is_extracted() {
    let root = fs::read_to_string(repo_path(ROOT)).expect("read root");
    assert_root_shell_budget(&root);
    assert_required_markers(&root);
    assert_moved_markers_removed(&root);
    assert_module_files_exist_and_fit_budget();
}

fn repo_path(path: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join(path)
}

fn assert_root_shell_budget(root: &str) {
    let lines = root.lines().count();
    assert!(
        lines <= ROOT_CAP,
        "expected {ROOT} <= {ROOT_CAP} lines after extraction, found {lines}"
    );
}

fn assert_required_markers(root: &str) {
    for marker in REQUIRED_MARKERS {
        assert!(
            root.contains(marker),
            "missing root module marker: {marker}"
        );
    }
}

fn assert_moved_markers_removed(root: &str) {
    for marker in MOVED_MARKERS {
        assert!(
            !root.contains(marker),
            "moved marker still present in root: {marker}"
        );
    }
}

fn assert_module_files_exist_and_fit_budget() {
    for path in MODULE_FILES {
        let full = repo_path(path);
        let full_display = full.display();
        assert!(full.exists(), "missing extracted module: {full_display}");
        let lines = fs::read_to_string(&full)
            .expect("read module")
            .lines()
            .count();
        assert!(
            lines <= MODULE_CAP,
            "extracted module exceeds {MODULE_CAP} lines: {full_display}"
        );
    }
}
