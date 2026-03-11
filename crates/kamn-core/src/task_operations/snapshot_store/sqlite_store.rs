#![allow(missing_docs)]

use super::*;

/// Sqlite-backed snapshot store implementation.
#[derive(Debug)]
pub struct SqliteTaskOperationSnapshotStore {
    backend: SqliteStoreBackend,
}

impl SqliteTaskOperationSnapshotStore {
    pub fn new(path: PathBuf) -> Result<Self, TaskOperationSnapshotStoreError> {
        let backend = SqliteStoreBackend::open(path.as_path()).map_err(map_sqlite_store_error)?;
        Ok(Self { backend })
    }
}

impl TaskOperationSnapshotStore for SqliteTaskOperationSnapshotStore {
    fn write(
        &mut self,
        snapshot: TaskOperationSnapshot,
    ) -> Result<(), TaskOperationSnapshotStoreError> {
        let mut verifier = TaskOperationEngine::new();
        verifier
            .restore_snapshot(snapshot.clone())
            .map_err(TaskOperationSnapshotStoreError::Snapshot)?;
        let payload = serialize_task_operation_snapshot(&snapshot)?;
        self.backend
            .put(
                "task_operation_snapshot_store",
                "latest",
                payload.as_bytes(),
            )
            .map_err(map_sqlite_store_error)?;
        Ok(())
    }

    fn read_latest(
        &self,
    ) -> Result<Option<TaskOperationSnapshot>, TaskOperationSnapshotStoreError> {
        let Some(payload_bytes) = self
            .backend
            .get("task_operation_snapshot_store", "latest")
            .map_err(map_sqlite_store_error)?
        else {
            return Ok(None);
        };
        let payload = String::from_utf8(payload_bytes).map_err(|_| {
            TaskOperationSnapshotStoreError::InvalidPayload(
                "task operation sqlite payload is not utf-8".to_owned(),
            )
        })?;
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
}
