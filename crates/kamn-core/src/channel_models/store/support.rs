use super::*;
use crate::AgentDid;
use crate::ChannelModelError;
use std::collections::BTreeSet;

pub(crate) fn validate_channel_id(channel_id: &str) -> Result<(), ChannelModelError> {
    if channel_id.trim().is_empty() {
        return Err(ChannelModelError::EmptyChannelId);
    }
    Ok(())
}

pub(crate) fn validate_did(value: &str) -> Result<(), ChannelModelError> {
    AgentDid::parse(value).map_err(|error| ChannelModelError::InvalidDid(error.to_string()))?;
    Ok(())
}

pub(crate) fn validate_metadata(metadata: &ChannelMetadata) -> Result<(), ChannelModelError> {
    let invalid = match metadata {
        ChannelMetadata::Broadcast { topic } if topic.trim().is_empty() => Some("topic"),
        ChannelMetadata::Task { task_id } if task_id.trim().is_empty() => Some("task_id"),
        ChannelMetadata::Marketplace { market_scope } if market_scope.trim().is_empty() => {
            Some("market_scope")
        }
        ChannelMetadata::Governance { proposal_scope } if proposal_scope.trim().is_empty() => {
            Some("proposal_scope")
        }
        _ => None,
    };
    invalid.map(invalid_metadata_error).map_or(Ok(()), Err)
}

pub(crate) fn invalid_metadata_error(field: &str) -> ChannelModelError {
    ChannelModelError::InvalidMetadata(format!("{field} must not be empty"))
}

pub(crate) fn enforce_specialized_member_requirements(
    channel_type: ChannelType,
    actual: usize,
) -> Result<(), ChannelModelError> {
    let minimum = minimum_members(channel_type);
    if actual < minimum {
        return Err(ChannelModelError::InsufficientMembers {
            channel_type,
            minimum,
            actual,
        });
    }
    Ok(())
}

pub(crate) fn metadata_matches_channel_type(
    channel_type: ChannelType,
    metadata: &ChannelMetadata,
) -> bool {
    matches!(
        (channel_type, metadata),
        (ChannelType::Direct, ChannelMetadata::Direct)
            | (ChannelType::Group, ChannelMetadata::Group)
            | (ChannelType::Broadcast, ChannelMetadata::Broadcast { .. })
            | (ChannelType::Task, ChannelMetadata::Task { .. })
            | (
                ChannelType::Marketplace,
                ChannelMetadata::Marketplace { .. }
            )
            | (ChannelType::Governance, ChannelMetadata::Governance { .. })
    )
}

impl ChannelStore {
    pub(crate) fn ensure_channel_not_exists(
        &self,
        channel_id: &str,
    ) -> Result<(), ChannelModelError> {
        if self.channels.contains_key(channel_id) {
            return Err(ChannelModelError::DuplicateChannelId(channel_id.to_owned()));
        }
        Ok(())
    }

    pub(crate) fn insert_channel(
        &mut self,
        channel_id: &str,
        channel_type: ChannelType,
        metadata: ChannelMetadata,
        members: BTreeSet<String>,
        admins: BTreeSet<String>,
    ) {
        self.channels.insert(
            channel_id.to_owned(),
            ChannelRecord {
                channel_type,
                metadata,
                members: members.clone(),
                admins,
            },
        );
        index_channel_members(&mut self.channels_by_member, channel_id, members);
    }
}

fn minimum_members(channel_type: ChannelType) -> usize {
    match channel_type {
        ChannelType::Task | ChannelType::Marketplace => 2,
        ChannelType::Governance => 3,
        _ => 1,
    }
}

fn index_channel_members(
    channels_by_member: &mut std::collections::BTreeMap<String, BTreeSet<String>>,
    channel_id: &str,
    members: BTreeSet<String>,
) {
    for member in members {
        channels_by_member
            .entry(member)
            .or_default()
            .insert(channel_id.to_owned());
    }
}
