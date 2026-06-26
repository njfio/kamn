use std::fs;
use std::path::{Path, PathBuf};

use super::baseline_threshold_support::{REASON_CODES_CSV, REASON_TAXONOMY_VERSION};

pub fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

pub fn repo_path(relative: &str) -> PathBuf {
    repo_root().join(relative)
}

pub fn fail(reason_code: &str, detail: &str) -> ! {
    panic!(
        "reason_taxonomy_version={} reason_codes_csv={} reason_code={} detail={}",
        REASON_TAXONOMY_VERSION, REASON_CODES_CSV, reason_code, detail
    );
}

pub fn read_file(path: &Path, reason_code: &str) -> String {
    fs::read_to_string(path)
        .unwrap_or_else(|error| fail(reason_code, &format!("{}: {}", path.display(), error)))
}

pub fn is_test_only_source_path(relative_path: &str) -> bool {
    if relative_path.starts_with("crates/kamn-e2e-harness/") {
        return true;
    }
    if relative_path
        .split('/')
        .any(|component| component == "main_tests" || component == "runtime_tests")
    {
        return true;
    }
    has_test_only_file_name(relative_path)
}

fn has_test_only_file_name(relative_path: &str) -> bool {
    let Some(file_name) = Path::new(relative_path)
        .file_name()
        .and_then(|value| value.to_str())
    else {
        return false;
    };
    if !file_name.ends_with(".rs") {
        return false;
    }
    let stem = &file_name[..file_name.len().saturating_sub(3)];
    stem == "tests"
        || stem.starts_with("test_")
        || stem.starts_with("runtime_tests")
        || stem.contains("_tests")
}
