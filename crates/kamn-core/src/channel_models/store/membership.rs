use super::*;
use crate::ChannelModelError;

impl ChannelStore {
    /// Returns the channel type for the requested channel identifier.
    pub fn channel_type(&self, channel_id: &str) -> Result<ChannelType, ChannelModelError> {
        Ok(self.record(channel_id)?.channel_type)
    }

    /// Returns the current member set for a channel.
    pub fn members(&self, channel_id: &str) -> Result<Vec<String>, ChannelModelError> {
        Ok(self.record(channel_id)?.members.iter().cloned().collect())
    }

    /// Returns the current admin set for a channel.
    pub fn admins(&self, channel_id: &str) -> Result<Vec<String>, ChannelModelError> {
        Ok(self.record(channel_id)?.admins.iter().cloned().collect())
    }

    /// Returns every channel currently indexed for a member DID.
    pub fn channels_for_member(&self, member: &str) -> Vec<String> {
        self.channels_by_member
            .get(member)
            .map(|values| values.iter().cloned().collect())
            .unwrap_or_default()
    }

    /// Reports whether the supplied DID is a member of the channel.
    pub fn is_member(&self, channel_id: &str, member: &str) -> Result<bool, ChannelModelError> {
        Ok(self.record(channel_id)?.members.contains(member))
    }

    /// Returns the metadata attached to a channel.
    pub fn metadata(&self, channel_id: &str) -> Result<ChannelMetadata, ChannelModelError> {
        Ok(self.record(channel_id)?.metadata.clone())
    }

    /// Invites a new member into a non-direct channel.
    pub fn invite_member(
        &mut self,
        channel_id: &str,
        actor: &str,
        new_member: &str,
    ) -> Result<(), ChannelModelError> {
        validate_did(new_member)?;
        let record = self.record_mut(channel_id)?;
        ensure_non_direct_channel(record.channel_type, "invite_member")?;
        ensure_admin(record, actor)?;
        if !record.members.insert(new_member.to_owned()) {
            return Err(ChannelModelError::MemberAlreadyPresent(
                new_member.to_owned(),
            ));
        }
        add_channel_member_index(&mut self.channels_by_member, new_member, channel_id);
        Ok(())
    }

    /// Removes a member from a non-direct channel.
    pub fn remove_member(
        &mut self,
        channel_id: &str,
        actor: &str,
        member: &str,
    ) -> Result<(), ChannelModelError> {
        {
            let record = self.record_mut(channel_id)?;
            ensure_non_direct_channel(record.channel_type, "remove_member")?;
            ensure_admin(record, actor)?;
            ensure_member_present(record, member)?;
            ensure_not_last_admin(record, member, channel_id)?;
            record.members.remove(member);
            record.admins.remove(member);
        }
        remove_channel_member_index(&mut self.channels_by_member, member, channel_id);
        Ok(())
    }

    /// Promotes a member to admin in a non-direct channel.
    pub fn add_admin(
        &mut self,
        channel_id: &str,
        actor: &str,
        member: &str,
    ) -> Result<(), ChannelModelError> {
        let record = self.record_mut(channel_id)?;
        ensure_non_direct_channel(record.channel_type, "add_admin")?;
        ensure_admin(record, actor)?;
        ensure_member_present(record, member)?;
        record.admins.insert(member.to_owned());
        Ok(())
    }

    /// Removes an admin from a non-direct channel.
    pub fn remove_admin(
        &mut self,
        channel_id: &str,
        actor: &str,
        member: &str,
    ) -> Result<(), ChannelModelError> {
        let record = self.record_mut(channel_id)?;
        ensure_non_direct_channel(record.channel_type, "remove_admin")?;
        ensure_admin(record, actor)?;
        ensure_admin_present(record, member)?;
        ensure_admin_quorum(record, channel_id)?;
        record.admins.remove(member);
        Ok(())
    }

    fn record(&self, channel_id: &str) -> Result<&ChannelRecord, ChannelModelError> {
        self.channels
            .get(channel_id)
            .ok_or_else(|| ChannelModelError::NotFound(channel_id.to_owned()))
    }

    fn record_mut(&mut self, channel_id: &str) -> Result<&mut ChannelRecord, ChannelModelError> {
        self.channels
            .get_mut(channel_id)
            .ok_or_else(|| ChannelModelError::NotFound(channel_id.to_owned()))
    }
}

fn ensure_non_direct_channel(
    channel_type: ChannelType,
    action: &'static str,
) -> Result<(), ChannelModelError> {
    if channel_type == ChannelType::Direct {
        return Err(ChannelModelError::UnsupportedOperation {
            channel_type,
            action,
        });
    }
    Ok(())
}

fn ensure_admin(record: &ChannelRecord, actor: &str) -> Result<(), ChannelModelError> {
    if !record.admins.contains(actor) {
        return Err(ChannelModelError::UnauthorizedActor {
            actor: actor.to_owned(),
            required: "admin",
        });
    }
    Ok(())
}

fn ensure_member_present(record: &ChannelRecord, member: &str) -> Result<(), ChannelModelError> {
    if !record.members.contains(member) {
        return Err(ChannelModelError::MemberNotFound(member.to_owned()));
    }
    Ok(())
}

fn ensure_admin_present(record: &ChannelRecord, member: &str) -> Result<(), ChannelModelError> {
    if !record.admins.contains(member) {
        return Err(ChannelModelError::AdminNotFound(member.to_owned()));
    }
    Ok(())
}

fn ensure_not_last_admin(
    record: &ChannelRecord,
    member: &str,
    channel_id: &str,
) -> Result<(), ChannelModelError> {
    if record.admins.contains(member) && record.admins.len() == 1 {
        return Err(ChannelModelError::LastAdminRemoval(channel_id.to_owned()));
    }
    Ok(())
}

fn ensure_admin_quorum(record: &ChannelRecord, channel_id: &str) -> Result<(), ChannelModelError> {
    if record.admins.len() == 1 {
        return Err(ChannelModelError::LastAdminRemoval(channel_id.to_owned()));
    }
    Ok(())
}

fn add_channel_member_index(
    channels_by_member: &mut std::collections::BTreeMap<String, std::collections::BTreeSet<String>>,
    member: &str,
    channel_id: &str,
) {
    channels_by_member
        .entry(member.to_owned())
        .or_default()
        .insert(channel_id.to_owned());
}

fn remove_channel_member_index(
    channels_by_member: &mut std::collections::BTreeMap<String, std::collections::BTreeSet<String>>,
    member: &str,
    channel_id: &str,
) {
    if let Some(channels) = channels_by_member.get_mut(member) {
        channels.remove(channel_id);
    }
}
