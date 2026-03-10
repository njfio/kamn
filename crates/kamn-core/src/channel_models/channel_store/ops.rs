impl ChannelStore {
    /// Construct an empty channel store.
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a direct channel between exactly two distinct participants.
    pub fn create_direct(
        &mut self,
        channel_id: &str,
        participant_a: &str,
        participant_b: &str,
    ) -> Result<(), ChannelModelError> {
        validate_channel_id(channel_id)?;
        self.ensure_channel_not_exists(channel_id)?;
        validate_did(participant_a)?;
        validate_did(participant_b)?;
        if participant_a == participant_b {
            return Err(ChannelModelError::InvalidDirectParticipants);
        }

        let members = BTreeSet::from([participant_a.to_owned(), participant_b.to_owned()]);
        let admins = members.clone();
        self.insert_channel(
            channel_id,
            ChannelType::Direct,
            ChannelMetadata::Direct,
            members,
            admins,
        );
        Ok(())
    }

    /// Create a group channel with explicit member/admin sets.
    pub fn create_group(
        &mut self,
        channel_id: &str,
        creator: &str,
        members: Vec<String>,
        admins: Vec<String>,
    ) -> Result<(), ChannelModelError> {
        validate_channel_id(channel_id)?;
        self.ensure_channel_not_exists(channel_id)?;
        validate_did(creator)?;
        if members.is_empty() {
            return Err(ChannelModelError::EmptyMembers);
        }
        if admins.is_empty() {
            return Err(ChannelModelError::EmptyAdmins);
        }

        let mut member_set = BTreeSet::new();
        for member in members {
            validate_did(&member)?;
            member_set.insert(member);
        }
        if !member_set.contains(creator) {
            return Err(ChannelModelError::CreatorNotMember(creator.to_owned()));
        }

        let mut admin_set = BTreeSet::new();
        for admin in admins {
            validate_did(&admin)?;
            if !member_set.contains(&admin) {
                return Err(ChannelModelError::AdminNotMember(admin));
            }
            admin_set.insert(admin);
        }
        if !admin_set.contains(creator) {
            return Err(ChannelModelError::UnauthorizedActor {
                actor: creator.to_owned(),
                required: "admin",
            });
        }

        self.insert_channel(
            channel_id,
            ChannelType::Group,
            ChannelMetadata::Group,
            member_set,
            admin_set,
        );
        Ok(())
    }

    /// Create a broadcast channel with topic metadata.
    pub fn create_broadcast(
        &mut self,
        channel_id: &str,
        creator: &str,
        topic: &str,
        members: Vec<String>,
        admins: Vec<String>,
    ) -> Result<(), ChannelModelError> {
        self.create_specialized_channel(
            channel_id,
            creator,
            ChannelType::Broadcast,
            ChannelMetadata::Broadcast {
                topic: topic.to_owned(),
            },
            members,
            admins,
        )
    }

    /// Create a task channel bound to a task identifier.
    pub fn create_task_channel(
        &mut self,
        channel_id: &str,
        creator: &str,
        task_id: &str,
        members: Vec<String>,
        admins: Vec<String>,
    ) -> Result<(), ChannelModelError> {
        self.create_specialized_channel(
            channel_id,
            creator,
            ChannelType::Task,
            ChannelMetadata::Task {
                task_id: task_id.to_owned(),
            },
            members,
            admins,
        )
    }

    /// Create a marketplace channel bound to a market scope.
    pub fn create_marketplace_channel(
        &mut self,
        channel_id: &str,
        creator: &str,
        market_scope: &str,
        members: Vec<String>,
        admins: Vec<String>,
    ) -> Result<(), ChannelModelError> {
        self.create_specialized_channel(
            channel_id,
            creator,
            ChannelType::Marketplace,
            ChannelMetadata::Marketplace {
                market_scope: market_scope.to_owned(),
            },
            members,
            admins,
        )
    }

    /// Create a governance channel bound to a proposal scope.
    pub fn create_governance_channel(
        &mut self,
        channel_id: &str,
        creator: &str,
        proposal_scope: &str,
        members: Vec<String>,
        admins: Vec<String>,
    ) -> Result<(), ChannelModelError> {
        self.create_specialized_channel(
            channel_id,
            creator,
            ChannelType::Governance,
            ChannelMetadata::Governance {
                proposal_scope: proposal_scope.to_owned(),
            },
            members,
            admins,
        )
    }

    fn create_specialized_channel(
        &mut self,
        channel_id: &str,
        creator: &str,
        channel_type: ChannelType,
        metadata: ChannelMetadata,
        members: Vec<String>,
        admins: Vec<String>,
    ) -> Result<(), ChannelModelError> {
        validate_metadata(&metadata)?;
        self.create_group(channel_id, creator, members, admins)?;
        let member_count = self
            .channels
            .get(channel_id)
            .map(|record| record.members.len())
            .ok_or_else(|| ChannelModelError::NotFound(channel_id.to_owned()))?;
        enforce_specialized_member_requirements(channel_type, member_count)?;
        let record = self
            .channels
            .get_mut(channel_id)
            .ok_or_else(|| ChannelModelError::NotFound(channel_id.to_owned()))?;
        record.channel_type = channel_type;
        record.metadata = metadata;
        Ok(())
    }

    /// Look up the channel type for a channel identifier.
    pub fn channel_type(&self, channel_id: &str) -> Result<ChannelType, ChannelModelError> {
        self.channels
            .get(channel_id)
            .map(|record| record.channel_type)
            .ok_or_else(|| ChannelModelError::NotFound(channel_id.to_owned()))
    }

    /// Return all channel members for a channel identifier.
    pub fn members(&self, channel_id: &str) -> Result<Vec<String>, ChannelModelError> {
        let record = self
            .channels
            .get(channel_id)
            .ok_or_else(|| ChannelModelError::NotFound(channel_id.to_owned()))?;
        Ok(record.members.iter().cloned().collect())
    }

    /// Return all channel admins for a channel identifier.
    pub fn admins(&self, channel_id: &str) -> Result<Vec<String>, ChannelModelError> {
        let record = self
            .channels
            .get(channel_id)
            .ok_or_else(|| ChannelModelError::NotFound(channel_id.to_owned()))?;
        Ok(record.admins.iter().cloned().collect())
    }

    /// Return channel IDs where the given DID is currently a member.
    pub fn channels_for_member(&self, member: &str) -> Vec<String> {
        self.channels_by_member
            .get(member)
            .map(|values| values.iter().cloned().collect())
            .unwrap_or_default()
    }

    /// Export all channels into a deterministic snapshot payload.
    pub fn export_snapshot(&self) -> ChannelSnapshot {
        let records = self
            .channels
            .iter()
            .map(|(channel_id, record)| ChannelRecordSnapshot {
                channel_id: channel_id.clone(),
                channel_type: record.channel_type,
                metadata: record.metadata.clone(),
                members: record.members.iter().cloned().collect(),
                admins: record.admins.iter().cloned().collect(),
            })
            .collect();

        ChannelSnapshot {
            schema_version: CHANNEL_SNAPSHOT_SCHEMA_VERSION,
            records,
        }
    }

    /// Restore channel state from a validated snapshot payload.
    pub fn restore_snapshot(
        &mut self,
        snapshot: ChannelSnapshot,
    ) -> Result<(), ChannelSnapshotError> {
        if snapshot.schema_version != CHANNEL_SNAPSHOT_SCHEMA_VERSION {
            return Err(ChannelSnapshotError::SnapshotVersionMismatch {
                expected: CHANNEL_SNAPSHOT_SCHEMA_VERSION,
                found: snapshot.schema_version,
            });
        }

        let mut channels = BTreeMap::new();
        let mut channels_by_member = BTreeMap::new();
        for record_snapshot in snapshot.records {
            if channels.contains_key(&record_snapshot.channel_id) {
                return Err(ChannelSnapshotError::DuplicateChannelId(
                    record_snapshot.channel_id,
                ));
            }

            let record =
                validate_snapshot_record(&record_snapshot).map_err(ChannelSnapshotError::Model)?;

            let channel_id = record_snapshot.channel_id;
            for member in &record.members {
                channels_by_member
                    .entry(member.clone())
                    .or_insert_with(BTreeSet::new)
                    .insert(channel_id.clone());
            }
            channels.insert(channel_id, record);
        }

        self.channels = channels;
        self.channels_by_member = channels_by_member;
        Ok(())
    }

    /// Check whether a DID is currently a member of the given channel.
    pub fn is_member(&self, channel_id: &str, member: &str) -> Result<bool, ChannelModelError> {
        let record = self
            .channels
            .get(channel_id)
            .ok_or_else(|| ChannelModelError::NotFound(channel_id.to_owned()))?;
        Ok(record.members.contains(member))
    }

    /// Return metadata associated with the given channel.
    pub fn metadata(&self, channel_id: &str) -> Result<ChannelMetadata, ChannelModelError> {
        let record = self
            .channels
            .get(channel_id)
            .ok_or_else(|| ChannelModelError::NotFound(channel_id.to_owned()))?;
        Ok(record.metadata.clone())
    }

    /// Invite a new member into a non-direct channel.
    pub fn invite_member(
        &mut self,
        channel_id: &str,
        actor: &str,
        new_member: &str,
    ) -> Result<(), ChannelModelError> {
        validate_did(new_member)?;
        let record = self
            .channels
            .get_mut(channel_id)
            .ok_or_else(|| ChannelModelError::NotFound(channel_id.to_owned()))?;
        if record.channel_type == ChannelType::Direct {
            return Err(ChannelModelError::UnsupportedOperation {
                channel_type: ChannelType::Direct,
                action: "invite_member",
            });
        }
        if !record.admins.contains(actor) {
            return Err(ChannelModelError::UnauthorizedActor {
                actor: actor.to_owned(),
                required: "admin",
            });
        }
        if !record.members.insert(new_member.to_owned()) {
            return Err(ChannelModelError::MemberAlreadyPresent(
                new_member.to_owned(),
            ));
        }

        self.channels_by_member
            .entry(new_member.to_owned())
            .or_default()
            .insert(channel_id.to_owned());
        Ok(())
    }

    /// Remove an existing member from a non-direct channel.
    pub fn remove_member(
        &mut self,
        channel_id: &str,
        actor: &str,
        member: &str,
    ) -> Result<(), ChannelModelError> {
        let record = self
            .channels
            .get_mut(channel_id)
            .ok_or_else(|| ChannelModelError::NotFound(channel_id.to_owned()))?;
        if record.channel_type == ChannelType::Direct {
            return Err(ChannelModelError::UnsupportedOperation {
                channel_type: ChannelType::Direct,
                action: "remove_member",
            });
        }
        if !record.admins.contains(actor) {
            return Err(ChannelModelError::UnauthorizedActor {
                actor: actor.to_owned(),
                required: "admin",
            });
        }
        if !record.members.contains(member) {
            return Err(ChannelModelError::MemberNotFound(member.to_owned()));
        }
        if record.admins.contains(member) && record.admins.len() == 1 {
            return Err(ChannelModelError::LastAdminRemoval(channel_id.to_owned()));
        }

        record.members.remove(member);
        record.admins.remove(member);
        if let Some(channels) = self.channels_by_member.get_mut(member) {
            channels.remove(channel_id);
        }
        Ok(())
    }

    /// Promote an existing member to admin on a non-direct channel.
    pub fn add_admin(
        &mut self,
        channel_id: &str,
        actor: &str,
        member: &str,
    ) -> Result<(), ChannelModelError> {
        let record = self
            .channels
            .get_mut(channel_id)
            .ok_or_else(|| ChannelModelError::NotFound(channel_id.to_owned()))?;
        if record.channel_type == ChannelType::Direct {
            return Err(ChannelModelError::UnsupportedOperation {
                channel_type: ChannelType::Direct,
                action: "add_admin",
            });
        }
        if !record.admins.contains(actor) {
            return Err(ChannelModelError::UnauthorizedActor {
                actor: actor.to_owned(),
                required: "admin",
            });
        }
        if !record.members.contains(member) {
            return Err(ChannelModelError::MemberNotFound(member.to_owned()));
        }

        record.admins.insert(member.to_owned());
        Ok(())
    }

    /// Demote an admin from a non-direct channel while preserving admin quorum.
    pub fn remove_admin(
        &mut self,
        channel_id: &str,
        actor: &str,
        member: &str,
    ) -> Result<(), ChannelModelError> {
        let record = self
            .channels
            .get_mut(channel_id)
            .ok_or_else(|| ChannelModelError::NotFound(channel_id.to_owned()))?;
        if record.channel_type == ChannelType::Direct {
            return Err(ChannelModelError::UnsupportedOperation {
                channel_type: ChannelType::Direct,
                action: "remove_admin",
            });
        }
        if !record.admins.contains(actor) {
            return Err(ChannelModelError::UnauthorizedActor {
                actor: actor.to_owned(),
                required: "admin",
            });
        }
        if !record.admins.contains(member) {
            return Err(ChannelModelError::AdminNotFound(member.to_owned()));
        }
        if record.admins.len() == 1 {
            return Err(ChannelModelError::LastAdminRemoval(channel_id.to_owned()));
        }

        record.admins.remove(member);
        Ok(())
    }

    fn ensure_channel_not_exists(&self, channel_id: &str) -> Result<(), ChannelModelError> {
        if self.channels.contains_key(channel_id) {
            return Err(ChannelModelError::DuplicateChannelId(channel_id.to_owned()));
        }
        Ok(())
    }

    fn insert_channel(
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

/// Errors emitted by channel creation, membership, and metadata workflows.
fn validate_channel_id(channel_id: &str) -> Result<(), ChannelModelError> {
    if channel_id.trim().is_empty() {
        return Err(ChannelModelError::EmptyChannelId);
    }
    Ok(())
}

fn validate_did(value: &str) -> Result<(), ChannelModelError> {
    AgentDid::parse(value).map_err(|error| ChannelModelError::InvalidDid(error.to_string()))?;
    Ok(())
}

fn validate_metadata(metadata: &ChannelMetadata) -> Result<(), ChannelModelError> {
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

fn enforce_specialized_member_requirements(
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

fn metadata_matches_channel_type(channel_type: ChannelType, metadata: &ChannelMetadata) -> bool {
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

fn validate_snapshot_record(
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
