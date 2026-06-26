use std::fs;
use std::path::{Path, PathBuf};

const ROOT: &str = "tests/ci_strategy_docs.rs";
const MODULE_DIR: &str = "tests/ci_strategy_docs";
const ROOT_MAX_LINES: usize = 2800;
const REQUIRED_MODULES: &[&str] = &[
    "service_api_request_path_authz_contract_tests.rs",
    "service_api_scope_policy_contract_tests.rs",
    "service_api_tenant_isolation_contract_tests.rs",
    "api_version_policy_contract_tests.rs",
    "request_response_schema_compatibility_contract_tests.rs",
];
const REQUIRED_MARKERS: &[&str] = &[
    "mod service_api_request_path_authz_contract_tests;",
    "mod service_api_scope_policy_contract_tests;",
    "mod service_api_tenant_isolation_contract_tests;",
    "mod api_version_policy_contract_tests;",
    "mod request_response_schema_compatibility_contract_tests;",
];
const MOVED_TEST_MARKERS: &[&str] = &[
    "fn doc_contains_service_api_request_path_authz_docs_parity_markers()",
    "fn doc_contains_service_api_scope_policy_docs_parity_markers()",
    "fn doc_contains_service_api_tenant_isolation_matrix_docs_parity_markers()",
    "fn doc_contains_api_version_policy_docs_parity_markers()",
    "fn doc_contains_runtime_request_response_schema_compatibility_contract_lane_ci_mode_markers()",
];

#[test]
fn ci_strategy_docs_service_api_policy_tranche_is_extracted() {
    let root = fs::read_to_string(repo_path(ROOT)).expect("read root");
    assert_root_budget(&root);
    assert_required_markers(&root);
    assert_required_modules_exist();
    assert_moved_tests_left_root(&root);
}

fn assert_root_budget(root: &str) {
    let lines = root.lines().count();
    assert!(
        lines <= ROOT_MAX_LINES,
        "expected {ROOT} <= {ROOT_MAX_LINES} lines after tranche extraction, found {lines}"
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

fn assert_required_modules_exist() {
    for name in REQUIRED_MODULES {
        let path = repo_path(MODULE_DIR).join(name);
        assert!(
            path.exists(),
            "missing extracted module: {}",
            path.display()
        );
    }
}

fn assert_moved_tests_left_root(root: &str) {
    for marker in MOVED_TEST_MARKERS {
        assert!(
            !root.contains(marker),
            "moved service-api test marker still present in root: {marker}"
        );
    }
}

fn repo_path(path: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join(path)
}
