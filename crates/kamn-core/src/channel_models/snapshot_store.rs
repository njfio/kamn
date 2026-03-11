#![allow(missing_docs)]

use crate::{SqliteStoreBackend, SqliteStoreBackendError};
use std::fs;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::PathBuf;

use super::errors::ChannelSnapshotStoreError;
use super::snapshot_codec::{
    append_channel_snapshot_journal_record, channel_snapshot_journal_path,
    channel_snapshot_journal_recovery_error, parse_channel_snapshot_payload,
    read_channel_snapshot_file, replay_channel_snapshot_journal, serialize_channel_snapshot,
};
use super::store::{ChannelSnapshot, ChannelStore};

mod file_store;
mod memory_store;
mod sqlite_store;
mod support;

pub use file_store::{ChannelRecoveryResult, FileChannelSnapshotStore};
pub use memory_store::InMemoryChannelSnapshotStore;
pub use sqlite_store::SqliteChannelSnapshotStore;
pub(crate) use support::map_sqlite_store_error;

pub trait ChannelSnapshotStore {
    fn write(&mut self, snapshot: ChannelSnapshot) -> Result<(), ChannelSnapshotStoreError>;
    fn read_latest(&self) -> Result<Option<ChannelSnapshot>, ChannelSnapshotStoreError>;
}
