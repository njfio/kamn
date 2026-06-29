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

fn expected_files() -> [&'static str; 5] {
    [
        "crates/kamn-sdk/src/tcp/envelope.rs",
        "crates/kamn-sdk/src/tcp/handshake.rs",
        "crates/kamn-sdk/src/tcp/transport.rs",
        "crates/kamn-sdk/src/tcp/support.rs",
        "crates/kamn-sdk/src/tcp/tests.rs",
    ]
}

fn expected_markers() -> [&'static str; 6] {
    [
        "#[path = \"tcp/envelope.rs\"]",
        "#[path = \"tcp/handshake.rs\"]",
        "#[path = \"tcp/transport.rs\"]",
        "#[path = \"tcp/support.rs\"]",
        "#[cfg(test)]",
        "#[path = \"tcp/tests.rs\"]",
    ]
}

fn assert_root_budget(root: &str) {
    let root_lines = lines(root);
    assert!(
        root_lines <= 180,
        "expected {root} to be <= 180 LOC after extraction, got {root_lines}"
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
        let path_lines = lines(path);
        assert!(
            repo_root().join(path).is_file(),
            "expected extracted file {path} to exist"
        );
        assert!(
            path_lines <= 200,
            "expected {path} to be <= 200 LOC, got {path_lines}"
        );
    }
}

#[test]
fn regression_tcp_root_is_split_into_bounded_modules() {
    let root = "crates/kamn-sdk/src/tcp.rs";
    let root_contents = read(root);
    assert_root_budget(root);
    assert_root_markers(root_contents.as_str());
    assert_extracted_files();
}
