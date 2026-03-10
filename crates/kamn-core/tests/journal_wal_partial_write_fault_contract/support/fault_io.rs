use std::ffi::OsString;
use std::fs;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::{Path, PathBuf};

pub(crate) struct StorePaths {
    pub(crate) snapshot_path: PathBuf,
    pub(crate) journal_path: PathBuf,
}

pub(crate) fn journal_path_for(snapshot_path: &Path) -> PathBuf {
    let mut journal: OsString = snapshot_path.as_os_str().to_os_string();
    journal.push(".journal");
    PathBuf::from(journal)
}

pub(crate) fn append_partial_journal_tail(journal_path: &Path) {
    let mut file = OpenOptions::new()
        .append(true)
        .open(journal_path)
        .expect("journal should be writable for partial-tail fault injection");
    file.write_all(b"entry|1|abc\n")
        .expect("partial journal tail must be appended");
}

pub(crate) fn truncate_snapshot_file(path: &Path) {
    let payload = fs::read_to_string(path).expect("snapshot payload must exist before truncation");
    let truncated_len = (payload.len() / 2).max(1);
    fs::write(path, &payload[..truncated_len]).expect("snapshot payload truncation should succeed");
}

pub(crate) fn write_partial_snapshot(snapshot_path: &Path) {
    fs::write(snapshot_path, "schema|1\nrecord|partial")
        .expect("invalid partial snapshot should be written");
}

pub(crate) fn clear_store_files(snapshot_path: &Path, journal_path: &Path) {
    let _ = fs::remove_file(snapshot_path);
    let _ = fs::remove_file(journal_path);
}

pub(crate) fn prepare_store_paths(
    temp_dir: &crate::support::TempDir,
    prefix: &str,
    case_id: &str,
) -> StorePaths {
    let snapshot_path = temp_dir.path().join(format!("{prefix}-{case_id}.snapshot"));
    let journal_path = journal_path_for(&snapshot_path);
    clear_store_files(&snapshot_path, &journal_path);
    StorePaths {
        snapshot_path,
        journal_path,
    }
}
