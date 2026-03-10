use std::fs;
use std::path::{Path, PathBuf};

const ROOT: &str = "tests/shell_test_surface_migration_wave1.rs";
const ROOT_CAP: usize = 120;
const MODULE_FILES: &[&str] = &[
    "tests/shell_test_surface_migration_wave1/support.rs",
    "tests/shell_test_surface_migration_wave1/ci_exclusion_contract_tests.rs",
    "tests/shell_test_surface_migration_wave1/workflow_policy_contract_tests.rs",
    "tests/shell_test_surface_migration_wave1/wrapper_parity_contract_tests.rs",
    "tests/shell_test_surface_migration_wave1/command_contract_tests.rs",
    "tests/shell_test_surface_migration_wave1/service_api_contract_tests.rs",
];
const REQUIRED_MARKERS: &[&str] = &[
    "mod support;",
    "mod ci_exclusion_contract_tests;",
    "mod workflow_policy_contract_tests;",
    "mod wrapper_parity_contract_tests;",
    "mod command_contract_tests;",
    "mod service_api_contract_tests;",
];
const MOVED_TEST_MARKERS: &[&str] = &[
    "fn spec_c01_block_reconciliation_partition_rejoin_ci_exclusion_policy_markers()",
    "fn spec_c03_performance_threshold_checker_contract()",
    "fn spec_c10_run_with_retry_contract()",
    "fn spec_c16_input_mutation_coverage_guided_contract_lane_wrapper_parity()",
    "fn spec_c20_libp2p_process_isolated_harness_validation_parity()",
];

#[test]
fn shell_test_surface_migration_wave1_root_is_extracted() {
    let root = fs::read_to_string(repo_path(ROOT)).expect("read root");
    assert_root_budget(&root);
    assert_root_markers(&root);
    assert_moved_tests_absent(&root);
    assert_module_files_exist_and_fit();
}

fn repo_path(path: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join(path)
}

fn assert_root_budget(root: &str) {
    let lines = root.lines().count();
    assert!(
        lines <= ROOT_CAP,
        "expected {ROOT} <= {ROOT_CAP} lines after extraction, found {lines}"
    );
}

fn assert_root_markers(root: &str) {
    for marker in REQUIRED_MARKERS {
        assert!(
            root.contains(marker),
            "missing root module marker: {marker}"
        );
    }
}

fn assert_moved_tests_absent(root: &str) {
    for marker in MOVED_TEST_MARKERS {
        assert!(
            !root.contains(marker),
            "moved test marker still present: {marker}"
        );
    }
}

fn assert_module_files_exist_and_fit() {
    for path in MODULE_FILES {
        let full = repo_path(path);
        assert!(
            full.exists(),
            "missing extracted module: {}",
            full.display()
        );
        assert_module_budget(&full);
    }
}

fn assert_module_budget(full: &Path) {
    let line_count = fs::read_to_string(full)
        .expect("read module")
        .lines()
        .count();
    assert!(
        line_count <= 200,
        "extracted module exceeds 200 lines: {}",
        full.display()
    );
}
