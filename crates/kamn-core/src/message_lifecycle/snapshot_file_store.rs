mod journal;

use super::snapshot_codec::serialize_message_lifecycle_snapshot;
use crate::message_lifecycle::{
    MessageLifecycleSnapshot, MessageLifecycleSnapshotStoreError, MessageLifecycleStore,
};
use journal::{
    message_lifecycle_snapshot_journal_path, read_message_lifecycle_snapshot_file,
    replay_message_lifecycle_snapshot_journal,
};
use std::fs;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::PathBuf;

/// Snapshot persistence contract for lifecycle state.
pub trait MessageLifecycleSnapshotStore {
    /// Persists the latest snapshot state.
    fn write(
        &mut self,
        snapshot: MessageLifecycleSnapshot,
    ) -> Result<(), MessageLifecycleSnapshotStoreError>;
    /// Loads the latest persisted snapshot when one exists.
    fn read_latest(
        &self,
    ) -> Result<Option<MessageLifecycleSnapshot>, MessageLifecycleSnapshotStoreError>;
}

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

#[derive(Debug, Clone, PartialEq, Eq)]
/// File-backed snapshot store for lifecycle state recovery.
pub struct FileMessageLifecycleSnapshotStore {
    path: PathBuf,
    journal_path: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// Recovery result returned when loading/repairing file-backed snapshots.
pub struct MessageLifecycleRecoveryResult {
    /// Latest usable snapshot after recovery.
    pub latest: Option<MessageLifecycleSnapshot>,
    /// Whether recovery truncated corrupt state.
    pub repaired: bool,
    /// Stable reason code describing the recovery outcome.
    pub reason_code: &'static str,
}

impl MessageLifecycleRecoveryResult {
    /// Returns the stable recovery reason code.
    pub fn reason_code(&self) -> &'static str {
        self.reason_code
    }
}

impl FileMessageLifecycleSnapshotStore {
    /// Creates a file-backed snapshot store rooted at `path`.
    pub fn new(path: PathBuf) -> Result<Self, MessageLifecycleSnapshotStoreError> {
        if path.as_os_str().is_empty() {
            return Err(MessageLifecycleSnapshotStoreError::InvalidPayload(
                "snapshot file path cannot be empty".to_owned(),
            ));
        }
        Ok(Self {
            journal_path: message_lifecycle_snapshot_journal_path(&path),
            path,
        })
    }

    /// Reads the latest snapshot and truncates corrupt payloads when repair is possible.
    pub fn recover_latest_and_repair(
        &mut self,
    ) -> Result<MessageLifecycleRecoveryResult, MessageLifecycleSnapshotStoreError> {
        if !self.path.exists() && !self.journal_path.exists() {
            return Ok(empty_recovery_result());
        }
        match self.read_latest() {
            Ok(snapshot) => Ok(clean_recovery_result(snapshot)),
            Err(MessageLifecycleSnapshotStoreError::InvalidPayload(value))
                if value.starts_with("message_lifecycle_snapshot_journal_corrupt_tail") =>
            {
                Err(MessageLifecycleSnapshotStoreError::InvalidPayload(value))
            }
            Err(MessageLifecycleSnapshotStoreError::InvalidPayload(_))
            | Err(MessageLifecycleSnapshotStoreError::Snapshot(_)) => {
                truncate_corrupt_snapshot_files(&self.path, &self.journal_path)?;
                Ok(repaired_recovery_result())
            }
            Err(error) => Err(error),
        }
    }
}

impl MessageLifecycleSnapshotStore for FileMessageLifecycleSnapshotStore {
    fn write(
        &mut self,
        snapshot: MessageLifecycleSnapshot,
    ) -> Result<(), MessageLifecycleSnapshotStoreError> {
        verify_snapshot(snapshot.clone())?;
        let payload = serialize_message_lifecycle_snapshot(&snapshot)?;
        write_snapshot_payload(&self.path, &payload)?;
        journal::append_message_lifecycle_snapshot_journal_record(&self.journal_path, &payload)
    }

    fn read_latest(
        &self,
    ) -> Result<Option<MessageLifecycleSnapshot>, MessageLifecycleSnapshotStoreError> {
        let journal_snapshot = replay_message_lifecycle_snapshot_journal(&self.journal_path)?;
        if journal_snapshot.is_some() {
            return Ok(journal_snapshot);
        }
        read_message_lifecycle_snapshot_file(&self.path)
    }
}

fn verify_snapshot(
    snapshot: MessageLifecycleSnapshot,
) -> Result<(), MessageLifecycleSnapshotStoreError> {
    let mut verifier = MessageLifecycleStore::new();
    verifier
        .restore_snapshot(snapshot)
        .map_err(MessageLifecycleSnapshotStoreError::Snapshot)
}

fn write_snapshot_payload(
    path: &PathBuf,
    payload: &str,
) -> Result<(), MessageLifecycleSnapshotStoreError> {
    let mut file = OpenOptions::new()
        .create(true)
        .truncate(true)
        .write(true)
        .open(path)
        .map_err(|error| MessageLifecycleSnapshotStoreError::Io(error.to_string()))?;
    file.write_all(payload.as_bytes())
        .map_err(|error| MessageLifecycleSnapshotStoreError::Io(error.to_string()))
}

fn truncate_corrupt_snapshot_files(
    path: &PathBuf,
    journal_path: &PathBuf,
) -> Result<(), MessageLifecycleSnapshotStoreError> {
    fs::write(path, "")
        .map_err(|error| MessageLifecycleSnapshotStoreError::Io(error.to_string()))?;
    fs::write(journal_path, "")
        .map_err(|error| MessageLifecycleSnapshotStoreError::Io(error.to_string()))?;
    Ok(())
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

fn repaired_recovery_result() -> MessageLifecycleRecoveryResult {
    MessageLifecycleRecoveryResult {
        latest: None,
        repaired: true,
        reason_code: "message_lifecycle_snapshot_recovery_repaired_corrupt_payload",
    }
}
