use super::lifecycle_errors::MessageLifecycleSnapshotStoreError;
use super::lifecycle_store::MessageLifecycleStore;
use super::lifecycle_types::MessageLifecycleSnapshot;
use crate::{SqliteStoreBackend, SqliteStoreBackendError};
use std::path::{Path, PathBuf};

pub(super) mod codec;
mod file;
mod in_memory;
pub(super) mod journal;
mod sqlite;

pub use file::FileMessageLifecycleSnapshotStore;
pub use in_memory::InMemoryMessageLifecycleSnapshotStore;
pub use sqlite::SqliteMessageLifecycleSnapshotStore;

/// Snapshot persistence contract for lifecycle state.
pub trait MessageLifecycleSnapshotStore {
    /// Persists a complete lifecycle snapshot atomically for later recovery.
    fn write(
        &mut self,
        snapshot: MessageLifecycleSnapshot,
    ) -> Result<(), MessageLifecycleSnapshotStoreError>;
    /// Loads the latest valid lifecycle snapshot, if any exists.
    fn read_latest(
        &self,
    ) -> Result<Option<MessageLifecycleSnapshot>, MessageLifecycleSnapshotStoreError>;
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// Recovery result returned when loading/repairing file-backed snapshots.
pub struct MessageLifecycleRecoveryResult {
    /// The latest valid snapshot recovered from file or journal state.
    pub latest: Option<MessageLifecycleSnapshot>,
    /// Whether recovery had to repair corrupt on-disk state.
    pub repaired: bool,
    /// Machine-readable reason code describing the recovery path taken.
    pub reason_code: &'static str,
}

impl MessageLifecycleRecoveryResult {
    /// Returns the recovery reason code emitted by the store.
    pub fn reason_code(&self) -> &'static str {
        self.reason_code
    }
}
