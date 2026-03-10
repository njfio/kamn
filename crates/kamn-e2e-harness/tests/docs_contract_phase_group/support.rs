use std::path::{Path, PathBuf};

pub(crate) fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("..").join("..").to_path_buf()
}

pub(crate) fn read_relative(path: &str, message: &str) -> String {
    std::fs::read_to_string(repo_root().join(path)).expect(message)
}

pub(crate) fn read_milestone_index() -> String {
    read_relative(
        "specs/milestones/r51-e2e-live-testing-prd-full-delivery/index.md",
        "milestone index should exist",
    )
}

pub(crate) fn assert_text_contains_all(text: &str, markers: &[&str]) {
    for marker in markers {
        assert!(text.contains(marker), "expected text to contain `{marker}`");
    }
}

pub(crate) fn assert_doc_markers(path: &str, message: &str, markers: &[&str]) {
    let doc = read_relative(path, message);
    assert_text_contains_all(&doc, markers);
}

pub(crate) fn assert_milestone_markers(markers: &[&str]) {
    let milestone_index = read_milestone_index();
    assert_text_contains_all(&milestone_index, markers);
}

pub(crate) fn assert_required_paths_exist(paths: &[&str]) {
    let root = repo_root();
    for path in paths {
        assert!(root.join(path).is_file(), "required path missing: {path}");
    }
}
