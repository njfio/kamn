use super::*;

pub(super) fn map_sqlite_store_error(error: SqliteStoreBackendError) -> ChannelSnapshotStoreError {
    match error {
        SqliteStoreBackendError::SchemaVersionMissing => ChannelSnapshotStoreError::InvalidPayload(
            "channel snapshot sqlite schema missing".to_owned(),
        ),
        SqliteStoreBackendError::SchemaVersionInvalid(value) => {
            ChannelSnapshotStoreError::InvalidPayload(format!(
                "channel snapshot sqlite schema invalid: {value}"
            ))
        }
        SqliteStoreBackendError::SchemaVersionMismatch { expected, found } => {
            ChannelSnapshotStoreError::InvalidPayload(format!(
                "channel snapshot sqlite schema mismatch: expected {expected}, found {found}"
            ))
        }
        SqliteStoreBackendError::InvalidPath => ChannelSnapshotStoreError::InvalidPayload(
            "snapshot file path cannot be empty".to_owned(),
        ),
        other => ChannelSnapshotStoreError::Io(other.to_string()),
    }
}

pub(super) const CHANNEL_SNAPSHOT_JOURNAL_CORRUPT_TAIL_PREFIX: &str =
    "channel_snapshot_journal_corrupt_tail";

pub(super) fn channel_snapshot_journal_path(path: &Path) -> PathBuf {
    default_snapshot_journal_path(path)
}

pub(super) fn read_channel_snapshot_file(
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

pub(super) fn append_channel_snapshot_journal_record(
    journal_path: &Path,
    payload: &str,
) -> Result<(), ChannelSnapshotStoreError> {
    append_snapshot_journal_record(journal_path, payload)
        .map_err(|error| ChannelSnapshotStoreError::Io(error.to_string()))?;
    Ok(())
}

pub(super) fn replay_channel_snapshot_journal(
    journal_path: &Path,
) -> Result<Option<ChannelSnapshot>, ChannelSnapshotStoreError> {
    if !journal_path.exists() {
        return Ok(None);
    }

    let payload = fs::read_to_string(journal_path)
        .map_err(|error| ChannelSnapshotStoreError::Io(error.to_string()))?;
    let mut latest = None;

    for (index, line) in payload.lines().enumerate() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        let payload_hex = parse_channel_snapshot_journal_record(trimmed, index + 1)?;
        let payload_bytes = decode_snapshot_journal_hex(&payload_hex)
            .ok_or_else(|| channel_snapshot_journal_corrupt_tail(index + 1))?;
        let payload = String::from_utf8(payload_bytes)
            .map_err(|_| channel_snapshot_journal_corrupt_tail(index + 1))?;
        let snapshot = parse_channel_snapshot_payload(&payload)
            .map_err(|_| channel_snapshot_journal_corrupt_tail(index + 1))?;
        let mut verifier = ChannelStore::new();
        verifier
            .restore_snapshot(snapshot.clone())
            .map_err(|_| channel_snapshot_journal_corrupt_tail(index + 1))?;
        latest = Some(snapshot);
    }

    Ok(latest)
}

fn parse_channel_snapshot_journal_record(
    line: &str,
    index: usize,
) -> Result<String, ChannelSnapshotStoreError> {
    parse_snapshot_journal_record(line).ok_or_else(|| channel_snapshot_journal_corrupt_tail(index))
}

fn channel_snapshot_journal_corrupt_tail(index: usize) -> ChannelSnapshotStoreError {
    ChannelSnapshotStoreError::InvalidPayload(format!(
        "{CHANNEL_SNAPSHOT_JOURNAL_CORRUPT_TAIL_PREFIX}:{index}"
    ))
}
