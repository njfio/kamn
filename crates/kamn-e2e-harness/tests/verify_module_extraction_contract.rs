use std::fs;
use std::path::{Path, PathBuf};

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("crate dir")
        .parent()
        .expect("repo root")
        .to_path_buf()
}

fn read(path: &str) -> String {
    fs::read_to_string(repo_root().join(path)).expect("read source file")
}

fn lines(path: &str) -> usize {
    read(path).lines().count()
}

fn expected_files() -> [&'static str; 6] {
    [
        "crates/kamn-e2e-harness/src/verify/manifest.rs",
        "crates/kamn-e2e-harness/src/verify/chain_dump.rs",
        "crates/kamn-e2e-harness/src/verify/evidence.rs",
        "crates/kamn-e2e-harness/src/verify/report.rs",
        "crates/kamn-e2e-harness/src/verify/support.rs",
        "crates/kamn-e2e-harness/src/verify/tests.rs",
    ]
}

fn expected_markers() -> [&'static str; 7] {
    [
        "#[path = \"verify/manifest.rs\"]",
        "#[path = \"verify/chain_dump.rs\"]",
        "#[path = \"verify/evidence.rs\"]",
        "#[path = \"verify/report.rs\"]",
        "#[path = \"verify/support.rs\"]",
        "#[cfg(test)]",
        "#[path = \"verify/tests.rs\"]",
    ]
}

fn assert_root_budget(root: &str) {
    assert!(
        lines(root) <= 180,
        "expected {root} to be <= 180 LOC after extraction, got {}",
        lines(root)
    );
}

fn assert_root_markers(root_contents: &str) {
    for marker in expected_markers() {
        assert!(
            root_contents.contains(marker),
            "expected root shell to contain marker: {marker}"
        );
    }
}

fn assert_extracted_files() {
    for path in expected_files() {
        assert!(
            repo_root().join(path).is_file(),
            "expected extracted file {path} to exist"
        );
        assert!(
            lines(path) <= 200,
            "expected {path} to be <= 200 LOC, got {}",
            lines(path)
        );
    }
}

#[test]
fn regression_verify_root_is_split_into_bounded_modules() {
    let root = "crates/kamn-e2e-harness/src/verify.rs";
    let root_contents = read(root);
    assert_root_budget(root);
    assert_root_markers(root_contents.as_str());
    assert_extracted_files();
}
