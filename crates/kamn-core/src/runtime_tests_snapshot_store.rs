use super::super::{
    FileRuntimeSnapshotStore, InMemoryRuntimeSnapshotStore, RuntimeSnapshot, RuntimeSnapshotStore,
    SnapshotRestoreError, SnapshotRestoreGuard, SnapshotStoreError,
};
use std::fs;
use std::path::PathBuf;
use std::time::{Instant, SystemTime, UNIX_EPOCH};

#[test]
fn functional_snapshot_restore_guard_accepts_matching_snapshot() {
    let guard = SnapshotRestoreGuard::new(42, "state-42").expect("restore guard should construct");
    let snapshot = RuntimeSnapshot::new(42, "state-42").expect("snapshot should be valid");
    assert!(guard.validate(snapshot).is_ok());
}

#[test]
fn unit_snapshot_restore_guard_rejects_invalid_state_hash() {
    let snapshot = RuntimeSnapshot::new(42, "");
    assert_eq!(snapshot, Err(SnapshotRestoreError::InvalidStateHash));
}

#[test]
fn regression_snapshot_restore_version_mismatch_is_rejected() {
    // Regression: #361
    let guard = SnapshotRestoreGuard::new(42, "state-42").expect("restore guard should construct");
    let snapshot = RuntimeSnapshot::new(41, "state-42").expect("snapshot should be valid");
    let error = guard
        .validate(snapshot)
        .expect_err("version mismatch should be rejected");
    assert_eq!(
        error,
        SnapshotRestoreError::StateVersionMismatch {
            expected: 42,
            found: 41
        }
    );
}

#[test]
fn regression_snapshot_restore_hash_mismatch_is_rejected() {
    // Regression: #361
    let guard = SnapshotRestoreGuard::new(42, "state-42").expect("restore guard should construct");
    let snapshot = RuntimeSnapshot::new(42, "state-41").expect("snapshot should be valid");
    let error = guard
        .validate(snapshot)
        .expect_err("hash mismatch should be rejected");
    assert_eq!(
        error,
        SnapshotRestoreError::StateHashMismatch {
            expected: "state-42".to_owned(),
            found: "state-41".to_owned()
        }
    );
}

#[test]
fn functional_snapshot_restore_guard_with_expected_cursor_accepts_matching_snapshot() {
    let guard = SnapshotRestoreGuard::with_expected_cursor(42, "state-42", 100)
        .expect("restore guard should construct");
    let snapshot =
        RuntimeSnapshot::with_cursor(42, "state-42", 100).expect("snapshot should be valid");
    assert!(guard.validate(snapshot).is_ok());
}

#[test]
fn regression_snapshot_restore_cursor_mismatch_is_rejected() {
    // Regression: #617
    let guard = SnapshotRestoreGuard::with_expected_cursor(42, "state-42", 100)
        .expect("restore guard should construct");
    let snapshot =
        RuntimeSnapshot::with_cursor(42, "state-42", 99).expect("snapshot should be valid");
    let error = guard
        .validate(snapshot)
        .expect_err("cursor mismatch should be rejected");
    assert_eq!(
        error,
        SnapshotRestoreError::CursorMismatch {
            expected: 100,
            found: 99
        }
    );
}

#[test]
fn functional_in_memory_snapshot_store_round_trips_snapshots() {
    let mut store = InMemoryRuntimeSnapshotStore::default();
    assert!(store.list().expect("list should succeed").is_empty());

    let snapshot_1 = RuntimeSnapshot::new(41, "state-41").expect("valid snapshot");
    let snapshot_2 = RuntimeSnapshot::new(42, "state-42").expect("valid snapshot");
    assert!(store.write(snapshot_1).is_ok());
    assert!(store.write(snapshot_2.clone()).is_ok());

    let latest = store.read_latest().expect("read_latest should succeed");
    assert_eq!(latest, Some(snapshot_2));
    assert_eq!(store.list().expect("list should succeed").len(), 2);
}

