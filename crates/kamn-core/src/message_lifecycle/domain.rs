#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
/// Canonical lifecycle state for a message tracked by [`super::MessageLifecycleStore`].
pub enum MessageStatus {
    /// Message metadata is registered but not signed.
    Created,
    /// Message is signed and ready for broadcast.
    Signed,
    /// Message has been broadcast to the transport layer.
    Broadcast,
    /// Message is included by the target chain/runtime.
    Included,
    /// Message is delivered to recipients.
    Delivered,
    /// Message is validated with processor proof evidence.
    Validated,
    /// Message is rejected after validation or policy checks.
    Rejected,
    /// Message is expired and no longer active.
    Expired,
}

/// Schema version for serialized lifecycle snapshots.
pub const MESSAGE_LIFECYCLE_SNAPSHOT_SCHEMA_VERSION: u16 = 1;

#[derive(Debug, Clone, PartialEq, Eq)]
/// Serializable snapshot record for one message lifecycle entry.
pub struct MessageRecordSnapshot {
    /// Stable message identifier.
    pub message_id: String,
    /// Sender DID.
    pub sender: String,
    /// Recipient DID set.
    pub recipients: Vec<String>,
    /// Envelope creation timestamp.
    pub created: String,
    /// Envelope expiry timestamp.
    pub expires: String,
    /// Current lifecycle status.
    pub status: MessageStatus,
    /// Ordered status transition history.
    pub history: Vec<MessageStatus>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// Serializable snapshot of all lifecycle records.
pub struct MessageLifecycleSnapshot {
    /// Snapshot schema version.
    pub schema_version: u16,
    /// Snapshot records keyed by message id inside the payload.
    pub records: Vec<MessageRecordSnapshot>,
}
