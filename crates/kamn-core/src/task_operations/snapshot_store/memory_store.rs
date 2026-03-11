use super::*;

/// In-memory snapshot store implementation for tests and ephemeral runtime paths.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct InMemoryTaskOperationSnapshotStore {
    latest: Option<TaskOperationSnapshot>,
}

impl TaskOperationSnapshotStore for InMemoryTaskOperationSnapshotStore {
    fn write(
        &mut self,
        snapshot: TaskOperationSnapshot,
    ) -> Result<(), TaskOperationSnapshotStoreError> {
        self.latest = Some(snapshot);
        Ok(())
    }

    fn read_latest(
        &self,
    ) -> Result<Option<TaskOperationSnapshot>, TaskOperationSnapshotStoreError> {
        Ok(self.latest.clone())
    }
}
