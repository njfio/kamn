use std::fs;
use std::path::{Path, PathBuf};

const ROOT: &str = "tests/ci_strategy_docs.rs";
const MODULE_DIR: &str = "tests/ci_strategy_docs";
const ROOT_MAX_LINES: usize = 980;
const REQUIRED_MODULES: &[&str] = &[
    "governance_advisory_tail_contract_tests.rs",
    "governance_advisory_tail_contract_tests/sbom_dependency_contract_tests.rs",
    "governance_advisory_tail_contract_tests/gonogo_boundary_contract_tests.rs",
    "governance_advisory_tail_contract_tests/audit_lifecycle_retention_contract_tests.rs",
    "governance_advisory_tail_contract_tests/support.rs",
];
const REQUIRED_MARKERS: &[&str] = &["mod governance_advisory_tail_contract_tests;"];
const MOVED_TEST_MARKERS: &[&str] = &[
    "fn doc_contains_sbom_provenance_artifact_generator_contract_markers()",
    "fn doc_contains_sbom_provenance_release_gonogo_checker_contract_markers()",
    "fn doc_contains_dependency_ci_smoke_advisory_fixture_contract_markers()",
    "fn doc_contains_dependency_ci_smoke_checker_threshold_parity_markers()",
    "fn doc_contains_cargo_audit_runner_impact_measurement_markers()",
    "fn doc_contains_anti_flake_rerun_policy_reason_taxonomy_markers()",
    "fn doc_contains_merge_gate_reliability_ci_smoke_local_heavy_boundary_markers()",
    "fn doc_contains_incident_gonogo_boundary_governance_matrix()",
    "fn doc_contains_incident_gonogo_boundary_reason_taxonomy_markers()",
    "fn doc_contains_live_gonogo_boundary_governance_matrix()",
    "fn doc_contains_live_gonogo_boundary_reason_taxonomy_markers()",
    "fn doc_contains_audit_integrity_dry_run_governance_markers()",
    "fn doc_contains_lifecycle_ci_dry_run_governance_markers()",
    "fn doc_contains_retention_policy_checker_taxonomy_contract_markers()",
];

#[test]
fn ci_strategy_docs_governance_advisory_tail_is_extracted() {
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
            "moved governance/advisory marker still present in root: {marker}"
        );
    }
}

fn repo_path(path: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join(path)
}
