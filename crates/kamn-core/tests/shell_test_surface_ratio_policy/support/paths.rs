use crate::support::constants::{REASON_CODES_CSV, REASON_TAXONOMY_VERSION};
use std::fs;
use std::path::{Path, PathBuf};

pub(crate) fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

pub(crate) fn repo_path(relative: &str) -> PathBuf {
    repo_root().join(relative)
}

pub(crate) fn fail(reason_code: &str, detail: &str) -> ! {
    panic!(
        "reason_taxonomy_version={} reason_codes_csv={} reason_code={} detail={}",
        REASON_TAXONOMY_VERSION, REASON_CODES_CSV, reason_code, detail
    );
}

pub(crate) fn read_file(path: &Path, reason_code: &str) -> String {
    fs::read_to_string(path)
        .unwrap_or_else(|error| fail(reason_code, &format!("{}: {}", path.display(), error)))
}
