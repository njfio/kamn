use super::super::{ChannelSnapshotStore, ChannelSnapshotStoreError, FileChannelSnapshotStore};
use super::support::{
    assert_roundtrip_within_budget, benchmark_group_store, file_store, group_store,
    remove_channel_snapshot_artifacts, temp_channel_snapshot_journal_path,
    temp_channel_snapshot_path, write_corrupt_channel_journal_tail, write_stale_channel_snapshot,
};
use std::fs;

#[test]
fn integration_file_channel_snapshot_store_roundtrips_snapshot() {
    let path = temp_channel_snapshot_path("roundtrip");
    let journal_path = temp_channel_snapshot_journal_path(&path);
    remove_channel_snapshot_artifacts(&path, &journal_path);
    let snapshot = group_store(
        "channel:group:snapshot-4",
        "kamn:did:agent:owner",
        "kamn:did:agent:member-1",
    )
    .export_snapshot();
    let mut store = file_store(path.clone());
    store.write(snapshot.clone()).expect("write should succeed");
    assert_eq!(
        store.read_latest().expect("read should succeed"),
        Some(snapshot)
    );
    remove_channel_snapshot_artifacts(&path, &journal_path);
}

#[test]
fn integration_file_channel_snapshot_store_replays_journal_when_snapshot_is_stale() {
    let (path, journal_path, mut file_store, mut store_state) = setup_journal_replay_case();
    let first_snapshot = store_state.export_snapshot();
    let second_snapshot = write_updated_snapshot(&mut file_store, &mut store_state);
    write_stale_channel_snapshot(&path, &first_snapshot);
    assert_eq!(
        file_store.read_latest().expect("journal replay should win"),
        Some(second_snapshot)
    );
    remove_channel_snapshot_artifacts(&path, &journal_path);
}

#[test]
fn regression_file_channel_snapshot_store_rejects_malformed_payload() {
    let path = temp_channel_snapshot_path("malformed");
    let journal_path = temp_channel_snapshot_journal_path(&path);
    remove_channel_snapshot_artifacts(&path, &journal_path);
    assert!(fs::write(&path, "schema|1\nrecord|broken\n").is_ok());
    let file_store = FileChannelSnapshotStore::new(path.clone()).expect("store");
    assert_eq!(
        file_store.read_latest(),
        Err(ChannelSnapshotStoreError::InvalidPayload(
            "record|broken".to_owned()
        ))
    );
    remove_channel_snapshot_artifacts(&path, &journal_path);
}

#[test]
fn functional_file_channel_snapshot_store_recovery_repairs_corrupt_payload() {
    let path = temp_channel_snapshot_path("recover");
    let journal_path = temp_channel_snapshot_journal_path(&path);
    remove_channel_snapshot_artifacts(&path, &journal_path);
    assert!(fs::write(&path, "schema|1\nrecord|broken\n").is_ok());
    let mut file_store = FileChannelSnapshotStore::new(path.clone()).expect("store");
    let recovery = file_store
        .recover_latest_and_repair()
        .expect("recovery should succeed");
    assert!(recovery.latest.is_none());
    assert!(recovery.repaired);
    assert_eq!(
        fs::read_to_string(&path).expect("expected test fixture operation to succeed"),
        ""
    );
    remove_channel_snapshot_artifacts(&path, &journal_path);
}

#[test]
fn regression_file_channel_snapshot_store_rejects_corrupt_journal_tail() {
    let path = temp_channel_snapshot_path("corrupt-journal-tail");
    let journal_path = temp_channel_snapshot_journal_path(&path);
    remove_channel_snapshot_artifacts(&path, &journal_path);
    let snapshot = group_store(
        "channel:group:journal-tail",
        "kamn:did:agent:owner",
        "kamn:did:agent:member-1",
    )
    .export_snapshot();
    let mut file_store = file_store(path.clone());
    file_store.write(snapshot).expect("write should succeed");
    write_corrupt_channel_journal_tail(&journal_path);
    assert_eq!(
        file_store.recover_latest_and_repair(),
        Err(ChannelSnapshotStoreError::InvalidPayload(
            "channel_snapshot_journal_corrupt_tail:2".to_owned()
        ))
    );
    remove_channel_snapshot_artifacts(&path, &journal_path);
}

#[test]
fn performance_channel_snapshot_roundtrip_stays_within_ci_budget() {
    let snapshot = benchmark_group_store("channel:group:perf", 256).export_snapshot();
    assert_roundtrip_within_budget(snapshot, 300);
}

#[test]
#[ignore = "scheduled channel snapshot deep lane"]
fn performance_channel_snapshot_deep_lane_stress() {
    let snapshot = benchmark_group_store("channel:group:deep", 6000).export_snapshot();
    let mut restored = super::super::ChannelStore::new();
    restored
        .restore_snapshot(snapshot)
        .expect("snapshot restore should succeed");
}

fn setup_journal_replay_case() -> (
    std::path::PathBuf,
    std::path::PathBuf,
    FileChannelSnapshotStore,
    super::super::ChannelStore,
) {
    let path = temp_channel_snapshot_path("journal-replay");
    let journal_path = temp_channel_snapshot_journal_path(&path);
    remove_channel_snapshot_artifacts(&path, &journal_path);
    let store_state = group_store(
        "channel:group:journal-1",
        "kamn:did:agent:owner",
        "kamn:did:agent:member-1",
    );
    let file_store = file_store(path.clone());
    (path, journal_path, file_store, store_state)
}

fn write_updated_snapshot(
    file_store: &mut FileChannelSnapshotStore,
    store_state: &mut super::super::ChannelStore,
) -> super::super::ChannelSnapshot {
    let first_snapshot = store_state.export_snapshot();
    file_store
        .write(first_snapshot)
        .expect("write should succeed");
    store_state
        .invite_member(
            "channel:group:journal-1",
            "kamn:did:agent:owner",
            "kamn:did:agent:member-2",
        )
        .expect("invite should succeed");
    let second_snapshot = store_state.export_snapshot();
    file_store
        .write(second_snapshot.clone())
        .expect("second write should succeed");
    second_snapshot
}
