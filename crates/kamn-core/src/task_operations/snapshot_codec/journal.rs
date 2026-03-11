use super::*;

const TASK_OPERATION_SNAPSHOT_JOURNAL_CORRUPT_TAIL_PREFIX: &str =
    "task_operation_snapshot_journal_corrupt_tail";

pub fn task_operation_snapshot_journal_path(path: &Path) -> PathBuf {
    default_snapshot_journal_path(path)
}

pub fn append_task_operation_snapshot_journal_record(
    journal_path: &Path,
    payload: &str,
) -> Result<(), TaskOperationSnapshotStoreError> {
    append_snapshot_journal_record(journal_path, payload)
        .map_err(|error| TaskOperationSnapshotStoreError::Io(error.to_string()))?;
    Ok(())
}

pub fn replay_task_operation_snapshot_journal(
    journal_path: &Path,
) -> Result<Option<TaskOperationSnapshot>, TaskOperationSnapshotStoreError> {
    if !journal_path.exists() {
        return Ok(None);
    }
    let payload = read_journal_payload(journal_path)?;
    let mut latest = None;
    for (index, line) in payload.lines().enumerate() {
        if line.trim().is_empty() {
            continue;
        }
        latest = Some(replay_snapshot_line(line, index + 1)?);
    }
    Ok(latest)
}

fn read_journal_payload(journal_path: &Path) -> Result<String, TaskOperationSnapshotStoreError> {
    fs::read_to_string(journal_path)
        .map_err(|error| TaskOperationSnapshotStoreError::Io(error.to_string()))
}

fn replay_snapshot_line(
    line: &str,
    index: usize,
) -> Result<TaskOperationSnapshot, TaskOperationSnapshotStoreError> {
    let payload = decode_journal_line(line.trim(), index)?;
    let snapshot = parse_task_operation_snapshot_payload(&payload)
        .map_err(|_| task_operation_snapshot_journal_corrupt_tail(index))?;
    verify_replayed_snapshot(&snapshot, index)?;
    Ok(snapshot)
}

fn decode_journal_line(
    line: &str,
    index: usize,
) -> Result<String, TaskOperationSnapshotStoreError> {
    let payload_hex = parse_task_operation_snapshot_journal_record(line, index)?;
    let payload_bytes = decode_snapshot_journal_hex(&payload_hex)
        .ok_or_else(|| task_operation_snapshot_journal_corrupt_tail(index))?;
    String::from_utf8(payload_bytes)
        .map_err(|_| task_operation_snapshot_journal_corrupt_tail(index))
}

fn verify_replayed_snapshot(
    snapshot: &TaskOperationSnapshot,
    index: usize,
) -> Result<(), TaskOperationSnapshotStoreError> {
    let mut verifier = TaskOperationEngine::new();
    verifier
        .restore_snapshot(snapshot.clone())
        .map_err(|_| task_operation_snapshot_journal_corrupt_tail(index))
}

pub fn parse_task_operation_snapshot_journal_record(
    line: &str,
    index: usize,
) -> Result<String, TaskOperationSnapshotStoreError> {
    parse_snapshot_journal_record(line)
        .ok_or_else(|| task_operation_snapshot_journal_corrupt_tail(index))
}

pub fn task_operation_snapshot_journal_corrupt_tail(
    index: usize,
) -> TaskOperationSnapshotStoreError {
    TaskOperationSnapshotStoreError::InvalidPayload(format!(
        "{TASK_OPERATION_SNAPSHOT_JOURNAL_CORRUPT_TAIL_PREFIX}:{index}"
    ))
}

pub fn task_operation_snapshot_journal_recovery_error() -> &'static str {
    TASK_OPERATION_SNAPSHOT_JOURNAL_CORRUPT_TAIL_PREFIX
}
