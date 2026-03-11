use super::*;
use crate::ChannelModelError;

impl ChannelStore {
    /// Creates a broadcast channel with topic metadata.
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

    /// Creates a task channel with task metadata.
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

    /// Creates a marketplace channel with marketplace metadata.
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

    /// Creates a governance channel with proposal-scope metadata.
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
        let member_count = self.channel_member_count(channel_id)?;
        enforce_specialized_member_requirements(channel_type, member_count)?;
        let record = self.channel_record_mut(channel_id)?;
        record.channel_type = channel_type;
        record.metadata = metadata;
        Ok(())
    }

    fn channel_member_count(&self, channel_id: &str) -> Result<usize, ChannelModelError> {
        Ok(self.channel_record(channel_id)?.members.len())
    }

    fn channel_record(&self, channel_id: &str) -> Result<&ChannelRecord, ChannelModelError> {
        self.channels
            .get(channel_id)
            .ok_or_else(|| ChannelModelError::NotFound(channel_id.to_owned()))
    }

    fn channel_record_mut(
        &mut self,
        channel_id: &str,
    ) -> Result<&mut ChannelRecord, ChannelModelError> {
        self.channels
            .get_mut(channel_id)
            .ok_or_else(|| ChannelModelError::NotFound(channel_id.to_owned()))
    }
}
