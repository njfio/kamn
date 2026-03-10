use std::fs;
use std::path::{Path, PathBuf};

const ROOT_FILE: &str = "src/main_tests/daemon_tests/runtime_contract_tests.rs";
const ROOT_MAX_LINES: usize = 160;
const EXPECTED_ROOT_MARKERS: &[&str] = &[
    "include!(\"runtime_contract_tests/structured_transition_contract_tests.rs\");",
    "include!(\"runtime_contract_tests/structured_shutdown_contract_tests.rs\");",
    "include!(\"runtime_contract_tests/parse_control_contract_tests.rs\");",
    "include!(\"runtime_contract_tests/completion_output_contract_tests.rs\");",
    "include!(\"runtime_contract_tests/phase6_projection_contract_tests.rs\");",
    "include!(\"runtime_contract_tests/selector_bundle_contract_tests.rs\");",
];
const EXPECTED_MODULE_FILES: &[&str] = &[
    "src/main_tests/daemon_tests/runtime_contract_tests/structured_transition_contract_tests.rs",
    "src/main_tests/daemon_tests/runtime_contract_tests/structured_shutdown_contract_tests.rs",
    "src/main_tests/daemon_tests/runtime_contract_tests/parse_control_contract_tests.rs",
    "src/main_tests/daemon_tests/runtime_contract_tests/completion_output_contract_tests.rs",
    "src/main_tests/daemon_tests/runtime_contract_tests/phase6_projection_contract_tests.rs",
    "src/main_tests/daemon_tests/runtime_contract_tests/selector_bundle_contract_tests.rs",
];
const MOVED_TEST_MARKERS: &[&str] = &[
    "functional_runtime_daemon_emits_structured_transition_markers",
    "parses_runtime_mode_daemon_with_bounded_controls",
    "integration_runtime_daemon_renders_bounded_completion_output",
    "functional_runtime_daemon_projects_phase6_applied_runtime_markers_in_report_output",
    "functional_runtime_daemon_live_postgres_selector_bundle_validation_contract_is_deterministic",
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
fn regression_runtime_contract_root_declares_extracted_submodules() {
    let root = read_repo_file(ROOT_FILE);
    for marker in EXPECTED_ROOT_MARKERS {
        assert!(
            root.contains(marker),
            "runtime_contract_tests.rs missing extracted-module marker: {marker}"
        );
    }
}

#[test]
fn regression_runtime_contract_module_files_exist() {
    let root = repo_root();
    for path in EXPECTED_MODULE_FILES {
        let full_path = root.join(path);
        assert!(
            full_path.exists(),
            "expected runtime contract module file missing: {}",
            full_path.display()
        );
    }
}

#[test]
fn regression_runtime_contract_root_removes_moved_test_bodies() {
    let root = read_repo_file(ROOT_FILE);
    for marker in MOVED_TEST_MARKERS {
        assert!(
            !root.contains(marker),
            "runtime_contract_tests.rs should not retain moved test marker: {marker}"
        );
    }
}

#[test]
fn regression_runtime_contract_root_stays_within_shell_budget() {
    let line_count = read_repo_file(ROOT_FILE).lines().count();
    assert!(
        line_count <= ROOT_MAX_LINES,
        "runtime_contract_tests.rs should stay within {ROOT_MAX_LINES} lines after extraction, found {line_count}"
    );
}
