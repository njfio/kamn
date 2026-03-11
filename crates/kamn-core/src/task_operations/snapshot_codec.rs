use kamn_snapshot_journal::{
    append_snapshot_journal_record, decode_snapshot_journal_hex, default_snapshot_journal_path,
    parse_snapshot_journal_record,
};
use std::fs;
use std::path::{Path, PathBuf};

use super::engine::TaskOperationEngine;
use super::models::{
    TaskOperationNoticeKind, TaskOperationRecordSnapshot, TaskOperationSnapshot,
    TaskOperationSnapshotStoreError,
};
use crate::TaskState;

mod journal;
mod parse;
mod serialize;

pub use journal::{
    append_task_operation_snapshot_journal_record, replay_task_operation_snapshot_journal,
    task_operation_snapshot_journal_path, task_operation_snapshot_journal_recovery_error,
};
pub use parse::parse_task_operation_snapshot_payload;
pub use serialize::serialize_task_operation_snapshot;

pub(super) fn read_task_operation_snapshot_file(
    path: &Path,
) -> Result<Option<TaskOperationSnapshot>, TaskOperationSnapshotStoreError> {
    if !path.exists() {
        return Ok(None);
    }
    let payload = fs::read_to_string(path)
        .map_err(|error| TaskOperationSnapshotStoreError::Io(error.to_string()))?;
    if payload.trim().is_empty() {
        return Ok(None);
    }
    let snapshot = parse_task_operation_snapshot_payload(&payload)?;
    let mut verifier = TaskOperationEngine::new();
    verifier
        .restore_snapshot(snapshot.clone())
        .map_err(TaskOperationSnapshotStoreError::Snapshot)?;
    Ok(Some(snapshot))
}
