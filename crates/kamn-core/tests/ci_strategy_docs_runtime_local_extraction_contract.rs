use std::fs;
use std::path::{Path, PathBuf};

const ROOT: &str = "tests/ci_strategy_docs.rs";
const MODULE_DIR: &str = "tests/ci_strategy_docs";
const ROOT_MAX_LINES: usize = 1260;
const REQUIRED_MODULES: &[&str] = &[
    "runtime_local_contract_lane_tests.rs",
    "runtime_local_contract_lane_tests/observability_retry_contract_tests.rs",
    "runtime_local_contract_lane_tests/signal_metrics_contract_tests.rs",
    "runtime_local_contract_lane_tests/discovery_observability_contract_tests.rs",
    "runtime_local_contract_lane_tests/support.rs",
];
const REQUIRED_MARKERS: &[&str] = &["mod runtime_local_contract_lane_tests;"];
const MOVED_TEST_MARKERS: &[&str] = &[
    "fn doc_contains_runtime_observability_endpoint_contract_lane_ci_mode_markers()",
    "fn doc_contains_runtime_local_retry_diagnostics_contract_lane_ci_mode_markers()",
    "fn doc_contains_runtime_local_signal_secret_hygiene_contract_lane_ci_mode_markers()",
    "fn doc_contains_runtime_local_metrics_scrape_contract_lane_ci_mode_markers()",
    "fn doc_contains_runtime_libp2p_three_node_discovery_contract_lane_ci_mode_markers()",
    "fn doc_contains_runtime_local_observability_scrape_contract_lane_ci_mode_markers()",
];

#[test]
fn ci_strategy_docs_runtime_local_tranche_is_extracted() {
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
            "moved runtime-local marker still present in root: {marker}"
        );
    }
}

fn repo_path(path: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join(path)
}
