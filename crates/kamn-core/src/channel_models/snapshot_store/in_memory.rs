use super::*;

impl ChannelSnapshotStore for InMemoryChannelSnapshotStore {
    fn write(&mut self, snapshot: ChannelSnapshot) -> Result<(), ChannelSnapshotStoreError> {
        self.latest = Some(snapshot);
        Ok(())
    }

    fn read_latest(&self) -> Result<Option<ChannelSnapshot>, ChannelSnapshotStoreError> {
        Ok(self.latest.clone())
    }
}

impl ChannelRecoveryResult {
    /// Returns the deterministic recovery reason code.
    pub fn reason_code(&self) -> &'static str {
        self.reason_code
    }
}
