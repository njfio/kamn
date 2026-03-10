use super::validation::validate_did;
use super::*;

impl ChannelStore {
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
}
