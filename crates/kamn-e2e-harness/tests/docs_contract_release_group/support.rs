use std::path::{Path, PathBuf};

pub(crate) fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .to_path_buf()
}

pub(crate) fn read_relative(path: &str, message: &str) -> String {
    std::fs::read_to_string(repo_root().join(path)).expect(message)
}

pub(crate) fn read_milestone_index(path: &str, message: &str) -> String {
    read_relative(path, message)
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

pub(crate) fn assert_milestone_markers(path: &str, message: &str, markers: &[&str]) {
    let milestone_index = read_milestone_index(path, message);
    assert_text_contains_all(&milestone_index, markers);
}

macro_rules! release_doc_contract_tests {
    (
        $doc_test:ident,
        $doc_path:literal,
        $doc_message:literal,
        [$($doc_marker:literal),+ $(,)?],
        $milestone_test:ident,
        $milestone_path:literal,
        $milestone_message:literal,
        [$($milestone_marker:literal),+ $(,)?]
    ) => {
        #[test]
        fn $doc_test() {
            crate::support::assert_doc_markers($doc_path, $doc_message, &[$($doc_marker),+]);
        }

        #[test]
        fn $milestone_test() {
            crate::support::assert_milestone_markers(
                $milestone_path,
                $milestone_message,
                &[$($milestone_marker),+],
            );
        }
    };
}

pub(crate) use release_doc_contract_tests;
