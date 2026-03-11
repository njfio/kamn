mod domain;
mod errors;
mod lifecycle_store;
mod snapshot_codec;
mod snapshot_file_store;
mod snapshot_sqlite_store;
#[cfg(test)]
mod tests;

pub use domain::{
    MessageLifecycleSnapshot, MessageRecordSnapshot, MessageStatus,
    MESSAGE_LIFECYCLE_SNAPSHOT_SCHEMA_VERSION,
};
pub use errors::{
    MessageLifecycleError, MessageLifecycleSnapshotError, MessageLifecycleSnapshotStoreError,
    MessageProofAdmissionError,
};
pub use lifecycle_store::MessageLifecycleStore;
pub use snapshot_file_store::{
    FileMessageLifecycleSnapshotStore, InMemoryMessageLifecycleSnapshotStore,
    MessageLifecycleRecoveryResult, MessageLifecycleSnapshotStore,
};
pub use snapshot_sqlite_store::SqliteMessageLifecycleSnapshotStore;
