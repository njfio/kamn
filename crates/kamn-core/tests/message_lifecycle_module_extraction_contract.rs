use std::fs;
use std::path::Path;

const ROOT: &str = "src/message_lifecycle.rs";
const ROOT_BUDGET: usize = 220;
const REQUIRED_MODULE_MARKERS: &[&str] = &[
    "mod lifecycle_types;",
    "mod lifecycle_store;",
    "mod snapshot_store;",
    "mod proof_admission;",
    "mod lifecycle_errors;",
];
const REQUIRED_FILES: &[&str] = &[
    "src/message_lifecycle/lifecycle_types.rs",
    "src/message_lifecycle/lifecycle_store.rs",
    "src/message_lifecycle/snapshot_store.rs",
    "src/message_lifecycle/proof_admission.rs",
    "src/message_lifecycle/lifecycle_errors.rs",
];

#[test]
fn message_lifecycle_root_is_extracted() {
    let source = fs::read_to_string(ROOT).expect("message_lifecycle root should be readable");
    assert_root_budget(source.as_str());
    assert_root_markers(source.as_str());
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
    for marker in REQUIRED_MODULE_MARKERS {
        assert!(
            source.contains(marker),
            "expected root shell to contain module marker `{marker}`"
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
