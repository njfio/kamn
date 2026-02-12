//! Channel-level authorization and retention policy contracts.

use crate::AgentDid;
use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

/// Schema version used for serialized [`ChannelPolicySnapshot`] payloads.
pub const CHANNEL_POLICY_SNAPSHOT_SCHEMA_VERSION: u16 = 1;

/// Channel operations that require permission checks.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChannelAction {
    /// Send a message into the channel.
    Send,
    /// Read message history from the channel.
    Read,
    /// Invite a new member into the channel.
    Invite,
    /// Remove an existing member from the channel.
    Remove,
    /// Update channel policy configuration.
    Configure,
}

/// Authorization rule used to evaluate a [`ChannelAction`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PermissionRule {
    /// Allow any actor, regardless of membership.
    All,
    /// Allow only registered channel members.
    Members,
    /// Allow only channel administrators.
    Admins,
    /// Allow only explicitly listed actor DIDs.
    Allowlist(BTreeSet<String>),
}

/// Retention strategy applied to channel message history.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RetentionPolicy {
    /// Keep all messages indefinitely.
    Forever,
    /// Retain messages no older than the given age window in seconds.
    MaxAgeSeconds(u64),
    /// Keep only the most recent `N` messages.
    MaxMessageCount(usize),
}

/// Permission configuration for every supported channel action.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChannelPermissions {
    /// Rule used for [`ChannelAction::Send`].
    pub send: PermissionRule,
    /// Rule used for [`ChannelAction::Read`].
    pub read: PermissionRule,
    /// Rule used for [`ChannelAction::Invite`].
    pub invite: PermissionRule,
    /// Rule used for [`ChannelAction::Remove`].
    pub remove: PermissionRule,
    /// Rule used for [`ChannelAction::Configure`].
    pub configure: PermissionRule,
    /// Retention policy used for pruning candidate evaluation.
    pub retention: RetentionPolicy,
}

/// Message metadata used by retention evaluation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RetentionMessage {
    /// Message identifier unique within the channel.
    pub id: String,
    /// Message creation timestamp in Unix seconds.
    pub created_at_secs: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ChannelPolicyRecord {
    members: BTreeSet<String>,
    admins: BTreeSet<String>,
    permissions: ChannelPermissions,
}

/// Serializable snapshot of all registered channel policy records.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChannelPolicySnapshot {
    /// Snapshot schema version for compatibility checks.
    pub schema_version: u16,
    /// Serialized channel policy records.
    pub channels: Vec<ChannelPolicySnapshotChannel>,
}

/// Serializable policy record for a single channel.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChannelPolicySnapshotChannel {
    /// Unique channel identifier.
    pub channel_id: String,
    /// Member DID set serialized as a sorted list.
    pub members: Vec<String>,
    /// Admin DID set serialized as a sorted list.
    pub admins: Vec<String>,
    /// Permission and retention rules for the channel.
    pub permissions: ChannelPermissions,
}

/// In-memory registry that enforces channel permission and retention contracts.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ChannelPermissionEngine {
    channels: BTreeMap<String, ChannelPolicyRecord>,
}

impl ChannelPermissionEngine {
    /// Create an empty permission engine.
    pub fn new() -> Self {
        Self::default()
    }

    /// Register a channel with members, admins, and policy rules.
    ///
    /// Returns an error when identifiers are invalid, members/admins are empty,
    /// admins are not members, or permission/retention rules fail validation.
    pub fn register_channel(
        &mut self,
        channel_id: &str,
        members: Vec<String>,
        admins: Vec<String>,
        permissions: ChannelPermissions,
    ) -> Result<(), ChannelPolicyError> {
        if channel_id.trim().is_empty() {
            return Err(ChannelPolicyError::EmptyChannelId);
        }
        if self.channels.contains_key(channel_id) {
            return Err(ChannelPolicyError::DuplicateChannelId(
                channel_id.to_owned(),
            ));
        }
        if members.is_empty() {
            return Err(ChannelPolicyError::EmptyMembers);
        }
        if admins.is_empty() {
            return Err(ChannelPolicyError::EmptyAdmins);
        }

        validate_retention_policy(&permissions.retention)?;
        validate_permission_rules(&permissions)?;

        let mut member_set = BTreeSet::new();
        for member in members {
            validate_did(&member)?;
            member_set.insert(member);
        }

        let mut admin_set = BTreeSet::new();
        for admin in admins {
            validate_did(&admin)?;
            if !member_set.contains(&admin) {
                return Err(ChannelPolicyError::AdminNotMember(admin));
            }
            admin_set.insert(admin);
        }

        self.channels.insert(
            channel_id.to_owned(),
            ChannelPolicyRecord {
                members: member_set,
                admins: admin_set,
                permissions,
            },
        );
        Ok(())
    }

