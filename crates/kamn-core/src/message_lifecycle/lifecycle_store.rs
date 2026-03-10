use super::lifecycle_errors::{MessageLifecycleError, MessageLifecycleSnapshotError};
use super::lifecycle_types::{
    MessageLifecycleSnapshot, MessageRecord, MessageRecordSnapshot, MessageStatus,
    MESSAGE_LIFECYCLE_SNAPSHOT_SCHEMA_VERSION,
};
use std::collections::{BTreeMap, BTreeSet};

mod query;
mod register;
mod snapshot;
mod transitions;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
/// In-memory lifecycle index for message status and participant lookups.
pub struct MessageLifecycleStore {
    pub(super) records: BTreeMap<String, MessageRecord>,
    pub(super) ids_by_status: BTreeMap<MessageStatus, BTreeSet<String>>,
    pub(super) ids_by_sender: BTreeMap<String, BTreeSet<String>>,
    pub(super) ids_by_recipient: BTreeMap<String, BTreeSet<String>>,
}

impl MessageLifecycleStore {
    /// Creates an empty lifecycle store.
    pub fn new() -> Self {
        Self::default()
    }
}
