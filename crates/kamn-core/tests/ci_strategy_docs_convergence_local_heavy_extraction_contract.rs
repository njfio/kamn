use std::fs;
use std::path::{Path, PathBuf};

const ROOT: &str = "tests/ci_strategy_docs.rs";
const MODULE_DIR: &str = "tests/ci_strategy_docs";
const ROOT_MAX_LINES: usize = 2200;
const REQUIRED_MODULES: &[&str] = &[
    "convergence_governance_contract_tests.rs",
    "local_heavy_policy_contract_tests.rs",
];
const REQUIRED_MARKERS: &[&str] = &[
    "mod convergence_governance_contract_tests;",
    "mod local_heavy_policy_contract_tests;",
];
const MOVED_TEST_MARKERS: &[&str] = &[
    "fn doc_contains_transport_observability_tls_ci_smoke_convergence_governance()",
    "fn doc_contains_local_heavy_redaction_validation_policy_checker_contract()",
    "fn doc_contains_failover_drift_ci_smoke_convergence_governance()",
];

#[test]
fn ci_strategy_docs_convergence_local_heavy_tranche_is_extracted() {
    let root = fs::read_to_string(repo_path(ROOT)).expect("read root");
    let lines = root.lines().count();
    assert!(
        lines <= ROOT_MAX_LINES,
        "expected {ROOT} <= {ROOT_MAX_LINES} lines after tranche extraction, found {lines}"
    );
    for marker in REQUIRED_MARKERS {
        assert!(root.contains(marker), "missing root module marker: {marker}");
    }
    for name in REQUIRED_MODULES {
        let path = repo_path(MODULE_DIR).join(name);
        assert!(path.exists(), "missing extracted module: {}", path.display());
    }
    for marker in MOVED_TEST_MARKERS {
        assert!(
            !root.contains(marker),
            "moved convergence/local-heavy marker still present in root: {marker}"
        );
    }
}

fn repo_path(path: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join(path)
}
