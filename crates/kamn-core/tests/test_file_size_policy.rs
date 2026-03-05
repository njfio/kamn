use std::fs;
use std::path::{Path, PathBuf};

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("repo root should resolve")
}

fn line_count(path: &Path) -> usize {
    fs::read_to_string(path)
        .unwrap_or_else(|error| panic!("failed to read {}: {error}", path.display()))
        .lines()
        .count()
}

#[test]
fn spec_c01_first_wave_command_contract_monolith_is_below_severe_threshold() {
    let path = repo_root().join("crates/kamn-e2e-harness/tests/command_contract.rs");
    let lines = line_count(&path);
    assert!(
        lines <= 2000,
        "first-wave severe threshold exceeded for {}: line_count={lines} threshold=2000",
        path.display()
    );
}

#[test]
fn spec_c02_test_file_size_policy_fixtures_exist() {
    let root = repo_root();
    let threshold_file = root.join(".ci/test_file_size_policy_thresholds.env");
    let baseline_file = root.join("fixtures/ci/test_file_size_policy_baseline.env");
    assert!(
        threshold_file.is_file(),
        "missing threshold file: {}",
        threshold_file.display()
    );
    assert!(
        baseline_file.is_file(),
        "missing baseline file: {}",
        baseline_file.display()
    );
}
