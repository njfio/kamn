use std::fs;
use std::path::Path;

const ROOT: &str = "src/service_api_endpoint/message_store.rs";
const ROOT_BUDGET: usize = 220;
const REQUIRED_ROOT_MARKERS: &[&str] = &[
    "mod models;",
    "mod persistence;",
    "mod store;",
    "mod runtime_evidence;",
    "#[cfg(test)]",
    "mod tests;",
];
const REQUIRED_FILES: &[&str] = &[
    "src/service_api_endpoint/message_store/models.rs",
    "src/service_api_endpoint/message_store/persistence.rs",
    "src/service_api_endpoint/message_store/store.rs",
    "src/service_api_endpoint/message_store/runtime_evidence.rs",
    "src/service_api_endpoint/message_store/tests.rs",
];
const MOVED_ROOT_MARKERS: &[&str] = &[
    "struct ServiceApiPersistedMessageRecord",
    "struct ServiceApiPersistedMessageStoreSnapshot",
    "impl ServiceApiMessageStore {",
    "fn build_runtime_evidence_context<'a>",
];

#[test]
fn message_store_root_is_extracted_into_bounded_modules() {
    let source = fs::read_to_string(ROOT).expect("message_store root should be readable");
    assert_root_budget(source.as_str());
    assert_root_markers(source.as_str());
    assert_moved_sections_leave_root(source.as_str());
    assert_extracted_files();
}

fn assert_root_budget(source: &str) {
    let line_count = source.lines().count();
    assert!(
        line_count <= ROOT_BUDGET,
        "expected {ROOT} to be <= {ROOT_BUDGET} lines after extraction, got {line_count}"
    );
}

fn assert_root_markers(source: &str) {
    for marker in REQUIRED_ROOT_MARKERS {
        assert!(
            source.contains(marker),
            "expected root shell to contain module marker `{marker}`"
        );
    }
}

fn assert_moved_sections_leave_root(source: &str) {
    for marker in MOVED_ROOT_MARKERS {
        assert!(
            !source.contains(marker),
            "expected extracted root to remove marker `{marker}`"
        );
    }
}

fn assert_extracted_files() {
    for path in REQUIRED_FILES {
        let file = Path::new(path);
        assert!(file.exists(), "expected extracted file `{path}` to exist");
        let line_count = fs::read_to_string(file)
            .unwrap_or_else(|error| panic!("failed to read `{path}`: {error}"))
            .lines()
            .count();
        assert!(
            line_count <= 200,
            "expected extracted file `{path}` to stay within 200 lines, got {line_count}"
        );
    }
}
