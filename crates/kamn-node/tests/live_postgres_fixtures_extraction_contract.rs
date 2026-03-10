use std::fs;
use std::path::{Path, PathBuf};

const ROOT_FILE: &str = "src/main_tests/daemon_tests/live_postgres_fixtures.rs";
const ROOT_MAX_LINES: usize = 200;
const EXPECTED_ROOT_MARKERS: &[&str] = &[
    "#[path = \"live_postgres_fixtures/constants.rs\"]",
    "mod constants;",
    "#[path = \"live_postgres_fixtures/gate_support.rs\"]",
    "mod gate_support;",
    "#[path = \"live_postgres_fixtures/matrix_profiles.rs\"]",
    "mod matrix_profiles;",
    "#[path = \"live_postgres_fixtures/topology_projections.rs\"]",
    "mod topology_projections;",
    "#[path = \"live_postgres_fixtures/multi_host_execution.rs\"]",
    "mod multi_host_execution;",
];
const EXPECTED_MODULE_FILES: &[&str] = &[
    "src/main_tests/daemon_tests/live_postgres_fixtures/constants.rs",
    "src/main_tests/daemon_tests/live_postgres_fixtures/gate_support.rs",
    "src/main_tests/daemon_tests/live_postgres_fixtures/matrix_profiles.rs",
    "src/main_tests/daemon_tests/live_postgres_fixtures/topology_projections.rs",
    "src/main_tests/daemon_tests/live_postgres_fixtures/multi_host_execution.rs",
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
fn regression_live_postgres_fixture_root_declares_extracted_submodules() {
    let root = read_repo_file(ROOT_FILE);
    for marker in EXPECTED_ROOT_MARKERS {
        assert!(
            root.contains(marker),
            "live_postgres_fixtures.rs missing extracted-module marker: {marker}"
        );
    }
}

#[test]
fn regression_live_postgres_fixture_module_files_exist() {
    let root = repo_root();
    for path in EXPECTED_MODULE_FILES {
        let full_path = root.join(path);
        assert!(
            full_path.exists(),
            "expected live-postgres fixture module file missing: {}",
            full_path.display()
        );
    }
}

#[test]
fn regression_live_postgres_fixture_root_stays_within_shell_budget() {
    let line_count = read_repo_file(ROOT_FILE).lines().count();
    assert!(
        line_count <= ROOT_MAX_LINES,
        "live_postgres_fixtures.rs should stay within {ROOT_MAX_LINES} lines after extraction, found {line_count}"
    );
}