#[test]
fn integration_file_snapshot_store_round_trips_snapshots() {
    let path = temp_snapshot_store_path("roundtrip");
    let _ = fs::remove_file(&path);

    let mut store = FileRuntimeSnapshotStore::new(path.clone()).expect("store should build");
    let snapshot_1 = RuntimeSnapshot::new(41, "state-41").expect("valid snapshot");
    let snapshot_2 = RuntimeSnapshot::new(42, "state-42").expect("valid snapshot");
    assert!(store.write(snapshot_1).is_ok());
    assert!(store.write(snapshot_2.clone()).is_ok());

    let latest = store.read_latest().expect("read_latest should succeed");
    assert_eq!(latest, Some(snapshot_2));
    assert_eq!(store.list().expect("list should succeed").len(), 2);

    let _ = fs::remove_file(path);
}

#[test]
fn regression_file_snapshot_store_rejects_malformed_payload() {
    // Regression: #387
    let path = temp_snapshot_store_path("malformed");
    let _ = fs::remove_file(&path);
    assert!(fs::write(&path, "not-a-valid-snapshot-line\n").is_ok());

    let store = FileRuntimeSnapshotStore::new(path.clone()).expect("store should build");
    let error = store
        .list()
        .expect_err("malformed payload must be rejected");
    assert_eq!(
        error,
        SnapshotStoreError::InvalidPayload("not-a-valid-snapshot-line".to_owned())
    );

    let _ = fs::remove_file(path);
}

#[test]
fn unit_file_snapshot_store_recovery_handles_missing_snapshot_file() {
    let path = temp_snapshot_store_path("recover-missing");
    let _ = fs::remove_file(&path);

    let mut store = FileRuntimeSnapshotStore::new(path.clone()).expect("store should build");
    let result = store
        .recover_latest_and_repair()
        .expect("recovery should pass");
    assert!(result.latest.is_none());
    assert_eq!(result.recovered_entries, 0);
    assert_eq!(result.dropped_corrupt_entries, 0);

    let _ = fs::remove_file(path);
}

#[test]
fn functional_file_snapshot_store_recovery_recovers_latest_after_trailing_corruption() {
    let path = temp_snapshot_store_path("recover-trailing-corruption");
    let _ = fs::remove_file(&path);
    assert!(fs::write(&path, "41|state-41\n42|state-42\n43|\n").is_ok());

    let mut store = FileRuntimeSnapshotStore::new(path.clone()).expect("store should build");
    assert_eq!(
        store.list(),
        Err(SnapshotStoreError::InvalidPayload("43|".to_owned()))
    );

    let result = store
        .recover_latest_and_repair()
        .expect("recovery should pass");
    assert_eq!(
        result.latest,
        Some(RuntimeSnapshot::new(42, "state-42").expect("valid snapshot"))
    );
    assert_eq!(result.recovered_entries, 2);
    assert_eq!(result.dropped_corrupt_entries, 1);
    assert_eq!(store.list().expect("list should succeed").len(), 2);

    let _ = fs::remove_file(path);
}

#[test]
fn integration_file_snapshot_store_recovery_allows_append_after_restart() {
    let path = temp_snapshot_store_path("recover-restart");
    let _ = fs::remove_file(&path);
    assert!(fs::write(&path, "41|state-41\n42|state-42\n43|\n").is_ok());

    let mut first_store = FileRuntimeSnapshotStore::new(path.clone()).expect("store should build");
    let first_recovery = first_store
        .recover_latest_and_repair()
        .expect("recovery should pass");
    assert_eq!(first_recovery.recovered_entries, 2);

    let mut restarted_store =
        FileRuntimeSnapshotStore::new(path.clone()).expect("store should build");
    let next_snapshot = RuntimeSnapshot::new(43, "state-43").expect("valid snapshot");
    assert!(restarted_store.write(next_snapshot.clone()).is_ok());
    assert_eq!(
        restarted_store.read_latest().expect("read should pass"),
        Some(next_snapshot)
    );
    assert_eq!(restarted_store.list().expect("list should pass").len(), 3);

    let _ = fs::remove_file(path);
}

