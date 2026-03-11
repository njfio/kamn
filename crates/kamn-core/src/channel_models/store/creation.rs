use super::*;
use crate::ChannelModelError;
use std::collections::BTreeSet;

impl ChannelStore {
    /// Creates an empty in-memory channel store.
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates a direct channel with exactly two participants.
    pub fn create_direct(
        &mut self,
        channel_id: &str,
        participant_a: &str,
        participant_b: &str,
    ) -> Result<(), ChannelModelError> {
        validate_channel_id(channel_id)?;
        self.ensure_channel_not_exists(channel_id)?;
        validate_direct_participants(participant_a, participant_b)?;
        let members = BTreeSet::from([participant_a.to_owned(), participant_b.to_owned()]);
        self.insert_channel(
            channel_id,
            ChannelType::Direct,
            ChannelMetadata::Direct,
            members.clone(),
            members,
        );
        Ok(())
    }

    /// Creates a group channel with the provided members and admins.
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
        validate_non_empty_members(&members)?;
        validate_non_empty_admins(&admins)?;
        let member_set = build_member_set(creator, members)?;
        let admin_set = build_admin_set(creator, admins, &member_set)?;
        self.insert_channel(
            channel_id,
            ChannelType::Group,
            ChannelMetadata::Group,
            member_set,
            admin_set,
        );
        Ok(())
    }
}

fn validate_direct_participants(
    participant_a: &str,
    participant_b: &str,
) -> Result<(), ChannelModelError> {
    validate_did(participant_a)?;
    validate_did(participant_b)?;
    if participant_a == participant_b {
        return Err(ChannelModelError::InvalidDirectParticipants);
    }
    Ok(())
}

fn validate_non_empty_members(members: &[String]) -> Result<(), ChannelModelError> {
    if members.is_empty() {
        return Err(ChannelModelError::EmptyMembers);
    }
    Ok(())
}

fn validate_non_empty_admins(admins: &[String]) -> Result<(), ChannelModelError> {
    if admins.is_empty() {
        return Err(ChannelModelError::EmptyAdmins);
    }
    Ok(())
}

fn build_member_set(
    creator: &str,
    members: Vec<String>,
) -> Result<BTreeSet<String>, ChannelModelError> {
    let mut member_set = BTreeSet::new();
    for member in members {
        validate_did(&member)?;
        member_set.insert(member);
    }
    if !member_set.contains(creator) {
        return Err(ChannelModelError::CreatorNotMember(creator.to_owned()));
    }
    Ok(member_set)
}

fn build_admin_set(
    creator: &str,
    admins: Vec<String>,
    member_set: &BTreeSet<String>,
) -> Result<BTreeSet<String>, ChannelModelError> {
    let mut admin_set = BTreeSet::new();
    for admin in admins {
        validate_did(&admin)?;
        if !member_set.contains(&admin) {
            return Err(ChannelModelError::AdminNotMember(admin));
        }
        admin_set.insert(admin);
    }
    ensure_creator_is_admin(creator, &admin_set)?;
    Ok(admin_set)
}

fn ensure_creator_is_admin(
    creator: &str,
    admin_set: &BTreeSet<String>,
) -> Result<(), ChannelModelError> {
    if !admin_set.contains(creator) {
        return Err(ChannelModelError::UnauthorizedActor {
            actor: creator.to_owned(),
            required: "admin",
        });
    }
    Ok(())
}
