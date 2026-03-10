#[test]
fn integration_file_message_lifecycle_snapshot_store_roundtrips_snapshot() {
    let (path, journal_path) = fresh_snapshot_paths("roundtrip");
    let snapshot = registered_snapshot("urn:uuid:msg-snapshot-4");
    let mut file_store = build_file_snapshot_store(&path);
    assert!(file_store.write(snapshot.clone()).is_ok());
    assert_eq!(
        file_store.read_latest().expect("read should pass"),
        Some(snapshot)
    );
    cleanup_snapshot_paths(&path, &journal_path);
}

#[test]
fn integration_file_message_lifecycle_snapshot_store_replays_journal_when_snapshot_is_stale() {
    let (path, journal_path) = fresh_snapshot_paths("journal-replay");
    let first_snapshot = registered_snapshot("urn:uuid:msg-journal-1");
    let second_snapshot = registered_snapshot_pair();
    let mut file_store = build_file_snapshot_store(&path);
    file_store
        .write(first_snapshot.clone())
        .expect("write should pass");
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
    cleanup_snapshot_paths(&path, &journal_path);
}

#[test]
fn regression_file_message_lifecycle_snapshot_store_rejects_malformed_payload() {
    // Regression: #617
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
    // Regression: #2690
    let (path, journal_path) = fresh_snapshot_paths("corrupt-journal-tail");
    let snapshot = registered_snapshot("urn:uuid:msg-tail");
    let mut file_store = build_file_snapshot_store(&path);
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
    cleanup_snapshot_paths(&path, &journal_path);
}

fn fresh_snapshot_paths(tag: &str) -> (PathBuf, PathBuf) {
    let path = temp_message_lifecycle_snapshot_path(tag);
    let journal_path = temp_message_lifecycle_snapshot_journal_path(&path);
    cleanup_snapshot_paths(&path, &journal_path);
    (path, journal_path)
}

fn cleanup_snapshot_paths(path: &Path, journal_path: &Path) {
    let _ = fs::remove_file(path);
    let _ = fs::remove_file(journal_path);
}

fn build_file_snapshot_store(path: &Path) -> FileMessageLifecycleSnapshotStore {
    FileMessageLifecycleSnapshotStore::new(path.to_path_buf()).expect("store should build")
}

fn registered_snapshot(message_id: &str) -> MessageLifecycleSnapshot {
    let mut store = MessageLifecycleStore::new();
    register_default_message(&mut store, message_id);
    store.export_snapshot()
}

fn registered_snapshot_pair() -> MessageLifecycleSnapshot {
    let mut store = MessageLifecycleStore::new();
    register_default_message(&mut store, "urn:uuid:msg-journal-1");
    register_message(
        &mut store,
        "urn:uuid:msg-journal-2",
        "kamn:did:agent:sender-2",
        vec!["kamn:did:agent:recipient-2".to_owned()],
    );
    store.export_snapshot()
}
