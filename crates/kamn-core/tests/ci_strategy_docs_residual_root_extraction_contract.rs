use std::fs;
use std::path::{Path, PathBuf};

const ROOT: &str = "tests/ci_strategy_docs.rs";
const ROOT_CAP: usize = 220;
const MODULE_FILES: &[&str] = &[
    "tests/ci_strategy_docs/residual_root_contract_tests.rs",
    "tests/ci_strategy_docs/residual_root_contract_tests/startup_surface_contract_tests.rs",
    "tests/ci_strategy_docs/residual_root_contract_tests/budget_runtime_contract_tests.rs",
    "tests/ci_strategy_docs/residual_root_contract_tests/anchoring_dependency_contract_tests.rs",
    "tests/ci_strategy_docs/residual_root_contract_tests/regression_selector_contract_tests.rs",
    "tests/ci_strategy_docs/residual_root_contract_tests/support.rs",
];
const REQUIRED_MARKERS: &[&str] = &[
    "#[path = \"ci_strategy_docs/residual_root_contract_tests.rs\"]",
    "mod residual_root_contract_tests;",
];
const MOVED_TEST_MARKERS: &[&str] = &[
    "fn doc_contains_touched_shell_strict_mode_markers()",
    "fn doc_contains_signer_provenance_fallback_policy_contract_markers()",
    "fn doc_contains_node_runtime_startup_negative_matrix_fast_lane_contract_markers()",
    "fn regression_requires_make_and_selector_demo_contract_marker()",
    "fn doc_contains_ignored_test_and_script_budget_trend_composed_contract_markers()",
    "fn doc_contains_combined_shell_surface_baseline_refresh_workflow_markers()",
    "fn doc_contains_test_harness_structural_budget_reason_taxonomy_and_ci_smoke_markers()",
    "fn doc_contains_runtime_local_full_mode_live_validation_runtime_error_taxonomy_markers()",
    "fn doc_contains_runtime_local_full_stack_runtime_budget_policy_markers()",
    "fn doc_contains_message_anchoring_ci_boundary_taxonomy_markers()",
    "fn doc_contains_dependency_license_metadata_governance_taxonomy_and_boundary_markers()",
    "fn assert_supply_chain_doc_marker(marker: &str)",
    "fn doc_contains_supply_chain_advisory_lane_markers()",
    "fn doc_enforces_dependency_license_metadata_remediation_markers_cover_reason_codes()",
];

#[test]
fn ci_strategy_docs_residual_root_tranche_is_extracted() {
    let root = fs::read_to_string(repo_path(ROOT)).expect("read root");
    let lines = root.lines().count();
    assert!(
        lines <= ROOT_CAP,
        "expected {ROOT} <= {ROOT_CAP} lines after residual extraction, found {lines}"
    );
    for marker in REQUIRED_MARKERS {
        assert!(
            root.contains(marker),
            "missing residual root marker: {marker}"
        );
    }
    for marker in MOVED_TEST_MARKERS {
        assert!(
            !root.contains(marker),
            "moved residual test still present: {marker}"
        );
    }
    for path in MODULE_FILES {
        let full_path = repo_path(path);
        assert!(
            full_path.exists(),
            "missing residual module: {}",
            full_path.display()
        );
        assert!(
            fs::read_to_string(&full_path)
                .expect("read module")
                .lines()
                .count()
                <= 200,
            "residual module exceeds 200 lines: {}",
            full_path.display()
        );
    }
}

fn repo_path(path: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join(path)
}
