use super::validation::{
    enforce_specialized_member_requirements, validate_channel_id, validate_did, validate_metadata,
};
use super::*;

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
}
