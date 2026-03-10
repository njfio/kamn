use super::*;
use kamn_snapshot_journal::{
    append_snapshot_journal_record, decode_snapshot_journal_hex, default_snapshot_journal_path,
    parse_snapshot_journal_record,
};
use std::fs;

pub(super) const MESSAGE_LIFECYCLE_SNAPSHOT_JOURNAL_CORRUPT_TAIL_PREFIX: &str =
    "message_lifecycle_snapshot_journal_corrupt_tail";

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
    let snapshot = codec::parse_message_lifecycle_snapshot_payload(&payload)?;
    let mut verifier = MessageLifecycleStore::new();
    verifier
        .restore_snapshot(snapshot.clone())
        .map_err(MessageLifecycleSnapshotStoreError::Snapshot)?;
    Ok(Some(snapshot))
}

pub(super) fn append_message_lifecycle_snapshot_journal_record(
    journal_path: &Path,
    payload: &str,
) -> Result<(), MessageLifecycleSnapshotStoreError> {
    append_snapshot_journal_record(journal_path, payload)
        .map_err(|error| MessageLifecycleSnapshotStoreError::Io(error.to_string()))?;
    Ok(())
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
        latest = replay_message_lifecycle_snapshot_journal_line(line, index + 1, latest)?;
    }
    Ok(latest)
}

fn replay_message_lifecycle_snapshot_journal_line(
    line: &str,
    index: usize,
    latest: Option<MessageLifecycleSnapshot>,
) -> Result<Option<MessageLifecycleSnapshot>, MessageLifecycleSnapshotStoreError> {
    let trimmed = line.trim();
    if trimmed.is_empty() {
        return Ok(latest);
    }
    let payload = decode_message_lifecycle_snapshot_payload(trimmed, index)?;
    let snapshot = parse_message_lifecycle_snapshot_payload(&payload, index)?;
    verify_message_lifecycle_snapshot(&snapshot, index)?;
    Ok(Some(snapshot))
}

fn decode_message_lifecycle_snapshot_payload(
    line: &str,
    index: usize,
) -> Result<String, MessageLifecycleSnapshotStoreError> {
    let payload_hex = parse_message_lifecycle_snapshot_journal_record(line, index)?;
    let payload_bytes = decode_snapshot_journal_hex(&payload_hex)
        .ok_or_else(|| message_lifecycle_snapshot_journal_corrupt_tail(index))?;
    String::from_utf8(payload_bytes)
        .map_err(|_| message_lifecycle_snapshot_journal_corrupt_tail(index))
}

fn parse_message_lifecycle_snapshot_payload(
    payload: &str,
    index: usize,
) -> Result<MessageLifecycleSnapshot, MessageLifecycleSnapshotStoreError> {
    codec::parse_message_lifecycle_snapshot_payload(payload)
        .map_err(|_| message_lifecycle_snapshot_journal_corrupt_tail(index))
}

fn verify_message_lifecycle_snapshot(
    snapshot: &MessageLifecycleSnapshot,
    index: usize,
) -> Result<(), MessageLifecycleSnapshotStoreError> {
    let mut verifier = MessageLifecycleStore::new();
    verifier
        .restore_snapshot(snapshot.clone())
        .map_err(|_| message_lifecycle_snapshot_journal_corrupt_tail(index))
}

fn parse_message_lifecycle_snapshot_journal_record(
    line: &str,
    index: usize,
) -> Result<String, MessageLifecycleSnapshotStoreError> {
    parse_snapshot_journal_record(line)
        .ok_or_else(|| message_lifecycle_snapshot_journal_corrupt_tail(index))
}

fn message_lifecycle_snapshot_journal_corrupt_tail(
    index: usize,
) -> MessageLifecycleSnapshotStoreError {
    MessageLifecycleSnapshotStoreError::InvalidPayload(format!(
        "{MESSAGE_LIFECYCLE_SNAPSHOT_JOURNAL_CORRUPT_TAIL_PREFIX}:{index}"
    ))
}
