use super::*;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
/// In-memory snapshot store used by tests and lightweight workflows.
pub struct InMemoryMessageLifecycleSnapshotStore {
    latest: Option<MessageLifecycleSnapshot>,
}

impl MessageLifecycleSnapshotStore for InMemoryMessageLifecycleSnapshotStore {
    fn write(
        &mut self,
        snapshot: MessageLifecycleSnapshot,
    ) -> Result<(), MessageLifecycleSnapshotStoreError> {
        self.latest = Some(snapshot);
        Ok(())
    }

    fn read_latest(
        &self,
    ) -> Result<Option<MessageLifecycleSnapshot>, MessageLifecycleSnapshotStoreError> {
        Ok(self.latest.clone())
    }
}
