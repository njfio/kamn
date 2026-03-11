mod creation;
mod domain;
mod membership;
mod snapshot;
mod specialized;
mod support;

pub(crate) use domain::ChannelRecord;
pub use domain::{
    ChannelMetadata, ChannelRecordSnapshot, ChannelSnapshot, ChannelStore, ChannelType,
    CHANNEL_SNAPSHOT_SCHEMA_VERSION,
};
pub(crate) use support::{
    enforce_specialized_member_requirements, metadata_matches_channel_type, validate_channel_id,
    validate_did, validate_metadata,
};
