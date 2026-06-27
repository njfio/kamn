use std::path::{Path, PathBuf};

#[test]
fn production_expect_checker_excludes_named_test_only_surfaces() {
    let checker = read_repo_file("scripts/ci/check_no_production_expect.py");

    assert_contains(&checker, "\"/runtime_tests/\"");
    assert_contains(&checker, "\"/cli_scripted_tests/\"");
    assert_contains(&checker, "\"/mcp_agent_tests/\"");
    assert_contains(&checker, "file_name == \"test_support.rs\"");
}

#[test]
fn production_expect_checker_names_test_only_path_markers() {
    let checker = read_repo_file("scripts/ci/check_no_production_expect.py");

    assert_contains(&checker, "TEST_ONLY_PATH_MARKERS");
}

#[test]
fn production_expect_spec_records_ci_compatible_integration_boundary() {
    let spec = read_repo_file("specs/7025-repair-production-expect-ci-tool-contract.md");

    assert_contains(&spec, "temporary `/opt/homebrew/bin/gdate` shim");
    assert_contains(&spec, "test_check_no_production_expect");
}

#[test]
fn production_expect_spec_records_closeout_gate_evidence() {
    let spec = read_repo_file("specs/7025-repair-production-expect-ci-tool-contract.md");

    assert_contains(
        &spec,
        "cargo clippy --workspace --all-targets --all-features -- -D warnings",
    );
    assert_contains(&spec, "`make check` passed");
}

fn assert_contains(haystack: &str, needle: &str) {
    assert!(haystack.contains(needle), "missing marker: {needle}");
}

fn read_repo_file(relative_path: &str) -> String {
    std::fs::read_to_string(repo_root().join(relative_path))
        .unwrap_or_else(|error| panic!("failed to read {relative_path}: {error}"))
}

fn repo_root() -> PathBuf {
    manifest_dir()
        .parent()
        .and_then(Path::parent)
        .expect("kamn-core manifest should live under crates/kamn-core")
        .to_path_buf()
}

fn manifest_dir() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR"))
}
