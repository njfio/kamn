use crate::message_lifecycle::snapshot_codec::{
    message_lifecycle_snapshot_journal_corrupt_tail, parse_message_lifecycle_snapshot_payload,
};
use crate::message_lifecycle::{
    MessageLifecycleSnapshot, MessageLifecycleSnapshotStoreError, MessageLifecycleStore,
};
use kamn_snapshot_journal::{
    append_snapshot_journal_record, decode_snapshot_journal_hex, default_snapshot_journal_path,
    parse_snapshot_journal_record,
};
use std::fs;
use std::path::{Path, PathBuf};

pub(super) fn message_lifecycle_snapshot_journal_path(path: &Path) -> PathBuf {
    default_snapshot_journal_path(path)
}

pub(super) fn read_message_lifecycle_snapshot_file(
    path: &Path,
) -> Result<Option<MessageLifecycleSnapshot>, MessageLifecycleSnapshotStoreError> {
    if !path.exists() {
        return Ok(None);
    }
    let payload = fs::read_to_string(path)
        .map_err(|error| MessageLifecycleSnapshotStoreError::Io(error.to_string()))?;
    if payload.trim().is_empty() {
        return Ok(None);
    }
    let snapshot = parse_message_lifecycle_snapshot_payload(&payload)?;
    verify_snapshot(snapshot.clone())?;
    Ok(Some(snapshot))
}

pub(super) fn append_message_lifecycle_snapshot_journal_record(
    journal_path: &Path,
    payload: &str,
) -> Result<(), MessageLifecycleSnapshotStoreError> {
    append_snapshot_journal_record(journal_path, payload)
        .map_err(|error| MessageLifecycleSnapshotStoreError::Io(error.to_string()))
}

pub(super) fn replay_message_lifecycle_snapshot_journal(
    journal_path: &Path,
) -> Result<Option<MessageLifecycleSnapshot>, MessageLifecycleSnapshotStoreError> {
    if !journal_path.exists() {
        return Ok(None);
    }
    let payload = fs::read_to_string(journal_path)
        .map_err(|error| MessageLifecycleSnapshotStoreError::Io(error.to_string()))?;
    let mut latest = None;
    for (index, line) in payload.lines().enumerate() {
        if line.trim().is_empty() {
            continue;
        }
        latest = Some(parse_journal_snapshot(line, index + 1)?);
    }
    Ok(latest)
}

fn parse_journal_snapshot(
    line: &str,
    index: usize,
) -> Result<MessageLifecycleSnapshot, MessageLifecycleSnapshotStoreError> {
    let payload_hex = parse_snapshot_journal_record(line)
        .ok_or_else(|| message_lifecycle_snapshot_journal_corrupt_tail(index))?;
    let payload_bytes = decode_snapshot_journal_hex(&payload_hex)
        .ok_or_else(|| message_lifecycle_snapshot_journal_corrupt_tail(index))?;
    let payload = String::from_utf8(payload_bytes)
        .map_err(|_| message_lifecycle_snapshot_journal_corrupt_tail(index))?;
    let snapshot = parse_message_lifecycle_snapshot_payload(&payload)
        .map_err(|_| message_lifecycle_snapshot_journal_corrupt_tail(index))?;
    verify_snapshot(snapshot.clone())
        .map_err(|_| message_lifecycle_snapshot_journal_corrupt_tail(index))?;
    Ok(snapshot)
}

fn verify_snapshot(
    snapshot: MessageLifecycleSnapshot,
) -> Result<(), MessageLifecycleSnapshotStoreError> {
    let mut verifier = MessageLifecycleStore::new();
    verifier
        .restore_snapshot(snapshot)
        .map_err(MessageLifecycleSnapshotStoreError::Snapshot)
}
