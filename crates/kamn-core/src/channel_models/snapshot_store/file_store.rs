use super::journal::{
    append_channel_snapshot_journal_record, channel_snapshot_journal_path,
    read_channel_snapshot_file, replay_channel_snapshot_journal,
    CHANNEL_SNAPSHOT_JOURNAL_CORRUPT_TAIL_PREFIX,
};
use super::*;

impl FileChannelSnapshotStore {
    /// Create a file-backed store for the given snapshot path.
    pub fn new(path: PathBuf) -> Result<Self, ChannelSnapshotStoreError> {
        if path.as_os_str().is_empty() {
            return Err(ChannelSnapshotStoreError::InvalidPayload(
                "snapshot file path cannot be empty".to_owned(),
            ));
        }
        let journal_path = channel_snapshot_journal_path(&path);
        Ok(Self { path, journal_path })
    }

    /// Attempt to read latest snapshot and repair invalid persisted payloads.
    pub fn recover_latest_and_repair(
        &mut self,
    ) -> Result<ChannelRecoveryResult, ChannelSnapshotStoreError> {
        if !self.path.exists() && !self.journal_path.exists() {
            return Ok(ChannelRecoveryResult {
                latest: None,
                repaired: false,
                reason_code: "channel_snapshot_recovery_empty",
            });
        }

        match self.read_latest() {
            Ok(snapshot) => Ok(ChannelRecoveryResult {
                latest: snapshot,
                repaired: false,
                reason_code: "channel_snapshot_recovery_clean",
            }),
            Err(ChannelSnapshotStoreError::InvalidPayload(value))
                if value.starts_with(CHANNEL_SNAPSHOT_JOURNAL_CORRUPT_TAIL_PREFIX) =>
            {
                Err(ChannelSnapshotStoreError::InvalidPayload(value))
            }
            Err(ChannelSnapshotStoreError::InvalidPayload(_))
            | Err(ChannelSnapshotStoreError::Snapshot(_)) => {
                fs::write(&self.path, "")
                    .map_err(|error| ChannelSnapshotStoreError::Io(error.to_string()))?;
                fs::write(&self.journal_path, "")
                    .map_err(|error| ChannelSnapshotStoreError::Io(error.to_string()))?;
                Ok(ChannelRecoveryResult {
                    latest: None,
                    repaired: true,
                    reason_code: "channel_snapshot_recovery_repaired_corrupt_payload",
                })
            }
            Err(error) => Err(error),
        }
    }
}

impl ChannelSnapshotStore for FileChannelSnapshotStore {
    fn write(&mut self, snapshot: ChannelSnapshot) -> Result<(), ChannelSnapshotStoreError> {
        let mut verifier = ChannelStore::new();
        verifier
            .restore_snapshot(snapshot.clone())
            .map_err(ChannelSnapshotStoreError::Snapshot)?;
        let payload = serialize_channel_snapshot(&snapshot)?;
        let mut file = OpenOptions::new()
            .create(true)
            .truncate(true)
            .write(true)
            .open(&self.path)
            .map_err(|error| ChannelSnapshotStoreError::Io(error.to_string()))?;
        file.write_all(payload.as_bytes())
            .map_err(|error| ChannelSnapshotStoreError::Io(error.to_string()))?;
        append_channel_snapshot_journal_record(&self.journal_path, &payload)
    }

    fn read_latest(&self) -> Result<Option<ChannelSnapshot>, ChannelSnapshotStoreError> {
        let journal_snapshot = replay_channel_snapshot_journal(&self.journal_path)?;
        if journal_snapshot.is_some() {
            return Ok(journal_snapshot);
        }
        read_channel_snapshot_file(&self.path)
    }
}
