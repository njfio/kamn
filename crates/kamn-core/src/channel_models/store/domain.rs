#![allow(missing_docs)]

use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChannelType {
    Direct,
    Group,
    Broadcast,
    Task,
    Marketplace,
    Governance,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ChannelMetadata {
    Direct,
    Group,
    Broadcast { topic: String },
    Task { task_id: String },
    Marketplace { market_scope: String },
    Governance { proposal_scope: String },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ChannelRecord {
    pub(crate) channel_type: ChannelType,
    pub(crate) metadata: ChannelMetadata,
    pub(crate) members: BTreeSet<String>,
    pub(crate) admins: BTreeSet<String>,
}

pub const CHANNEL_SNAPSHOT_SCHEMA_VERSION: u16 = 1;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChannelRecordSnapshot {
    pub channel_id: String,
    pub channel_type: ChannelType,
    pub metadata: ChannelMetadata,
    pub members: Vec<String>,
    pub admins: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChannelSnapshot {
    pub schema_version: u16,
    pub records: Vec<ChannelRecordSnapshot>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ChannelStore {
    pub(crate) channels: BTreeMap<String, ChannelRecord>,
    pub(crate) channels_by_member: BTreeMap<String, BTreeSet<String>>,
}
