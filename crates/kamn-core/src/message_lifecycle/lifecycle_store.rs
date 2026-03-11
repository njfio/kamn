mod core;
mod mutation;
mod queries;
mod restore;
mod validation;

use super::domain::{MessageRecordSnapshot, MessageStatus};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct MessageRecord {
    sender: String,
    recipients: Vec<String>,
    created: String,
    expires: String,
    status: MessageStatus,
    history: Vec<MessageStatus>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
/// In-memory lifecycle index for message status and participant lookups.
pub struct MessageLifecycleStore {
    pub(super) records: BTreeMap<String, MessageRecord>,
    pub(super) ids_by_status: BTreeMap<MessageStatus, BTreeSet<String>>,
    pub(super) ids_by_sender: BTreeMap<String, BTreeSet<String>>,
    pub(super) ids_by_recipient: BTreeMap<String, BTreeSet<String>>,
}

#[derive(Default)]
pub(super) struct RestoredSnapshotState {
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
