use super::channel_errors::ChannelSnapshotStoreError;
use super::channel_store::ChannelStore;
use super::channel_types::ChannelSnapshot;
use super::snapshot_codec::{parse_channel_snapshot_payload, serialize_channel_snapshot};
use crate::{SqliteStoreBackend, SqliteStoreBackendError};
use kamn_snapshot_journal::{
    append_snapshot_journal_record, decode_snapshot_journal_hex, default_snapshot_journal_path,
    parse_snapshot_journal_record,
};
use std::fs;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::{Path, PathBuf};

mod file_store;
mod in_memory;
mod journal;
mod sqlite_store;

/// Persistence contract for channel snapshots.
pub trait ChannelSnapshotStore {
    fn write(&mut self, snapshot: ChannelSnapshot) -> Result<(), ChannelSnapshotStoreError>;
    fn read_latest(&self) -> Result<Option<ChannelSnapshot>, ChannelSnapshotStoreError>;
}

/// In-memory snapshot store for deterministic tests and ephemeral flows.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct InMemoryChannelSnapshotStore {
    latest: Option<ChannelSnapshot>,
}

/// File-backed snapshot store for durable channel-state persistence.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileChannelSnapshotStore {
    path: PathBuf,
    journal_path: PathBuf,
}

/// Result of file-store recovery attempt.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChannelRecoveryResult {
    pub latest: Option<ChannelSnapshot>,
    pub repaired: bool,
    pub reason_code: &'static str,
}

/// Sqlite-backed snapshot store for durable channel-state persistence.
#[derive(Debug)]
pub struct SqliteChannelSnapshotStore {
    backend: SqliteStoreBackend,
}
