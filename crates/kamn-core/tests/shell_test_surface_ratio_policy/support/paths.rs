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
        "reason_taxonomy_version={REASON_TAXONOMY_VERSION} reason_codes_csv={REASON_CODES_CSV} reason_code={reason_code} detail={detail}"
    );
}

pub(crate) fn read_file(path: &Path, reason_code: &str) -> String {
    fs::read_to_string(path).unwrap_or_else(|error| {
        let path_display = path.display();
        fail(reason_code, &format!("{path_display}: {error}"))
    })
}
