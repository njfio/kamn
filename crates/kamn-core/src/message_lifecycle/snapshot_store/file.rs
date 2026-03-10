use super::*;
use std::fs;
use std::fs::OpenOptions;
use std::io::Write;

#[derive(Debug, Clone, PartialEq, Eq)]
/// File-backed snapshot store for lifecycle state recovery.
pub struct FileMessageLifecycleSnapshotStore {
    path: PathBuf,
    journal_path: PathBuf,
}

impl FileMessageLifecycleSnapshotStore {
    /// Creates a file-backed snapshot store at `path`.
    pub fn new(path: PathBuf) -> Result<Self, MessageLifecycleSnapshotStoreError> {
        if path.as_os_str().is_empty() {
            return Err(MessageLifecycleSnapshotStoreError::InvalidPayload(
                "snapshot file path cannot be empty".to_owned(),
            ));
        }
        let journal_path = journal::message_lifecycle_snapshot_journal_path(&path);
        Ok(Self { path, journal_path })
    }

    /// Loads the latest snapshot and repairs malformed payloads by truncating the file.
    pub fn recover_latest_and_repair(
        &mut self,
    ) -> Result<MessageLifecycleRecoveryResult, MessageLifecycleSnapshotStoreError> {
        if !self.path.exists() && !self.journal_path.exists() {
            return Ok(empty_recovery_result());
        }
        match self.read_latest() {
            Ok(snapshot) => Ok(clean_recovery_result(snapshot)),
            Err(MessageLifecycleSnapshotStoreError::InvalidPayload(value))
                if value.starts_with(
                    journal::MESSAGE_LIFECYCLE_SNAPSHOT_JOURNAL_CORRUPT_TAIL_PREFIX,
                ) =>
            {
                Err(MessageLifecycleSnapshotStoreError::InvalidPayload(value))
            }
            Err(MessageLifecycleSnapshotStoreError::InvalidPayload(_))
            | Err(MessageLifecycleSnapshotStoreError::Snapshot(_)) => repair_corrupt_payload(self),
            Err(error) => Err(error),
        }
    }
}

impl MessageLifecycleSnapshotStore for FileMessageLifecycleSnapshotStore {
    fn write(
        &mut self,
        snapshot: MessageLifecycleSnapshot,
    ) -> Result<(), MessageLifecycleSnapshotStoreError> {
        let mut verifier = MessageLifecycleStore::new();
        verifier
            .restore_snapshot(snapshot.clone())
            .map_err(MessageLifecycleSnapshotStoreError::Snapshot)?;
        let payload = codec::serialize_message_lifecycle_snapshot(&snapshot)?;
        let mut file = OpenOptions::new()
            .create(true)
            .truncate(true)
            .write(true)
            .open(&self.path)
            .map_err(|error| MessageLifecycleSnapshotStoreError::Io(error.to_string()))?;
        file.write_all(payload.as_bytes())
            .map_err(|error| MessageLifecycleSnapshotStoreError::Io(error.to_string()))?;
        journal::append_message_lifecycle_snapshot_journal_record(&self.journal_path, &payload)
    }

    fn read_latest(
        &self,
    ) -> Result<Option<MessageLifecycleSnapshot>, MessageLifecycleSnapshotStoreError> {
        let journal_snapshot =
            journal::replay_message_lifecycle_snapshot_journal(&self.journal_path)?;
        if journal_snapshot.is_some() {
            return Ok(journal_snapshot);
        }
        journal::read_message_lifecycle_snapshot_file(&self.path)
    }
}

fn empty_recovery_result() -> MessageLifecycleRecoveryResult {
    MessageLifecycleRecoveryResult {
        latest: None,
        repaired: false,
        reason_code: "message_lifecycle_snapshot_recovery_empty",
    }
}

fn clean_recovery_result(
    latest: Option<MessageLifecycleSnapshot>,
) -> MessageLifecycleRecoveryResult {
    MessageLifecycleRecoveryResult {
        latest,
        repaired: false,
        reason_code: "message_lifecycle_snapshot_recovery_clean",
    }
}

fn repair_corrupt_payload(
    store: &FileMessageLifecycleSnapshotStore,
) -> Result<MessageLifecycleRecoveryResult, MessageLifecycleSnapshotStoreError> {
    truncate_snapshot_path(&store.path)?;
    truncate_snapshot_path(&store.journal_path)?;
    Ok(MessageLifecycleRecoveryResult {
        latest: None,
        repaired: true,
        reason_code: "message_lifecycle_snapshot_recovery_repaired_corrupt_payload",
    })
}

fn truncate_snapshot_path(path: &Path) -> Result<(), MessageLifecycleSnapshotStoreError> {
    fs::write(path, "").map_err(|error| MessageLifecycleSnapshotStoreError::Io(error.to_string()))
}
