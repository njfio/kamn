#![allow(missing_docs)]

use super::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileChannelSnapshotStore {
    path: PathBuf,
    journal_path: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChannelRecoveryResult {
    pub latest: Option<ChannelSnapshot>,
    pub repaired: bool,
    pub reason_code: &'static str,
}

impl ChannelRecoveryResult {
    pub fn reason_code(&self) -> &'static str {
        self.reason_code
    }
}

impl FileChannelSnapshotStore {
    pub fn new(path: PathBuf) -> Result<Self, ChannelSnapshotStoreError> {
        if path.as_os_str().is_empty() {
            return Err(ChannelSnapshotStoreError::InvalidPayload(
                "snapshot file path cannot be empty".to_owned(),
            ));
        }
        let journal_path = channel_snapshot_journal_path(&path);
        Ok(Self { path, journal_path })
    }

    pub fn recover_latest_and_repair(
        &mut self,
    ) -> Result<ChannelRecoveryResult, ChannelSnapshotStoreError> {
        if self.recovery_paths_absent() {
            return Ok(empty_recovery_result());
        }
        self.recovery_from_latest_snapshot()
    }

    fn repair_corrupt_payload(&self) -> Result<ChannelRecoveryResult, ChannelSnapshotStoreError> {
        clear_recovery_path(&self.path)?;
        clear_recovery_path(&self.journal_path)?;
        Ok(recovery_result(
            None,
            true,
            "channel_snapshot_recovery_repaired_corrupt_payload",
        ))
    }

    fn recovery_paths_absent(&self) -> bool {
        !self.path.exists() && !self.journal_path.exists()
    }

    fn recovery_from_latest_snapshot(
        &mut self,
    ) -> Result<ChannelRecoveryResult, ChannelSnapshotStoreError> {
        match self.read_latest() {
            Ok(snapshot) => Ok(clean_recovery_result(snapshot)),
            Err(error) => self.handle_recovery_error(error),
        }
    }

    fn handle_recovery_error(
        &self,
        error: ChannelSnapshotStoreError,
    ) -> Result<ChannelRecoveryResult, ChannelSnapshotStoreError> {
        match error {
            ChannelSnapshotStoreError::InvalidPayload(value)
                if value.starts_with(channel_snapshot_journal_recovery_error()) =>
            {
                Err(ChannelSnapshotStoreError::InvalidPayload(value))
            }
            ChannelSnapshotStoreError::InvalidPayload(_)
            | ChannelSnapshotStoreError::Snapshot(_) => self.repair_corrupt_payload(),
            other => Err(other),
        }
    }
}

impl ChannelSnapshotStore for FileChannelSnapshotStore {
    fn write(&mut self, snapshot: ChannelSnapshot) -> Result<(), ChannelSnapshotStoreError> {
        verify_snapshot(&snapshot)?;
        let payload = serialize_channel_snapshot(&snapshot)?;
        write_snapshot_file(&self.path, &payload)?;
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

fn verify_snapshot(snapshot: &ChannelSnapshot) -> Result<(), ChannelSnapshotStoreError> {
    let mut verifier = ChannelStore::new();
    verifier
        .restore_snapshot(snapshot.clone())
        .map_err(ChannelSnapshotStoreError::Snapshot)
}

fn write_snapshot_file(path: &PathBuf, payload: &str) -> Result<(), ChannelSnapshotStoreError> {
    let mut file = OpenOptions::new()
        .create(true)
        .truncate(true)
        .write(true)
        .open(path)
        .map_err(|error| ChannelSnapshotStoreError::Io(error.to_string()))?;
    file.write_all(payload.as_bytes())
        .map_err(|error| ChannelSnapshotStoreError::Io(error.to_string()))
}

fn clear_recovery_path(path: &PathBuf) -> Result<(), ChannelSnapshotStoreError> {
    fs::write(path, "").map_err(|error| ChannelSnapshotStoreError::Io(error.to_string()))
}

fn recovery_result(
    latest: Option<ChannelSnapshot>,
    repaired: bool,
    reason_code: &'static str,
) -> ChannelRecoveryResult {
    ChannelRecoveryResult {
        latest,
        repaired,
        reason_code,
    }
}

fn empty_recovery_result() -> ChannelRecoveryResult {
    recovery_result(None, false, "channel_snapshot_recovery_empty")
}

fn clean_recovery_result(latest: Option<ChannelSnapshot>) -> ChannelRecoveryResult {
    recovery_result(latest, false, "channel_snapshot_recovery_clean")
}
