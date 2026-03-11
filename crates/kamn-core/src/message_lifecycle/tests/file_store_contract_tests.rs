use super::support::{
    register_message, temp_message_lifecycle_snapshot_journal_path,
    temp_message_lifecycle_snapshot_path,
};
use super::*;
use std::fs;
use std::fs::OpenOptions;
use std::io::Write;

#[test]
fn integration_file_message_lifecycle_snapshot_store_roundtrips_snapshot() {
    let path = temp_message_lifecycle_snapshot_path("roundtrip");
    let journal_path = temp_message_lifecycle_snapshot_journal_path(&path);
    let _ = fs::remove_file(&path);
    let _ = fs::remove_file(&journal_path);
    let mut store = MessageLifecycleStore::new();
    register_message(&mut store, "urn:uuid:msg-snapshot-4");
    let snapshot = store.export_snapshot();
    let mut file_store =
        FileMessageLifecycleSnapshotStore::new(path.clone()).expect("store should build");
    assert!(file_store.write(snapshot.clone()).is_ok());
    assert_eq!(
        file_store.read_latest().expect("read should pass"),
        Some(snapshot)
    );
    let _ = fs::remove_file(path);
    let _ = fs::remove_file(journal_path);
}

#[test]
fn integration_file_message_lifecycle_snapshot_store_replays_journal_when_snapshot_is_stale() {
    let path = temp_message_lifecycle_snapshot_path("journal-replay");
    let journal_path = temp_message_lifecycle_snapshot_journal_path(&path);
    let _ = fs::remove_file(&path);
    let _ = fs::remove_file(&journal_path);
    let mut store = MessageLifecycleStore::new();
    register_message(&mut store, "urn:uuid:msg-journal-1");
    let first_snapshot = store.export_snapshot();
    let mut file_store = FileMessageLifecycleSnapshotStore::new(path.clone()).expect("store");
    file_store
        .write(first_snapshot.clone())
        .expect("write should pass");
    store
        .register(
            "urn:uuid:msg-journal-2",
            "kamn:did:agent:sender-2",
            vec!["kamn:did:agent:recipient-2".to_owned()],
            "2026-02-07T21:15:30.123Z",
            "2026-02-07T21:45:30.123Z",
        )
        .expect("second register should succeed");
    let second_snapshot = store.export_snapshot();
    file_store
        .write(second_snapshot.clone())
        .expect("second write should pass");
    let stale_payload = serialize_message_lifecycle_snapshot(&first_snapshot)
        .expect("first snapshot should serialize");
    assert!(fs::write(&path, stale_payload).is_ok());
    assert_eq!(
        file_store.read_latest().expect("journal replay should win"),
        Some(second_snapshot)
    );
    let _ = fs::remove_file(path);
    let _ = fs::remove_file(journal_path);
}

#[test]
fn regression_file_message_lifecycle_snapshot_store_rejects_malformed_payload() {
    let path = temp_message_lifecycle_snapshot_path("malformed");
    let _ = fs::remove_file(&path);
    assert!(fs::write(&path, "schema|1\nrecord|broken\n").is_ok());
    let file_store = FileMessageLifecycleSnapshotStore::new(path.clone()).expect("store");
    assert_eq!(
        file_store.read_latest(),
        Err(MessageLifecycleSnapshotStoreError::InvalidPayload(
            "record|broken".to_owned()
        ))
    );
    let _ = fs::remove_file(path);
}

#[test]
fn functional_file_message_lifecycle_snapshot_store_recovery_repairs_corrupt_payload() {
    let path = temp_message_lifecycle_snapshot_path("recover");
    let journal_path = temp_message_lifecycle_snapshot_journal_path(&path);
    let _ = fs::remove_file(&path);
    let _ = fs::remove_file(&journal_path);
    assert!(fs::write(&path, "schema|1\nrecord|broken\n").is_ok());
    let mut file_store = FileMessageLifecycleSnapshotStore::new(path.clone()).expect("store");
    let recovery = file_store
        .recover_latest_and_repair()
        .expect("recovery should pass");
    assert!(recovery.latest.is_none());
    assert!(recovery.repaired);
    assert_eq!(
        fs::read_to_string(&path).expect("repaired file should be readable"),
        ""
    );
    let _ = fs::remove_file(path);
    let _ = fs::remove_file(journal_path);
}

#[test]
fn regression_file_message_lifecycle_snapshot_store_rejects_corrupt_journal_tail() {
    let path = temp_message_lifecycle_snapshot_path("corrupt-journal-tail");
    let journal_path = temp_message_lifecycle_snapshot_journal_path(&path);
    let _ = fs::remove_file(&path);
    let _ = fs::remove_file(&journal_path);
    let mut store = MessageLifecycleStore::new();
    register_message(&mut store, "urn:uuid:msg-tail");
    let snapshot = store.export_snapshot();
    let mut file_store = FileMessageLifecycleSnapshotStore::new(path.clone()).expect("store");
    file_store.write(snapshot).expect("write should pass");
    let mut journal = OpenOptions::new()
        .append(true)
        .open(&journal_path)
        .expect("journal should exist");
    assert!(journal.write_all(b"entry|1|deadbeefz\n").is_ok());
    assert_eq!(
        file_store.recover_latest_and_repair(),
        Err(MessageLifecycleSnapshotStoreError::InvalidPayload(
            "message_lifecycle_snapshot_journal_corrupt_tail:2".to_owned()
        ))
    );
    let _ = fs::remove_file(path);
    let _ = fs::remove_file(journal_path);
}
