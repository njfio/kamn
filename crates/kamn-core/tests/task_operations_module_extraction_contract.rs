use std::fs;
use std::path::PathBuf;

const ROOT_FILE: &str = "src/task_operations.rs";
const ROOT_BUDGET_LINES: usize = 180;
const MODULE_BUDGET_LINES: usize = 200;
const MODULE_FILES: &[&str] = &[
    "src/task_operations/models.rs",
    "src/task_operations/engine.rs",
    "src/task_operations/snapshot_store.rs",
    "src/task_operations/snapshot_codec.rs",
    "src/task_operations/tests.rs",
];
const ROOT_MARKERS: &[&str] = &[
    "mod models;",
    "mod engine;",
    "mod snapshot_store;",
    "mod snapshot_codec;",
    "#[cfg(test)] mod tests;",
];
const ROOT_INLINE_MARKERS: &[&str] = &[
    "pub struct TaskOperationRecord {",
    "pub struct TaskOperationEngine {",
    "pub struct FileTaskOperationSnapshotStore {",
    "fn parse_task_operation_snapshot_payload(",
    "mod tests {",
];

fn repo_file(path: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(path)
}

fn read_repo_file(path: &str) -> String {
    fs::read_to_string(repo_file(path)).unwrap_or_else(|error| {
        panic!("failed to read {path}: {error}");
    })
}

#[test]
fn regression_task_operations_root_respects_staged_line_budget() {
    let root = read_repo_file(ROOT_FILE);
    let line_count = root.lines().count();
    assert!(
        line_count <= ROOT_BUDGET_LINES,
        "expected {ROOT_FILE} to stay within {ROOT_BUDGET_LINES} lines, found {line_count}"
    );
}

#[test]
fn regression_task_operations_root_declares_extracted_modules() {
    let root = read_repo_file(ROOT_FILE);
    for marker in ROOT_MARKERS {
        assert!(root.contains(marker), "missing root module marker: {marker}");
    }
}

#[test]
fn regression_task_operations_module_files_exist() {
    for path in MODULE_FILES {
        assert!(repo_file(path).is_file(), "expected extracted module file to exist: {path}");
    }
}

#[test]
fn regression_task_operations_module_files_stay_within_budget() {
    for path in MODULE_FILES {
        let source = read_repo_file(path);
        let line_count = source.lines().count();
        assert!(
            line_count <= MODULE_BUDGET_LINES,
            "expected {path} to stay within {MODULE_BUDGET_LINES} lines, found {line_count}"
        );
    }
}

#[test]
fn regression_task_operations_root_removes_inline_monolith_sections() {
    let root = read_repo_file(ROOT_FILE);
    for marker in ROOT_INLINE_MARKERS {
        assert!(
            !root.contains(marker),
            "root should not retain inline task-operations section: {marker}"
        );
    }
}