#[test]
fn regression_file_snapshot_store_recovery_truncates_corrupt_suffix() {
    // Regression: #617
    let path = temp_snapshot_store_path("recover-corrupt-suffix");
    let _ = fs::remove_file(&path);
    assert!(fs::write(&path, "41|state-41\n42|state-42\nbroken\n43|state-43\n").is_ok());

    let mut store = FileRuntimeSnapshotStore::new(path.clone()).expect("store should build");
    let result = store
        .recover_latest_and_repair()
        .expect("recovery should pass");
    assert_eq!(
        result.latest,
        Some(RuntimeSnapshot::new(42, "state-42").expect("valid snapshot"))
    );
    assert_eq!(result.recovered_entries, 2);
    assert_eq!(result.dropped_corrupt_entries, 2);
    assert_eq!(
        fs::read_to_string(&path).expect("snapshot file should be readable"),
        "41|state-41|41\n42|state-42|42\n"
    );

    let _ = fs::remove_file(path);
}

#[test]
fn unit_runtime_snapshot_with_cursor_rejects_zero_cursor() {
    let snapshot = RuntimeSnapshot::with_cursor(42, "state-42", 0);
    assert_eq!(snapshot, Err(SnapshotRestoreError::InvalidCursor));
}

#[test]
fn unit_runtime_snapshot_rejects_hash_with_metadata_delimiter() {
    let snapshot = RuntimeSnapshot::new(42, "state-42|100");
    assert_eq!(snapshot, Err(SnapshotRestoreError::InvalidStateHash));
}

#[test]
fn unit_in_memory_snapshot_store_rejects_state_version_regression() {
    let mut store = InMemoryRuntimeSnapshotStore::default();
    let baseline = RuntimeSnapshot::with_cursor(41, "state-41", 100).expect("valid snapshot");
    assert!(store.write(baseline).is_ok());
    let stale = RuntimeSnapshot::with_cursor(40, "state-40", 101).expect("valid snapshot");
    assert_eq!(
        store.write(stale),
        Err(SnapshotStoreError::StateVersionRegression {
            previous: 41,
            found: 40
        })
    );
}

#[test]
fn unit_in_memory_snapshot_store_rejects_cursor_regression() {
    let mut store = InMemoryRuntimeSnapshotStore::default();
    let baseline = RuntimeSnapshot::with_cursor(41, "state-41", 100).expect("valid snapshot");
    assert!(store.write(baseline).is_ok());
    let stale = RuntimeSnapshot::with_cursor(42, "state-42", 99).expect("valid snapshot");
    assert_eq!(
        store.write(stale),
        Err(SnapshotStoreError::CursorRegression {
            previous: 100,
            found: 99
        })
    );
}

#[test]
fn regression_file_snapshot_store_rejects_version_regression_metadata() {
    // Regression: #617
    let path = temp_snapshot_store_path("version-regression");
    let _ = fs::remove_file(&path);
    assert!(fs::write(&path, "41|state-41|100\n40|state-40|101\n").is_ok());

    let store = FileRuntimeSnapshotStore::new(path.clone()).expect("store should build");
    assert_eq!(
        store.list(),
        Err(SnapshotStoreError::StateVersionRegression {
            previous: 41,
            found: 40
        })
    );

    let _ = fs::remove_file(path);
}

#[test]
fn regression_file_snapshot_store_rejects_cursor_regression_metadata() {
    // Regression: #617
    let path = temp_snapshot_store_path("cursor-regression");
    let _ = fs::remove_file(&path);
    assert!(fs::write(&path, "41|state-41|100\n42|state-42|99\n").is_ok());

    let store = FileRuntimeSnapshotStore::new(path.clone()).expect("store should build");
    assert_eq!(
        store.list(),
        Err(SnapshotStoreError::CursorRegression {
            previous: 100,
            found: 99
        })
    );

    let _ = fs::remove_file(path);
}

