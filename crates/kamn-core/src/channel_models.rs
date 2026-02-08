use crate::AgentDid;
use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChannelType {
    Direct,
    Group,
    Broadcast,
    Task,
    Marketplace,
    Governance,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ChannelMetadata {
    Direct,
    Group,
    Broadcast { topic: String },
    Task { task_id: String },
    Marketplace { market_scope: String },
    Governance { proposal_scope: String },
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ChannelRecord {
    channel_type: ChannelType,
    metadata: ChannelMetadata,
    members: BTreeSet<String>,
    admins: BTreeSet<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ChannelStore {
    channels: BTreeMap<String, ChannelRecord>,
    channels_by_member: BTreeMap<String, BTreeSet<String>>,
}

impl ChannelStore {
    pub fn new() -> Self {
        Self::default()
    }

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

    pub fn channel_type(&self, channel_id: &str) -> Result<ChannelType, ChannelModelError> {
        self.channels
            .get(channel_id)
            .map(|record| record.channel_type)
            .ok_or_else(|| ChannelModelError::NotFound(channel_id.to_owned()))
    }

    pub fn members(&self, channel_id: &str) -> Result<Vec<String>, ChannelModelError> {
        let record = self
            .channels
            .get(channel_id)
            .ok_or_else(|| ChannelModelError::NotFound(channel_id.to_owned()))?;
        Ok(record.members.iter().cloned().collect())
    }

    pub fn admins(&self, channel_id: &str) -> Result<Vec<String>, ChannelModelError> {
        let record = self
            .channels
            .get(channel_id)
            .ok_or_else(|| ChannelModelError::NotFound(channel_id.to_owned()))?;
        Ok(record.admins.iter().cloned().collect())
    }

    pub fn channels_for_member(&self, member: &str) -> Vec<String> {
        self.channels_by_member
            .get(member)
            .map(|values| values.iter().cloned().collect())
            .unwrap_or_default()
    }

    pub fn is_member(&self, channel_id: &str, member: &str) -> Result<bool, ChannelModelError> {
        let record = self
            .channels
            .get(channel_id)
            .ok_or_else(|| ChannelModelError::NotFound(channel_id.to_owned()))?;
        Ok(record.members.contains(member))
    }

    pub fn metadata(&self, channel_id: &str) -> Result<ChannelMetadata, ChannelModelError> {
        let record = self
            .channels
            .get(channel_id)
            .ok_or_else(|| ChannelModelError::NotFound(channel_id.to_owned()))?;
        Ok(record.metadata.clone())
    }

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

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ChannelModelError {
    EmptyChannelId,
    InvalidDid(String),
    DuplicateChannelId(String),
    InvalidDirectParticipants,
    EmptyMembers,
    EmptyAdmins,
    InvalidMetadata(String),
    InsufficientMembers {
        channel_type: ChannelType,
        minimum: usize,
        actual: usize,
    },
    CreatorNotMember(String),
    AdminNotMember(String),
    UnauthorizedActor {
        actor: String,
        required: &'static str,
    },
    NotFound(String),
    MemberAlreadyPresent(String),
    MemberNotFound(String),
    AdminNotFound(String),
    LastAdminRemoval(String),
    UnsupportedOperation {
        channel_type: ChannelType,
        action: &'static str,
    },
}

impl fmt::Display for ChannelModelError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyChannelId => write!(f, "channel_id must not be empty"),
            Self::InvalidDid(value) => write!(f, "invalid channel DID: {value}"),
            Self::DuplicateChannelId(value) => write!(f, "duplicate channel id: {value}"),
            Self::InvalidDirectParticipants => {
                write!(f, "direct channels require two distinct participants")
            }
            Self::EmptyMembers => write!(f, "group channel members must not be empty"),
            Self::EmptyAdmins => write!(f, "group channel admins must not be empty"),
            Self::InvalidMetadata(value) => write!(f, "invalid channel metadata: {value}"),
            Self::InsufficientMembers {
                channel_type,
                minimum,
                actual,
            } => write!(
                f,
                "channel type {channel_type:?} requires at least {minimum} members, found {actual}"
            ),
            Self::CreatorNotMember(value) => write!(f, "creator must be a member: {value}"),
            Self::AdminNotMember(value) => write!(f, "admin must be a member: {value}"),
            Self::UnauthorizedActor { actor, required } => {
                write!(f, "unauthorized actor {actor}, requires {required}")
            }
            Self::NotFound(value) => write!(f, "channel not found: {value}"),
            Self::MemberAlreadyPresent(value) => write!(f, "member already present: {value}"),
            Self::MemberNotFound(value) => write!(f, "member not found: {value}"),
            Self::AdminNotFound(value) => write!(f, "admin not found: {value}"),
            Self::LastAdminRemoval(value) => write!(f, "cannot remove last admin from {value}"),
            Self::UnsupportedOperation {
                channel_type,
                action,
            } => write!(
                f,
                "unsupported operation {action} for channel type {channel_type:?}"
            ),
        }
    }
}

impl std::error::Error for ChannelModelError {}

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

#[cfg(test)]
mod tests {
    use super::{ChannelMetadata, ChannelModelError, ChannelStore, ChannelType};

    #[test]
    fn group_creator_must_be_member() {
        let mut store = ChannelStore::new();
        assert_eq!(
            store.create_group(
                "channel:group:1",
                "kamn:did:agent:owner",
                vec!["kamn:did:agent:member-1".to_owned()],
                vec!["kamn:did:agent:member-1".to_owned()],
            ),
            Err(ChannelModelError::CreatorNotMember(
                "kamn:did:agent:owner".to_owned()
            ))
        );
    }

    #[test]
    fn direct_channels_require_distinct_participants() {
        let mut store = ChannelStore::new();
        assert_eq!(
            store.create_direct(
                "channel:direct:1",
                "kamn:did:agent:alice",
                "kamn:did:agent:alice",
            ),
            Err(ChannelModelError::InvalidDirectParticipants)
        );
    }

    #[test]
    fn governance_channels_require_three_members() {
        let mut store = ChannelStore::new();
        assert_eq!(
            store.create_governance_channel(
                "channel:gov:1",
                "kamn:did:agent:owner",
                "core-protocol",
                vec![
                    "kamn:did:agent:owner".to_owned(),
                    "kamn:did:agent:validator-1".to_owned(),
                ],
                vec!["kamn:did:agent:owner".to_owned()],
            ),
            Err(ChannelModelError::InsufficientMembers {
                channel_type: ChannelType::Governance,
                minimum: 3,
                actual: 2,
            })
        );
    }

    #[test]
    fn broadcast_metadata_requires_non_empty_topic() {
        let mut store = ChannelStore::new();
        assert_eq!(
            store.create_broadcast(
                "channel:broadcast:1",
                "kamn:did:agent:owner",
                "",
                vec!["kamn:did:agent:owner".to_owned()],
                vec!["kamn:did:agent:owner".to_owned()],
            ),
            Err(ChannelModelError::InvalidMetadata(
                "topic must not be empty".to_owned()
            ))
        );

        store
            .create_broadcast(
                "channel:broadcast:2",
                "kamn:did:agent:owner",
                "announcements",
                vec!["kamn:did:agent:owner".to_owned()],
                vec!["kamn:did:agent:owner".to_owned()],
            )
            .expect("broadcast should be created");

        assert_eq!(
            store
                .metadata("channel:broadcast:2")
                .expect("metadata should resolve"),
            ChannelMetadata::Broadcast {
                topic: "announcements".to_owned(),
            }
        );
    }
}
