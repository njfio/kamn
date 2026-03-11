use kamn_snapshot_journal::{
    append_snapshot_journal_record, decode_snapshot_journal_hex, default_snapshot_journal_path,
    parse_snapshot_journal_record,
};
use std::fs;
use std::path::Path;

use super::errors::ChannelSnapshotStoreError;
use super::store::{
    ChannelMetadata, ChannelRecordSnapshot, ChannelSnapshot, ChannelStore, ChannelType,
};

mod journal;
mod parse;
mod serialize;
mod support;

pub(crate) use journal::{
    append_channel_snapshot_journal_record, channel_snapshot_journal_path,
    channel_snapshot_journal_recovery_error, replay_channel_snapshot_journal,
};
pub(crate) use parse::parse_channel_snapshot_payload;
pub(crate) use serialize::serialize_channel_snapshot;
pub(crate) use support::{
    channel_type_code, ensure_snapshot_token, metadata_snapshot_value, parse_channel_type_code,
    parse_metadata_snapshot_value,
};

pub(crate) fn read_channel_snapshot_file(
    path: &Path,
) -> Result<Option<ChannelSnapshot>, ChannelSnapshotStoreError> {
    if !path.exists() {
        return Ok(None);
    }
    let payload = fs::read_to_string(path)
        .map_err(|error| ChannelSnapshotStoreError::Io(error.to_string()))?;
    if payload.trim().is_empty() {
        return Ok(None);
    }
    let snapshot = parse_channel_snapshot_payload(&payload)?;
    let mut verifier = ChannelStore::new();
    verifier
        .restore_snapshot(snapshot.clone())
        .map_err(ChannelSnapshotStoreError::Snapshot)?;
    Ok(Some(snapshot))
}