#[test]
fn regression_file_snapshot_store_rejects_stale_hash_metadata() {
    // Regression: #617
    let path = temp_snapshot_store_path("hash-regression");
    let _ = fs::remove_file(&path);
    assert!(fs::write(&path, "41|state-41|100\n42|state-41|101\n").is_ok());

    let store = FileRuntimeSnapshotStore::new(path.clone()).expect("store should build");
    assert_eq!(
        store.list(),
        Err(SnapshotStoreError::StaleStateHash {
            state_hash: "state-41".to_owned(),
            previous_version: 41,
            found_version: 42
        })
    );

    let _ = fs::remove_file(path);
}

#[test]
fn functional_file_snapshot_store_recovery_truncates_stale_metadata_suffix() {
    let path = temp_snapshot_store_path("recover-stale-metadata");
    let _ = fs::remove_file(&path);
    assert!(fs::write(&path, "41|state-41|100\n42|state-42|99\n43|state-43|102\n").is_ok());

    let mut store = FileRuntimeSnapshotStore::new(path.clone()).expect("store should build");
    let result = store
        .recover_latest_and_repair()
        .expect("recovery should pass");
    assert_eq!(
        result.latest,
        Some(RuntimeSnapshot::with_cursor(41, "state-41", 100).expect("valid snapshot"))
    );
    assert_eq!(result.recovered_entries, 1);
    assert_eq!(result.dropped_corrupt_entries, 2);
    assert_eq!(
        fs::read_to_string(&path).expect("snapshot file should be readable"),
        "41|state-41|100\n"
    );

    let _ = fs::remove_file(path);
}

#[test]
fn performance_file_snapshot_store_recovery_scan_stays_within_ci_budget() {
    let path = temp_snapshot_store_path("recover-performance");
    let _ = fs::remove_file(&path);
    let mut payload = String::new();
    for state_version in 1..=256 {
        payload.push_str(&format!("{state_version}|state-{state_version}\n"));
    }
    payload.push_str("broken\n");
    assert!(fs::write(&path, payload).is_ok());

    let mut store = FileRuntimeSnapshotStore::new(path.clone()).expect("store should build");
    let start = Instant::now();
    let result = store
        .recover_latest_and_repair()
        .expect("recovery should pass");
    let elapsed_millis = start.elapsed().as_millis();
    assert_eq!(result.recovered_entries, 256);
    assert_eq!(result.dropped_corrupt_entries, 1);
    assert!(
        elapsed_millis < 250,
        "snapshot recovery exceeded CI budget: {elapsed_millis}ms"
    );

    let _ = fs::remove_file(path);
}

#[test]
#[ignore = "scheduled snapshot deep lane"]
fn performance_file_snapshot_store_recovery_deep_lane_large_payload() {
    let path = temp_snapshot_store_path("recover-deep-lane");
    let _ = fs::remove_file(&path);
    let mut payload = String::new();
    for state_version in 1..=8192 {
        payload.push_str(&format!(
            "{state_version}|state-{state_version}|{state_version}\n"
        ));
    }
    payload.push_str("8193|state-8193|0\n");
    assert!(fs::write(&path, payload).is_ok());

    let mut store = FileRuntimeSnapshotStore::new(path.clone()).expect("store should build");
    let start = Instant::now();
    let result = store
        .recover_latest_and_repair()
        .expect("recovery should pass");
    let elapsed_millis = start.elapsed().as_millis();
    assert_eq!(result.recovered_entries, 8192);
    assert_eq!(result.dropped_corrupt_entries, 1);
    assert!(
        elapsed_millis < 2000,
        "snapshot deep-lane recovery exceeded budget: {elapsed_millis}ms"
    );

    let _ = fs::remove_file(path);
}

fn temp_snapshot_store_path(tag: &str) -> PathBuf {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock should be monotonic")
        .as_nanos();
    std::env::temp_dir().join(format!("kamn-runtime-snapshot-{tag}-{nonce}.log"))
}
