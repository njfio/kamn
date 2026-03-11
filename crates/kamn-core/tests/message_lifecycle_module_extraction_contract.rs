use std::{fs, path::PathBuf};

const ROOT_MAX_LINES: usize = 180;
const ROOT_MARKERS: &[&str] = &[
    "mod domain;",
    "mod errors;",
    "mod lifecycle_store;",
    "mod snapshot_codec;",
    "mod snapshot_file_store;",
    "mod snapshot_sqlite_store;",
    "#[cfg(test)]",
    "mod tests;",
];
const REQUIRED_FILES: &[&str] = &[
    "src/message_lifecycle/domain.rs",
    "src/message_lifecycle/errors.rs",
    "src/message_lifecycle/lifecycle_store.rs",
    "src/message_lifecycle/snapshot_codec.rs",
    "src/message_lifecycle/snapshot_file_store.rs",
    "src/message_lifecycle/snapshot_sqlite_store.rs",
    "src/message_lifecycle/tests.rs",
    "src/message_lifecycle/tests/store_contract_tests.rs",
    "src/message_lifecycle/tests/snapshot_codec_contract_tests.rs",
    "src/message_lifecycle/tests/file_store_contract_tests.rs",
    "src/message_lifecycle/tests/performance_contract_tests.rs",
];

#[test]
fn message_lifecycle_root_shell_stays_within_budget() {
    let source = read_root();
    let line_count = source.lines().count();
    assert!(
        line_count <= ROOT_MAX_LINES,
        "message_lifecycle.rs should stay within {} lines, found {}",
        ROOT_MAX_LINES,
        line_count
    );
}

#[test]
fn message_lifecycle_root_declares_extracted_modules() {
    let source = read_root();
    for marker in ROOT_MARKERS {
        assert!(
            source.contains(marker),
            "message_lifecycle.rs should declare extracted module marker: {marker}"
        );
    }
}

#[test]
fn message_lifecycle_extracted_files_exist() {
    for relative in REQUIRED_FILES {
        let path = repo_root().join(relative);
        assert!(
            path.exists(),
            "expected extracted file to exist: {}",
            path.display()
        );
    }
}

fn read_root() -> String {
    fs::read_to_string(repo_root().join("src/message_lifecycle.rs"))
        .expect("message_lifecycle root should read")
}

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}
