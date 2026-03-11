use super::super::errors::ChannelSnapshotStoreError;
use super::*;

pub(crate) fn channel_type_code(channel_type: ChannelType) -> &'static str {
    match channel_type {
        ChannelType::Direct => "0",
        ChannelType::Group => "1",
        ChannelType::Broadcast => "2",
        ChannelType::Task => "3",
        ChannelType::Marketplace => "4",
        ChannelType::Governance => "5",
    }
}

pub(crate) fn parse_channel_type_code(raw: &str) -> Option<ChannelType> {
    match raw {
        "0" => Some(ChannelType::Direct),
        "1" => Some(ChannelType::Group),
        "2" => Some(ChannelType::Broadcast),
        "3" => Some(ChannelType::Task),
        "4" => Some(ChannelType::Marketplace),
        "5" => Some(ChannelType::Governance),
        _ => None,
    }
}

pub(crate) fn metadata_snapshot_value(metadata: &ChannelMetadata) -> &str {
    match metadata {
        ChannelMetadata::Direct | ChannelMetadata::Group => "",
        ChannelMetadata::Broadcast { topic } => topic,
        ChannelMetadata::Task { task_id } => task_id,
        ChannelMetadata::Marketplace { market_scope } => market_scope,
        ChannelMetadata::Governance { proposal_scope } => proposal_scope,
    }
}

pub(crate) fn parse_metadata_snapshot_value(
    channel_type: ChannelType,
    value: &str,
) -> Result<ChannelMetadata, ChannelSnapshotStoreError> {
    match channel_type {
        ChannelType::Direct => parse_empty_metadata(value, ChannelMetadata::Direct),
        ChannelType::Group => parse_empty_metadata(value, ChannelMetadata::Group),
        ChannelType::Broadcast => Ok(ChannelMetadata::Broadcast {
            topic: value.to_owned(),
        }),
        ChannelType::Task => Ok(ChannelMetadata::Task {
            task_id: value.to_owned(),
        }),
        ChannelType::Marketplace => Ok(ChannelMetadata::Marketplace {
            market_scope: value.to_owned(),
        }),
        ChannelType::Governance => Ok(ChannelMetadata::Governance {
            proposal_scope: value.to_owned(),
        }),
    }
}

pub(crate) fn ensure_snapshot_token(
    value: &str,
    field: &str,
) -> Result<(), ChannelSnapshotStoreError> {
    if value.contains('|') || value.contains('\n') || value.contains('\r') || value.contains(',') {
        return Err(ChannelSnapshotStoreError::InvalidPayload(format!(
            "{field} contains unsupported delimiter characters"
        )));
    }
    Ok(())
}

fn parse_empty_metadata(
    value: &str,
    metadata: ChannelMetadata,
) -> Result<ChannelMetadata, ChannelSnapshotStoreError> {
    if !value.is_empty() {
        return Err(ChannelSnapshotStoreError::InvalidPayload(
            "channel metadata payload must be empty".to_owned(),
        ));
    }
    Ok(metadata)
}