    /// Authorize an actor for a channel action using stored policy rules.
    pub fn authorize(
        &self,
        channel_id: &str,
        actor: &str,
        action: ChannelAction,
    ) -> Result<(), ChannelPolicyError> {
        validate_did(actor)?;
        let record = self
            .channels
            .get(channel_id)
            .ok_or_else(|| ChannelPolicyError::NotFound(channel_id.to_owned()))?;
        let rule = match action {
            ChannelAction::Send => &record.permissions.send,
            ChannelAction::Read => &record.permissions.read,
            ChannelAction::Invite => &record.permissions.invite,
            ChannelAction::Remove => &record.permissions.remove,
            ChannelAction::Configure => &record.permissions.configure,
        };

        if is_authorized(rule, actor, &record.members, &record.admins) {
            Ok(())
        } else {
            Err(ChannelPolicyError::Unauthorized {
                actor: actor.to_owned(),
                action,
                rule: rule.clone(),
            })
        }
    }

    /// Compute message IDs that are eligible for pruning under retention policy.
    pub fn retention_candidates(
        &self,
        channel_id: &str,
        now_secs: u64,
        mut messages: Vec<RetentionMessage>,
    ) -> Result<Vec<String>, ChannelPolicyError> {
        let record = self
            .channels
            .get(channel_id)
            .ok_or_else(|| ChannelPolicyError::NotFound(channel_id.to_owned()))?;
        match record.permissions.retention {
            RetentionPolicy::Forever => Ok(Vec::new()),
            RetentionPolicy::MaxAgeSeconds(max_age) => {
                let mut candidates: Vec<String> = messages
                    .into_iter()
                    .filter(|message| now_secs.saturating_sub(message.created_at_secs) > max_age)
                    .map(|message| message.id)
                    .collect();
                candidates.sort();
                Ok(candidates)
            }
            RetentionPolicy::MaxMessageCount(limit) => {
                if messages.len() <= limit {
                    return Ok(Vec::new());
                }
                messages.sort_by(|left, right| {
                    left.created_at_secs
                        .cmp(&right.created_at_secs)
                        .then_with(|| left.id.cmp(&right.id))
                });
                let prune_count = messages.len() - limit;
                Ok(messages
                    .into_iter()
                    .take(prune_count)
                    .map(|message| message.id)
                    .collect())
            }
        }
    }

    /// Export the current engine state into a serializable snapshot payload.
    pub fn export_snapshot(&self) -> ChannelPolicySnapshot {
        let channels = self
            .channels
            .iter()
            .map(|(channel_id, record)| ChannelPolicySnapshotChannel {
                channel_id: channel_id.clone(),
                members: record.members.iter().cloned().collect(),
                admins: record.admins.iter().cloned().collect(),
                permissions: record.permissions.clone(),
            })
            .collect();

        ChannelPolicySnapshot {
            schema_version: CHANNEL_POLICY_SNAPSHOT_SCHEMA_VERSION,
            channels,
        }
    }

    /// Restore engine state from a snapshot after schema and policy validation.
    pub fn restore_snapshot(
        &mut self,
        snapshot: ChannelPolicySnapshot,
    ) -> Result<(), ChannelPolicySnapshotError> {
        if snapshot.schema_version != CHANNEL_POLICY_SNAPSHOT_SCHEMA_VERSION {
            return Err(ChannelPolicySnapshotError::SchemaVersionMismatch {
                expected: CHANNEL_POLICY_SNAPSHOT_SCHEMA_VERSION,
                found: snapshot.schema_version,
            });
        }

        let mut restored = ChannelPermissionEngine::new();
        for channel in snapshot.channels {
            restored.register_channel(
                &channel.channel_id,
                channel.members,
                channel.admins,
                channel.permissions,
            )?;
        }

        self.channels = restored.channels;
        Ok(())
    }

    /// Construct a new engine from a snapshot payload.
    pub fn from_snapshot(
        snapshot: ChannelPolicySnapshot,
    ) -> Result<Self, ChannelPolicySnapshotError> {
        let mut engine = Self::new();
        engine.restore_snapshot(snapshot)?;
        Ok(engine)
    }
}

