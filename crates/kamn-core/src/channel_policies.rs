use crate::AgentDid;
use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChannelAction {
    Send,
    Read,
    Invite,
    Remove,
    Configure,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PermissionRule {
    All,
    Members,
    Admins,
    Allowlist(BTreeSet<String>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RetentionPolicy {
    Forever,
    MaxAgeSeconds(u64),
    MaxMessageCount(usize),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChannelPermissions {
    pub send: PermissionRule,
    pub read: PermissionRule,
    pub invite: PermissionRule,
    pub remove: PermissionRule,
    pub configure: PermissionRule,
    pub retention: RetentionPolicy,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RetentionMessage {
    pub id: String,
    pub created_at_secs: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ChannelPolicyRecord {
    members: BTreeSet<String>,
    admins: BTreeSet<String>,
    permissions: ChannelPermissions,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ChannelPermissionEngine {
    channels: BTreeMap<String, ChannelPolicyRecord>,
}

impl ChannelPermissionEngine {
    pub fn new() -> Self {
        Self::default()
    }

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
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ChannelPolicyError {
    EmptyChannelId,
    DuplicateChannelId(String),
    EmptyMembers,
    EmptyAdmins,
    InvalidDid(String),
    AdminNotMember(String),
    NotFound(String),
    Unauthorized {
        actor: String,
        action: ChannelAction,
        rule: PermissionRule,
    },
    InvalidPermissionRule(String),
    InvalidRetentionPolicy(String),
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
