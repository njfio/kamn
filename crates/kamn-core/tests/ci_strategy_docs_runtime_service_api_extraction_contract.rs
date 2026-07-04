use std::fs;
use std::path::{Path, PathBuf};

const ROOT: &str = "tests/ci_strategy_docs.rs";
const MODULE_DIR: &str = "tests/ci_strategy_docs";
const ROOT_MAX_LINES: usize = 1500;
const REQUIRED_MODULES: &[&str] = &[
    "service_api_runtime_contract_lane_tests.rs",
    "service_api_runtime_contract_lane_tests/axum_ingress_contract_tests.rs",
    "service_api_runtime_contract_lane_tests/serde_reason_contract_tests.rs",
    "service_api_runtime_contract_lane_tests/validation_tenant_version_contract_tests.rs",
    "service_api_runtime_contract_lane_tests/shutdown_metrics_contract_tests.rs",
    "service_api_runtime_contract_lane_tests/support.rs",
];
const REQUIRED_MARKERS: &[&str] = &["mod service_api_runtime_contract_lane_tests;"];
const MOVED_TEST_MARKERS: &[&str] = &[
    "fn doc_contains_runtime_service_api_axum_ingress_contract_lane_ci_mode_markers()",
    "fn doc_contains_runtime_service_api_serde_payload_parity_contract_lane_ci_mode_markers()",
    "fn doc_contains_runtime_service_api_reason_code_compatibility_contract_lane_ci_mode_markers()",
    "fn doc_contains_runtime_service_api_validation_negative_matrix_contract_lane_ci_mode_markers()",
    "fn doc_contains_runtime_service_api_tenant_isolation_matrix_contract_lane_ci_mode_markers()",
    "fn doc_contains_runtime_api_version_policy_contract_lane_ci_mode_markers()",
    "fn doc_contains_runtime_service_api_graceful_shutdown_drain_contract_lane_ci_mode_markers()",
    "fn doc_contains_runtime_service_api_shutdown_abrupt_close_regression_contract_lane_ci_mode_markers()",
    "fn doc_contains_runtime_service_api_prometheus_metrics_contract_lane_ci_mode_markers()",
];

#[test]
fn ci_strategy_docs_runtime_service_api_tranche_is_extracted() {
    let root = fs::read_to_string(repo_path(ROOT)).expect("read root");
    let lines = root.lines().count();
    assert!(
        lines <= ROOT_MAX_LINES,
        "expected {ROOT} <= {ROOT_MAX_LINES} lines after tranche extraction, found {lines}"
    );
    for marker in REQUIRED_MARKERS {
        assert!(
            root.contains(marker),
            "missing root module marker: {marker}"
        );
    }
    for name in REQUIRED_MODULES {
        let path = repo_path(MODULE_DIR).join(name);
        assert!(
            path.exists(),
            "missing extracted module: {}",
            path.display()
        );
    }
    for marker in MOVED_TEST_MARKERS {
        assert!(
            !root.contains(marker),
            "moved runtime service-api marker still present in root: {marker}"
        );
    }
}

fn repo_path(path: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join(path)
}
