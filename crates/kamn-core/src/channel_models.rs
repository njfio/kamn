//! Channel model contracts covering membership, admin policy, and snapshot recovery.
//!
//! Snapshot persistence is implemented by `snapshot_codec`, which consumes the extracted
//! `kamn_snapshot_journal` crate.

mod errors;
mod snapshot_codec;
mod snapshot_store;
mod store;
#[cfg(test)]
mod tests;

pub use errors::{ChannelModelError, ChannelSnapshotError, ChannelSnapshotStoreError};
pub use snapshot_store::{
    ChannelRecoveryResult, ChannelSnapshotStore, FileChannelSnapshotStore,
    InMemoryChannelSnapshotStore, SqliteChannelSnapshotStore,
};
pub use store::{
    ChannelMetadata, ChannelRecordSnapshot, ChannelSnapshot, ChannelStore, ChannelType,
    CHANNEL_SNAPSHOT_SCHEMA_VERSION,
};
