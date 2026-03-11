use super::*;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct InMemoryChannelSnapshotStore {
    latest: Option<ChannelSnapshot>,
}

impl ChannelSnapshotStore for InMemoryChannelSnapshotStore {
    fn write(&mut self, snapshot: ChannelSnapshot) -> Result<(), ChannelSnapshotStoreError> {
        self.latest = Some(snapshot);
        Ok(())
    }

    fn read_latest(&self) -> Result<Option<ChannelSnapshot>, ChannelSnapshotStoreError> {
        Ok(self.latest.clone())
    }
}
