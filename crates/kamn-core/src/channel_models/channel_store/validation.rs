use super::*;

impl ChannelStore {
    pub(super) fn ensure_channel_not_exists(
        &self,
        channel_id: &str,
    ) -> Result<(), ChannelModelError> {
        if self.channels.contains_key(channel_id) {
            return Err(ChannelModelError::DuplicateChannelId(channel_id.to_owned()));
        }
        Ok(())
    }

    pub(super) fn insert_channel(
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
        for member in members {
            self.channels_by_member
                .entry(member)
                .or_default()
                .insert(channel_id.to_owned());
        }
    }
}

pub(super) fn validate_channel_id(channel_id: &str) -> Result<(), ChannelModelError> {
    if channel_id.trim().is_empty() {
        return Err(ChannelModelError::EmptyChannelId);
    }
    Ok(())
}

pub(super) fn validate_did(value: &str) -> Result<(), ChannelModelError> {
    AgentDid::parse(value).map_err(|error| ChannelModelError::InvalidDid(error.to_string()))?;
    Ok(())
}

pub(super) fn validate_metadata(metadata: &ChannelMetadata) -> Result<(), ChannelModelError> {
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

    if let Some(field) = invalid {
        return Err(ChannelModelError::InvalidMetadata(format!(
            "{field} must not be empty"
        )));
    }
    Ok(())
}

pub(super) fn enforce_specialized_member_requirements(
    channel_type: ChannelType,
    actual: usize,
) -> Result<(), ChannelModelError> {
    let minimum = match channel_type {
        ChannelType::Task | ChannelType::Marketplace => 2,
        ChannelType::Governance => 3,
        _ => 1,
    };

    if actual < minimum {
        return Err(ChannelModelError::InsufficientMembers {
            channel_type,
            minimum,
            actual,
        });
    }
    Ok(())
}

pub(super) fn metadata_matches_channel_type(
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

pub(super) fn validate_snapshot_record(
    record: &ChannelRecordSnapshot,
) -> Result<ChannelRecord, ChannelModelError> {
    validate_channel_id(&record.channel_id)?;
    if !metadata_matches_channel_type(record.channel_type, &record.metadata) {
        return Err(ChannelModelError::InvalidMetadata(
            "channel type and metadata variant mismatch".to_owned(),
        ));
    }
    validate_metadata(&record.metadata)?;
    if record.members.is_empty() {
        return Err(ChannelModelError::EmptyMembers);
    }
    if record.admins.is_empty() {
        return Err(ChannelModelError::EmptyAdmins);
    }

    let mut members = BTreeSet::new();
    for member in &record.members {
        validate_did(member)?;
        if !members.insert(member.clone()) {
            return Err(ChannelModelError::InvalidMetadata(
                "duplicate member DID in snapshot".to_owned(),
            ));
        }
    }

    let mut admins = BTreeSet::new();
    for admin in &record.admins {
        validate_did(admin)?;
        if !members.contains(admin) {
            return Err(ChannelModelError::AdminNotMember(admin.clone()));
        }
        if !admins.insert(admin.clone()) {
            return Err(ChannelModelError::InvalidMetadata(
                "duplicate admin DID in snapshot".to_owned(),
            ));
        }
    }

    if record.channel_type == ChannelType::Direct {
        if members.len() != 2 || admins != members {
            return Err(ChannelModelError::InvalidDirectParticipants);
        }
    } else {
        enforce_specialized_member_requirements(record.channel_type, members.len())?;
    }

    Ok(ChannelRecord {
        channel_type: record.channel_type,
        metadata: record.metadata.clone(),
        members,
        admins,
    })
}
