use super::channel_errors::{ChannelModelError, ChannelSnapshotError};
use super::channel_types::{
    ChannelMetadata, ChannelRecord, ChannelRecordSnapshot, ChannelSnapshot, ChannelType,
    CHANNEL_SNAPSHOT_SCHEMA_VERSION,
};
use crate::AgentDid;
use std::collections::{BTreeMap, BTreeSet};

mod create;
mod membership;
mod query_snapshot;
mod validation;

/// In-memory channel state store with membership and admin indexes.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ChannelStore {
    channels: BTreeMap<String, ChannelRecord>,
    channels_by_member: BTreeMap<String, BTreeSet<String>>,
}
