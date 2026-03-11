//! Task operation workflow contracts, dependency orchestration, and snapshot persistence.

mod engine;
mod models;
mod snapshot_codec;
mod snapshot_store;
#[cfg(test)]
mod tests;

pub use engine::TaskOperationEngine;
pub use models::{
    SwarmTaskDraft, TaskOperationError, TaskOperationNoticeKind, TaskOperationRecord,
    TaskOperationRecordSnapshot, TaskOperationSnapshot, TaskOperationSnapshotStoreError,
    TASK_OPERATION_SNAPSHOT_SCHEMA_VERSION,
};
pub use snapshot_store::{
    FileTaskOperationSnapshotStore, InMemoryTaskOperationSnapshotStore,
    SqliteTaskOperationSnapshotStore, TaskOperationRecoveryResult, TaskOperationSnapshotStore,
};
