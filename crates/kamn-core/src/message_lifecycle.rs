mod lifecycle_errors;
mod lifecycle_store;
mod lifecycle_types;
mod proof_admission;
mod snapshot_store;

// Preserve the root module's `kamn_snapshot_journal` ownership marker for split contracts.
pub use lifecycle_errors::{
    MessageLifecycleError, MessageLifecycleSnapshotError, MessageLifecycleSnapshotStoreError,
    MessageProofAdmissionError,
};
pub use lifecycle_store::MessageLifecycleStore;
pub use lifecycle_types::{
    MessageLifecycleSnapshot, MessageRecordSnapshot, MessageStatus,
    MESSAGE_LIFECYCLE_SNAPSHOT_SCHEMA_VERSION,
};
pub use snapshot_store::{
    FileMessageLifecycleSnapshotStore, InMemoryMessageLifecycleSnapshotStore,
    MessageLifecycleRecoveryResult, MessageLifecycleSnapshotStore,
    SqliteMessageLifecycleSnapshotStore,
};

#[allow(dead_code)]
fn serialize_message_lifecycle_snapshot(
    snapshot: &MessageLifecycleSnapshot,
) -> Result<String, MessageLifecycleSnapshotStoreError> {
    snapshot_store::codec::serialize_message_lifecycle_snapshot(snapshot)
}

#[allow(dead_code)]
fn parse_message_lifecycle_snapshot_payload(
    payload: &str,
) -> Result<MessageLifecycleSnapshot, MessageLifecycleSnapshotStoreError> {
    snapshot_store::codec::parse_message_lifecycle_snapshot_payload(payload)
}

#[allow(dead_code)]
fn parse_message_lifecycle_snapshot_schema(
    schema_line: &str,
) -> Result<u16, MessageLifecycleSnapshotStoreError> {
    snapshot_store::codec::parse_message_lifecycle_snapshot_schema(schema_line)
}

#[allow(dead_code)]
fn parse_message_lifecycle_snapshot_record(
    line: &str,
) -> Result<MessageRecordSnapshot, MessageLifecycleSnapshotStoreError> {
    snapshot_store::codec::parse_message_lifecycle_snapshot_record(line)
}

#[allow(dead_code)]
fn parse_message_lifecycle_snapshot_status_history(
    raw: &str,
    line: &str,
) -> Result<Vec<MessageStatus>, MessageLifecycleSnapshotStoreError> {
    snapshot_store::codec::parse_message_lifecycle_snapshot_status_history(raw, line)
}

#[cfg(test)]
#[path = "message_lifecycle/tests/mod.rs"]
mod tests;
