use std::path::{Path, PathBuf};

pub(crate) fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("..").join("..")
}

pub(crate) fn read_file_if_exists(path: &Path) -> Option<String> {
    std::fs::read_to_string(path).ok()
}

pub(crate) fn workflow_fixture() -> String {
    let root = repo_root();
    read_file_if_exists(&root.join(".github/workflows/e2e-live.yml"))
        .expect("workflow fixture should exist")
}

pub(crate) fn strategy_fixture() -> String {
    let root = repo_root();
    read_file_if_exists(&root.join("docs/ci/strategy.md")).expect("strategy fixture exists")
}

pub(crate) fn count_occurrences(haystack: &str, needle: &str) -> usize {
    haystack.matches(needle).count()
}
