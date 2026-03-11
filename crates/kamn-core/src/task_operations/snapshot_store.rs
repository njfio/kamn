#![allow(missing_docs)]

use crate::{SqliteStoreBackend, SqliteStoreBackendError};
use std::fs;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::PathBuf;

use super::engine::TaskOperationEngine;
use super::models::{TaskOperationSnapshot, TaskOperationSnapshotStoreError};
use super::snapshot_codec::{
    append_task_operation_snapshot_journal_record, parse_task_operation_snapshot_payload,
    read_task_operation_snapshot_file, replay_task_operation_snapshot_journal,
    serialize_task_operation_snapshot, task_operation_snapshot_journal_path,
    task_operation_snapshot_journal_recovery_error,
};

mod file_store;
mod memory_store;
mod sqlite_store;
mod support;

pub use file_store::{FileTaskOperationSnapshotStore, TaskOperationRecoveryResult};
pub use memory_store::InMemoryTaskOperationSnapshotStore;
pub use sqlite_store::SqliteTaskOperationSnapshotStore;
pub use support::map_sqlite_store_error;

/// Snapshot persistence abstraction for task operation state.
pub trait TaskOperationSnapshotStore {
    fn write(
        &mut self,
        snapshot: TaskOperationSnapshot,
    ) -> Result<(), TaskOperationSnapshotStoreError>;

    fn read_latest(&self)
        -> Result<Option<TaskOperationSnapshot>, TaskOperationSnapshotStoreError>;
}
