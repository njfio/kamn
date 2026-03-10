pub(super) use super::super::{
    snapshot_codec::serialize_channel_snapshot, ChannelMetadata, ChannelModelError,
    ChannelRecordSnapshot, ChannelSnapshot, ChannelSnapshotError, ChannelSnapshotStore,
    ChannelSnapshotStoreError, ChannelStore, ChannelType, FileChannelSnapshotStore,
};
pub(super) use std::fs;
pub(super) use std::fs::OpenOptions;
pub(super) use std::io::Write;
pub(super) use std::path::PathBuf;
pub(super) use std::time::{Instant, SystemTime, UNIX_EPOCH};

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
