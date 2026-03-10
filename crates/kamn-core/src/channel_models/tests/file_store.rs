use super::support::*;
#[test]
fn integration_file_channel_snapshot_store_roundtrips_snapshot() {
    let path = temp_channel_snapshot_path("roundtrip");
    let journal_path = temp_channel_snapshot_journal_path(&path);
    let _ = fs::remove_file(&path);
    let _ = fs::remove_file(&journal_path);

    let mut store = ChannelStore::new();
    store
        .create_group(
            "channel:group:snapshot-4",
            "kamn:did:agent:owner",
            vec![
                "kamn:did:agent:owner".to_owned(),
                "kamn:did:agent:member-1".to_owned(),
            ],
            vec!["kamn:did:agent:owner".to_owned()],
        )
        .expect("group should be created");
    let snapshot = store.export_snapshot();

    let mut file_store = FileChannelSnapshotStore::new(path.clone()).expect("store");
    file_store
        .write(snapshot.clone())
        .expect("write should succeed");
    assert_eq!(
        file_store.read_latest().expect("read should succeed"),
        Some(snapshot)
    );

    let _ = fs::remove_file(path);
    let _ = fs::remove_file(journal_path);
}

#[test]
fn integration_file_channel_snapshot_store_replays_journal_when_snapshot_is_stale() {
    let path = temp_channel_snapshot_path("journal-replay");
    let journal_path = temp_channel_snapshot_journal_path(&path);
    let _ = fs::remove_file(&path);
    let _ = fs::remove_file(&journal_path);

    let mut store = ChannelStore::new();
    store
        .create_group(
            "channel:group:journal-1",
            "kamn:did:agent:owner",
            vec![
                "kamn:did:agent:owner".to_owned(),
                "kamn:did:agent:member-1".to_owned(),
            ],
            vec!["kamn:did:agent:owner".to_owned()],
        )
        .expect("group should be created");
    let first_snapshot = store.export_snapshot();

    let mut file_store = FileChannelSnapshotStore::new(path.clone()).expect("store");
    file_store
        .write(first_snapshot.clone())
        .expect("write should succeed");

    store
        .invite_member(
            "channel:group:journal-1",
            "kamn:did:agent:owner",
            "kamn:did:agent:member-2",
        )
        .expect("invite should succeed");
    let second_snapshot = store.export_snapshot();
    file_store
        .write(second_snapshot.clone())
        .expect("second write should succeed");

    let stale_payload =
        serialize_channel_snapshot(&first_snapshot).expect("first snapshot should serialize");
    assert!(fs::write(&path, stale_payload).is_ok());
    assert_eq!(
        file_store.read_latest().expect("journal replay should win"),
        Some(second_snapshot)
    );

    let _ = fs::remove_file(path);
    let _ = fs::remove_file(journal_path);
}

#[test]
fn regression_file_channel_snapshot_store_rejects_malformed_payload() {
    // Regression: #617
    let path = temp_channel_snapshot_path("malformed");
    let _ = fs::remove_file(&path);
    assert!(fs::write(&path, "schema|1\nrecord|broken\n").is_ok());

    let file_store = FileChannelSnapshotStore::new(path.clone()).expect("store");
    assert_eq!(
        file_store.read_latest(),
        Err(ChannelSnapshotStoreError::InvalidPayload(
            "record|broken".to_owned()
        ))
    );

    let _ = fs::remove_file(path);
}

#[test]
fn functional_file_channel_snapshot_store_recovery_repairs_corrupt_payload() {
    let path = temp_channel_snapshot_path("recover");
    let journal_path = temp_channel_snapshot_journal_path(&path);
    let _ = fs::remove_file(&path);
    let _ = fs::remove_file(&journal_path);
    assert!(fs::write(&path, "schema|1\nrecord|broken\n").is_ok());

    let mut file_store = FileChannelSnapshotStore::new(path.clone()).expect("store");
    let recovery = file_store
        .recover_latest_and_repair()
        .expect("recovery should succeed");
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
fn regression_file_channel_snapshot_store_rejects_corrupt_journal_tail() {
    // Regression: #2690
    let path = temp_channel_snapshot_path("corrupt-journal-tail");
    let journal_path = temp_channel_snapshot_journal_path(&path);
    let _ = fs::remove_file(&path);
    let _ = fs::remove_file(&journal_path);

    let mut store = ChannelStore::new();
    store
        .create_group(
            "channel:group:journal-tail",
            "kamn:did:agent:owner",
            vec![
                "kamn:did:agent:owner".to_owned(),
                "kamn:did:agent:member-1".to_owned(),
            ],
            vec!["kamn:did:agent:owner".to_owned()],
        )
        .expect("group should be created");
    let snapshot = store.export_snapshot();
    let mut file_store = FileChannelSnapshotStore::new(path.clone()).expect("store");
    file_store.write(snapshot).expect("write should succeed");

    let mut journal = OpenOptions::new()
        .append(true)
        .open(&journal_path)
        .expect("journal should exist");
    assert!(journal.write_all(b"entry|1|deadbeefz\n").is_ok());
    assert_eq!(
        file_store.recover_latest_and_repair(),
        Err(ChannelSnapshotStoreError::InvalidPayload(
            "channel_snapshot_journal_corrupt_tail:2".to_owned()
        ))
    );

    let _ = fs::remove_file(path);
    let _ = fs::remove_file(journal_path);
}