/// Errors emitted by channel registration, authorization, and retention checks.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ChannelPolicyError {
    /// Registration rejected because `channel_id` is empty.
    EmptyChannelId,
    /// Registration rejected because the channel already exists.
    DuplicateChannelId(String),
    /// Registration rejected because the members list is empty.
    EmptyMembers,
    /// Registration rejected because the admins list is empty.
    EmptyAdmins,
    /// An actor or member/admin DID failed validation.
    InvalidDid(String),
    /// Registration rejected because an admin is not in the members set.
    AdminNotMember(String),
    /// Channel lookup failed because no policy record exists.
    NotFound(String),
    /// Action authorization failed for the actor under the configured rule.
    Unauthorized {
        /// DID of the actor that failed authorization.
        actor: String,
        /// Action that was denied.
        action: ChannelAction,
        /// Permission rule evaluated for this action.
        rule: PermissionRule,
    },
    /// Permission configuration failed rule-level validation.
    InvalidPermissionRule(String),
    /// Retention policy failed policy-level validation.
    InvalidRetentionPolicy(String),
}

/// Errors emitted while restoring or loading channel policy snapshots.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ChannelPolicySnapshotError {
    /// Snapshot schema version does not match the runtime expectation.
    SchemaVersionMismatch {
        /// Schema version required by the running engine.
        expected: u16,
        /// Schema version provided by the snapshot payload.
        found: u16,
    },
    /// Snapshot channel record failed normal channel policy validation.
    ChannelPolicy(ChannelPolicyError),
}

impl fmt::Display for ChannelPolicyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyChannelId => write!(f, "channel_id must not be empty"),
            Self::DuplicateChannelId(value) => write!(f, "duplicate channel id: {value}"),
            Self::EmptyMembers => write!(f, "members must not be empty"),
            Self::EmptyAdmins => write!(f, "admins must not be empty"),
            Self::InvalidDid(value) => write!(f, "invalid did: {value}"),
            Self::AdminNotMember(value) => write!(f, "admin must be a member: {value}"),
            Self::NotFound(value) => write!(f, "channel not found: {value}"),
            Self::Unauthorized {
                actor,
                action,
                rule,
            } => write!(
                f,
                "actor {actor} is unauthorized for action {action:?} under rule {rule:?}"
            ),
            Self::InvalidPermissionRule(value) => write!(f, "invalid permission rule: {value}"),
            Self::InvalidRetentionPolicy(value) => write!(f, "invalid retention policy: {value}"),
        }
    }
}

impl std::error::Error for ChannelPolicyError {}

impl fmt::Display for ChannelPolicySnapshotError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::SchemaVersionMismatch { expected, found } => write!(
                f,
                "channel policy snapshot schema version mismatch, expected {expected}, found {found}"
            ),
            Self::ChannelPolicy(error) => write!(f, "{error}"),
        }
    }
}

impl std::error::Error for ChannelPolicySnapshotError {}

impl From<ChannelPolicyError> for ChannelPolicySnapshotError {
    fn from(value: ChannelPolicyError) -> Self {
        Self::ChannelPolicy(value)
    }
}

fn validate_did(value: &str) -> Result<(), ChannelPolicyError> {
    AgentDid::parse(value).map_err(|error| ChannelPolicyError::InvalidDid(error.to_string()))?;
    Ok(())
}

fn validate_retention_policy(policy: &RetentionPolicy) -> Result<(), ChannelPolicyError> {
    match policy {
        RetentionPolicy::MaxMessageCount(0) => Err(ChannelPolicyError::InvalidRetentionPolicy(
            "max message count must be greater than zero".to_owned(),
        )),
        _ => Ok(()),
    }
}

fn validate_permission_rules(permissions: &ChannelPermissions) -> Result<(), ChannelPolicyError> {
    validate_permission_rule("send", &permissions.send)?;
    validate_permission_rule("read", &permissions.read)?;
    validate_permission_rule("invite", &permissions.invite)?;
    validate_permission_rule("remove", &permissions.remove)?;
    validate_permission_rule("configure", &permissions.configure)?;
    Ok(())
}

fn validate_permission_rule(
    action: &'static str,
    rule: &PermissionRule,
) -> Result<(), ChannelPolicyError> {
    match rule {
        PermissionRule::Allowlist(values) => {
            if values.is_empty() {
                return Err(ChannelPolicyError::InvalidPermissionRule(format!(
                    "allowlist for {action} must not be empty"
                )));
            }
            for value in values {
                if AgentDid::parse(value).is_err() {
                    return Err(ChannelPolicyError::InvalidPermissionRule(format!(
                        "allowlist for {action} contains invalid did: {value}"
                    )));
                }
            }
            Ok(())
        }
        _ => Ok(()),
    }
}

