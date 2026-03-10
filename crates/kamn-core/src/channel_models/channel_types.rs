use std::collections::BTreeSet;

/// Supported channel categories in the KAMN messaging model.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChannelType {
    /// Two-party direct channel.
    Direct,
    /// Multi-party group channel.
    Group,
    /// One-to-many broadcast channel.
    Broadcast,
    /// Task-scoped collaboration channel.
    Task,
    /// Marketplace-scoped negotiation channel.
    Marketplace,
    /// Governance-scoped proposal channel.
    Governance,
}

/// Channel-type-specific metadata payload.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ChannelMetadata {
    /// Direct channel metadata (no additional payload).
    Direct,
    /// Group channel metadata (no additional payload).
    Group,
    /// Broadcast metadata with topic label.
    Broadcast { topic: String },
    /// Task metadata with bound task identifier.
    Task { task_id: String },
    /// Marketplace metadata with scope identifier.
    Marketplace { market_scope: String },
    /// Governance metadata with proposal scope identifier.
    Governance { proposal_scope: String },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ChannelRecord {
    pub(crate) channel_type: ChannelType,
    pub(crate) metadata: ChannelMetadata,
    pub(crate) members: BTreeSet<String>,
    pub(crate) admins: BTreeSet<String>,
}

/// Schema version for serialized [`ChannelSnapshot`] payloads.
pub const CHANNEL_SNAPSHOT_SCHEMA_VERSION: u16 = 1;

/// Serializable channel record used for snapshot export/import.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChannelRecordSnapshot {
    pub channel_id: String,
    pub channel_type: ChannelType,
    pub metadata: ChannelMetadata,
    pub members: Vec<String>,
    pub admins: Vec<String>,
}

/// Serializable snapshot of all channel records.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChannelSnapshot {
    pub schema_version: u16,
    pub records: Vec<ChannelRecordSnapshot>,
}
