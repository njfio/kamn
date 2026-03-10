use std::fs;
use std::path::{Path, PathBuf};

const ROOT_FILE: &str = "src/main_tests/daemon_tests/live_postgres_matrix_contract_tests.rs";
const ROOT_MAX_LINES: usize = 200;
const EXPECTED_ROOT_MARKERS: &[&str] = &[
    "include!(\"live_postgres_matrix_contract_tests/env_gate_execution_contract_tests.rs\");",
    "include!(\"live_postgres_matrix_contract_tests/projection_taxonomy_contract_tests.rs\");",
    "include!(\"live_postgres_matrix_contract_tests/load_profile_contract_tests.rs\");",
    "include!(\"live_postgres_matrix_contract_tests/role_profile_contract_tests.rs\");",
    "include!(\"live_postgres_matrix_contract_tests/role_pair_contract_tests.rs\");",
    "include!(\"live_postgres_matrix_contract_tests/parallel_role_pair_lane_contract_tests.rs\");",
    "include!(\"live_postgres_matrix_contract_tests/asymmetric_parallel_lane_contract_tests.rs\");",
    "include!(\"live_postgres_matrix_contract_tests/parallel_lane_invariance_contract_tests.rs\");",
];
const EXPECTED_MODULE_FILES: &[&str] = &[
    "src/main_tests/daemon_tests/live_postgres_matrix_contract_tests/env_gate_execution_contract_tests.rs",
    "src/main_tests/daemon_tests/live_postgres_matrix_contract_tests/projection_taxonomy_contract_tests.rs",
    "src/main_tests/daemon_tests/live_postgres_matrix_contract_tests/load_profile_contract_tests.rs",
    "src/main_tests/daemon_tests/live_postgres_matrix_contract_tests/role_profile_contract_tests.rs",
    "src/main_tests/daemon_tests/live_postgres_matrix_contract_tests/role_pair_contract_tests.rs",
    "src/main_tests/daemon_tests/live_postgres_matrix_contract_tests/parallel_role_pair_lane_contract_tests.rs",
    "src/main_tests/daemon_tests/live_postgres_matrix_contract_tests/asymmetric_parallel_lane_contract_tests.rs",
    "src/main_tests/daemon_tests/live_postgres_matrix_contract_tests/parallel_lane_invariance_contract_tests.rs",
];
const MOVED_TEST_MARKERS: &[&str] = &[
    "integration_runtime_daemon_phase6_live_postgres_validation_slice()",
    "functional_runtime_daemon_live_postgres_validation_slice_load_profile_matrix_contract_is_canonical",
    "integration_runtime_daemon_phase6_live_postgres_validation_slice_role_profile_matrix_is_deterministic",
    "integration_runtime_daemon_phase6_live_postgres_validation_slice_role_pair_matrix_is_deterministic",
    "integration_runtime_daemon_phase6_live_postgres_validation_slice_parallel_lane_permutations_are_invariant",
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
fn regression_live_postgres_matrix_root_declares_extracted_submodules() {
    let root = read_repo_file(ROOT_FILE);
    for marker in EXPECTED_ROOT_MARKERS {
        assert!(
            root.contains(marker),
            "live_postgres_matrix_contract_tests.rs missing extracted-module marker: {marker}"
        );
    }
}

#[test]
fn regression_live_postgres_matrix_module_files_exist() {
    let root = repo_root();
    for path in EXPECTED_MODULE_FILES {
        let full_path = root.join(path);
        assert!(
            full_path.exists(),
            "expected live-postgres matrix module file missing: {}",
            full_path.display()
        );
    }
}

#[test]
fn regression_live_postgres_matrix_root_removes_moved_test_bodies() {
    let root = read_repo_file(ROOT_FILE);
    for marker in MOVED_TEST_MARKERS {
        assert!(
            !root.contains(marker),
            "live_postgres_matrix_contract_tests.rs should not retain moved test marker: {marker}"
        );
    }
}

#[test]
fn regression_live_postgres_matrix_root_stays_within_shell_budget() {
    let line_count = read_repo_file(ROOT_FILE).lines().count();
    assert!(
        line_count <= ROOT_MAX_LINES,
        "live_postgres_matrix_contract_tests.rs should stay within {ROOT_MAX_LINES} lines after extraction, found {line_count}"
    );
}
