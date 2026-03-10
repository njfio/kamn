//! Channel model contracts covering membership, admin policy, and snapshot recovery.

mod channel_errors;
mod channel_store;
mod channel_types;
mod snapshot_codec;
mod snapshot_store;

pub use channel_errors::{ChannelModelError, ChannelSnapshotError, ChannelSnapshotStoreError};
pub use channel_store::ChannelStore;
pub use channel_types::{
    ChannelMetadata, ChannelRecordSnapshot, ChannelSnapshot, ChannelType,
    CHANNEL_SNAPSHOT_SCHEMA_VERSION,
};
pub use snapshot_store::{
    ChannelRecoveryResult, ChannelSnapshotStore, FileChannelSnapshotStore,
    InMemoryChannelSnapshotStore, SqliteChannelSnapshotStore,
};

#[cfg(test)]
#[path = "channel_models/tests.rs"]
mod tests;
