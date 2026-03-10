use super::fault_io::{append_partial_journal_tail, clear_store_files, journal_path_for, truncate_snapshot_file, write_partial_snapshot};
use super::snapshot_fixtures::channel_snapshots;
use super::{FixtureCase, TempDir};
use kamn_core::{ChannelSnapshotStore, ChannelSnapshotStoreError, FileChannelSnapshotStore};

pub(crate) fn run(case: &FixtureCase, temp_dir: &TempDir) {
    let snapshot_path = temp_dir.path().join(format!("channel-{}.snapshot", case.case_id));
    let journal_path = journal_path_for(&snapshot_path);
    clear_store_files(&snapshot_path, &journal_path);
    match case.fault_mode.as_str() {
        "partial_snapshot_file_write" => run_clean_recovery(case, snapshot_path),
        "partial_journal_tail_write" => run_corrupt_tail(case, snapshot_path, journal_path),
        "partial_snapshot_without_journal" => run_repaired_payload(case, snapshot_path),
        unknown => panic!("unsupported channel fault mode: {unknown}"),
    }
}

fn run_clean_recovery(case: &FixtureCase, snapshot_path: std::path::PathBuf) {
    assert_eq!(case.expected_outcome, "recovery_clean");
    let (first, second) = channel_snapshots();
    let mut store = FileChannelSnapshotStore::new(snapshot_path.clone()).expect("store should build");
    store.write(first).expect("first write should succeed");
    store.write(second.clone()).expect("second write should succeed");
    truncate_snapshot_file(&snapshot_path);
    assert_eq!(store.read_latest().expect("journal should remain authoritative"), Some(second.clone()));
    let recovery = store.recover_latest_and_repair().expect("recovery should stay clean with valid journal commit");
    assert!(!recovery.repaired);
    assert_eq!(recovery.reason_code(), case.expected_marker);
    assert_eq!(recovery.latest, Some(second));
}

fn run_corrupt_tail(case: &FixtureCase, snapshot_path: std::path::PathBuf, journal_path: std::path::PathBuf) {
    assert_eq!(case.expected_outcome, "fail_closed_corrupt_tail");
    let (_, second) = channel_snapshots();
    let mut store = FileChannelSnapshotStore::new(snapshot_path).expect("store should build");
    store.write(second).expect("snapshot write should create first journal entry");
    append_partial_journal_tail(&journal_path);
    match store.recover_latest_and_repair() {
        Err(ChannelSnapshotStoreError::InvalidPayload(value)) => {
            assert_eq!(value, format!("{}:2", case.expected_marker));
        }
        other => panic!("expected channel corrupt-tail failure, got {other:?}"),
    }
}

fn run_repaired_payload(case: &FixtureCase, snapshot_path: std::path::PathBuf) {
    assert_eq!(case.expected_outcome, "recovery_repaired_corrupt_payload");
    write_partial_snapshot(&snapshot_path);
    let mut store = FileChannelSnapshotStore::new(snapshot_path).expect("store should build");
    let recovery = store
        .recover_latest_and_repair()
        .expect("recovery should repair malformed payload when journal is absent");
    assert!(recovery.repaired);
    assert!(recovery.latest.is_none());
    assert_eq!(recovery.reason_code(), case.expected_marker);
}
