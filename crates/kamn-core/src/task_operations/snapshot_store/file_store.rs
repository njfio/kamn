#![allow(missing_docs)]

use super::*;

/// Filesystem-backed snapshot store implementation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileTaskOperationSnapshotStore {
    path: PathBuf,
    journal_path: PathBuf,
}

/// Recovery outcome for filesystem snapshot repair flow.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TaskOperationRecoveryResult {
    pub latest: Option<TaskOperationSnapshot>,
    pub repaired: bool,
    pub reason_code: &'static str,
}

impl TaskOperationRecoveryResult {
    pub fn reason_code(&self) -> &'static str {
        self.reason_code
    }
}

impl FileTaskOperationSnapshotStore {
    pub fn new(path: PathBuf) -> Result<Self, TaskOperationSnapshotStoreError> {
        if path.as_os_str().is_empty() {
            return Err(TaskOperationSnapshotStoreError::InvalidPayload(
                "snapshot file path cannot be empty".to_owned(),
            ));
        }
        let journal_path = task_operation_snapshot_journal_path(&path);
        Ok(Self { path, journal_path })
    }

    pub fn recover_latest_and_repair(
        &mut self,
    ) -> Result<TaskOperationRecoveryResult, TaskOperationSnapshotStoreError> {
        if !self.path.exists() && !self.journal_path.exists() {
            return Ok(recovery_result(None, false, "task_operation_snapshot_recovery_empty"));
        }
        match self.read_latest() {
            Ok(snapshot) => Ok(recovery_result(
                snapshot,
                false,
                "task_operation_snapshot_recovery_clean",
            )),
            Err(TaskOperationSnapshotStoreError::InvalidPayload(value))
                if value.starts_with(task_operation_snapshot_journal_recovery_error()) =>
            {
                Err(TaskOperationSnapshotStoreError::InvalidPayload(value))
            }
            Err(TaskOperationSnapshotStoreError::InvalidPayload(_))
            | Err(TaskOperationSnapshotStoreError::Snapshot(_)) => self.repair_corrupt_payload(),
            Err(error) => Err(error),
        }
    }

    fn repair_corrupt_payload(
        &self,
    ) -> Result<TaskOperationRecoveryResult, TaskOperationSnapshotStoreError> {
        clear_recovery_path(&self.path)?;
        clear_recovery_path(&self.journal_path)?;
        Ok(recovery_result(
            None,
            true,
            "task_operation_snapshot_recovery_repaired_corrupt_payload",
        ))
    }
}

impl TaskOperationSnapshotStore for FileTaskOperationSnapshotStore {
    fn write(
        &mut self,
        snapshot: TaskOperationSnapshot,
    ) -> Result<(), TaskOperationSnapshotStoreError> {
        let mut verifier = TaskOperationEngine::new();
        verifier
            .restore_snapshot(snapshot.clone())
            .map_err(TaskOperationSnapshotStoreError::Snapshot)?;
        let payload = serialize_task_operation_snapshot(&snapshot)?;
        let mut file = OpenOptions::new()
            .create(true)
            .truncate(true)
            .write(true)
            .open(&self.path)
            .map_err(|error| TaskOperationSnapshotStoreError::Io(error.to_string()))?;
        file.write_all(payload.as_bytes())
            .map_err(|error| TaskOperationSnapshotStoreError::Io(error.to_string()))?;
        append_task_operation_snapshot_journal_record(&self.journal_path, &payload)
    }

    fn read_latest(
        &self,
    ) -> Result<Option<TaskOperationSnapshot>, TaskOperationSnapshotStoreError> {
        let journal_snapshot = replay_task_operation_snapshot_journal(&self.journal_path)?;
        if journal_snapshot.is_some() {
            return Ok(journal_snapshot);
        }
        read_task_operation_snapshot_file(&self.path)
    }
}

fn clear_recovery_path(path: &PathBuf) -> Result<(), TaskOperationSnapshotStoreError> {
    fs::write(path, "").map_err(|error| TaskOperationSnapshotStoreError::Io(error.to_string()))
}

fn recovery_result(
    latest: Option<TaskOperationSnapshot>,
    repaired: bool,
    reason_code: &'static str,
) -> TaskOperationRecoveryResult {
    TaskOperationRecoveryResult {
        latest,
        repaired,
        reason_code,
    }
}