fn is_authorized(
    rule: &PermissionRule,
    actor: &str,
    members: &BTreeSet<String>,
    admins: &BTreeSet<String>,
) -> bool {
    match rule {
        PermissionRule::All => true,
        PermissionRule::Members => members.contains(actor),
        PermissionRule::Admins => admins.contains(actor),
        PermissionRule::Allowlist(values) => values.contains(actor),
    }
}

#[cfg(test)]
mod tests {
    use super::{
        ChannelAction, ChannelPermissionEngine, ChannelPermissions, ChannelPolicyError,
        PermissionRule, RetentionMessage, RetentionPolicy,
    };
    use std::collections::BTreeSet;

    fn permissions(retention: RetentionPolicy) -> ChannelPermissions {
        ChannelPermissions {
            send: PermissionRule::Members,
            read: PermissionRule::Members,
            invite: PermissionRule::Admins,
            remove: PermissionRule::Admins,
            configure: PermissionRule::Admins,
            retention,
        }
    }

    #[test]
    fn registration_rejects_admin_outside_members() {
        let mut engine = ChannelPermissionEngine::new();
        assert_eq!(
            engine.register_channel(
                "channel:group:1",
                vec!["kamn:did:agent:member-1".to_owned()],
                vec!["kamn:did:agent:admin-1".to_owned()],
                permissions(RetentionPolicy::Forever),
            ),
            Err(ChannelPolicyError::AdminNotMember(
                "kamn:did:agent:admin-1".to_owned()
            ))
        );
    }

    #[test]
    fn max_count_retention_prunes_oldest_first() {
        let mut engine = ChannelPermissionEngine::new();
        engine
            .register_channel(
                "channel:group:2",
                vec!["kamn:did:agent:member-1".to_owned()],
                vec!["kamn:did:agent:member-1".to_owned()],
                permissions(RetentionPolicy::MaxMessageCount(2)),
            )
            .expect("registration should succeed");

        let candidates = engine
            .retention_candidates(
                "channel:group:2",
                500,
                vec![
                    RetentionMessage {
                        id: "msg-c".to_owned(),
                        created_at_secs: 200,
                    },
                    RetentionMessage {
                        id: "msg-a".to_owned(),
                        created_at_secs: 100,
                    },
                    RetentionMessage {
                        id: "msg-b".to_owned(),
                        created_at_secs: 200,
                    },
                ],
            )
            .expect("retention candidates should compute");

        assert_eq!(candidates, vec!["msg-a".to_owned()]);
    }

    #[test]
    fn authorize_members_rule_requires_membership() {
        let mut engine = ChannelPermissionEngine::new();
        engine
            .register_channel(
                "channel:group:3",
                vec!["kamn:did:agent:member-1".to_owned()],
                vec!["kamn:did:agent:member-1".to_owned()],
                permissions(RetentionPolicy::Forever),
            )
            .expect("registration should succeed");

        assert_eq!(
            engine.authorize(
                "channel:group:3",
                "kamn:did:agent:member-2",
                ChannelAction::Send,
            ),
            Err(ChannelPolicyError::Unauthorized {
                actor: "kamn:did:agent:member-2".to_owned(),
                action: ChannelAction::Send,
                rule: PermissionRule::Members,
            })
        );
    }

    #[test]
    fn registration_rejects_empty_allowlist_rule() {
        let mut engine = ChannelPermissionEngine::new();
        let mut config = permissions(RetentionPolicy::Forever);
        config.send = PermissionRule::Allowlist(BTreeSet::new());

        assert_eq!(
            engine.register_channel(
                "channel:group:4",
                vec![
                    "kamn:did:agent:member-1".to_owned(),
                    "kamn:did:agent:member-2".to_owned(),
                ],
                vec!["kamn:did:agent:member-1".to_owned()],
                config,
            ),
            Err(ChannelPolicyError::InvalidPermissionRule(
                "allowlist for send must not be empty".to_owned()
            ))
        );
    }

    #[test]
    fn registration_rejects_allowlist_rule_with_invalid_did_entry() {
        let mut engine = ChannelPermissionEngine::new();
        let mut config = permissions(RetentionPolicy::Forever);
        config.send = PermissionRule::Allowlist(["bad-did".to_owned()].into_iter().collect());

        assert_eq!(
            engine.register_channel(
                "channel:group:5",
                vec![
                    "kamn:did:agent:member-1".to_owned(),
                    "kamn:did:agent:member-2".to_owned(),
                ],
                vec!["kamn:did:agent:member-1".to_owned()],
                config,
            ),
            Err(ChannelPolicyError::InvalidPermissionRule(
                "allowlist for send contains invalid did: bad-did".to_owned()
            ))
        );
    }
}
