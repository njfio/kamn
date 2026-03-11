use super::super::{
    snapshot_codec::serialize_channel_snapshot, ChannelSnapshot, ChannelStore,
    FileChannelSnapshotStore,
};
use std::fs;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::PathBuf;
use std::time::{Instant, SystemTime, UNIX_EPOCH};

pub(super) fn temp_channel_snapshot_path(tag: &str) -> PathBuf {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock should be monotonic")
        .as_nanos();
    std::env::temp_dir().join(format!("kamn-channel-snapshot-{tag}-{nonce}.log"))
}

pub(super) fn temp_channel_snapshot_journal_path(path: &std::path::Path) -> PathBuf {
    let mut journal = path.as_os_str().to_os_string();
    journal.push(".journal");
    PathBuf::from(journal)
}

pub(super) fn remove_channel_snapshot_artifacts(path: &PathBuf, journal_path: &PathBuf) {
    let _ = fs::remove_file(path);
    let _ = fs::remove_file(journal_path);
}

pub(super) fn group_store(channel_id: &str, owner: &str, member: &str) -> ChannelStore {
    let mut store = ChannelStore::new();
    store
        .create_group(
            channel_id,
            owner,
            vec![owner.to_owned(), member.to_owned()],
            vec![owner.to_owned()],
        )
        .expect("group should be created");
    store
}

pub(super) fn write_stale_channel_snapshot(path: &PathBuf, snapshot: &ChannelSnapshot) {
    let stale_payload = serialize_channel_snapshot(snapshot).expect("snapshot should serialize");
    assert!(fs::write(path, stale_payload).is_ok());
}

pub(super) fn write_corrupt_channel_journal_tail(journal_path: &PathBuf) {
    let mut journal = OpenOptions::new()
        .append(true)
        .open(journal_path)
        .expect("journal should exist");
    assert!(journal.write_all(b"entry|1|deadbeefz\n").is_ok());
}

pub(super) fn benchmark_group_store(prefix: &str, count: usize) -> ChannelStore {
    let mut store = ChannelStore::new();
    for index in 0..count {
        store
            .create_group(
                &format!("{prefix}-{index}"),
                "kamn:did:agent:owner",
                vec![
                    "kamn:did:agent:owner".to_owned(),
                    format!("kamn:did:agent:member-{index}"),
                ],
                vec!["kamn:did:agent:owner".to_owned()],
            )
            .expect("group should be created");
    }
    store
}

pub(super) fn assert_roundtrip_within_budget(snapshot: ChannelSnapshot, budget_ms: u128) {
    let mut restored = ChannelStore::new();
    let start = Instant::now();
    restored
        .restore_snapshot(snapshot)
        .expect("snapshot restore should succeed");
    let elapsed_millis = start.elapsed().as_millis();
    assert!(
        elapsed_millis < budget_ms,
        "channel snapshot roundtrip exceeded CI budget: {elapsed_millis}ms"
    );
}

pub(super) fn file_store(path: PathBuf) -> FileChannelSnapshotStore {
    FileChannelSnapshotStore::new(path).expect("store should build")
}
