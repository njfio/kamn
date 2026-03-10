use super::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ChannelModelError {
    /// Channel identifier was empty.
    EmptyChannelId,
    /// DID failed validation.
    InvalidDid(String),
    /// Channel identifier already exists.
    DuplicateChannelId(String),
    /// Direct channels require two distinct participants/admins.
    InvalidDirectParticipants,
    /// Member list is empty.
    EmptyMembers,
    /// Admin list is empty.
    EmptyAdmins,
    /// Metadata payload is invalid for the channel type.
    InvalidMetadata(String),
    /// Channel type requires more members than provided.
    InsufficientMembers {
        /// Channel type being validated.
        channel_type: ChannelType,
        /// Required minimum member count.
        minimum: usize,
        /// Actual member count provided.
        actual: usize,
    },
    /// Declared creator is not present in members.
    CreatorNotMember(String),
    /// Declared admin is not present in members.
    AdminNotMember(String),
    /// Actor lacks required role for the attempted action.
    UnauthorizedActor {
        /// Actor DID that attempted the action.
        actor: String,
        /// Required role label for authorization.
        required: &'static str,
    },
    /// Channel identifier does not exist.
    NotFound(String),
    /// Member already exists in channel membership set.
    MemberAlreadyPresent(String),
    /// Member does not exist in channel membership set.
    MemberNotFound(String),
    /// Admin does not exist in channel admin set.
    AdminNotFound(String),
    /// Action would remove the final remaining admin.
    LastAdminRemoval(String),
    /// Action is unsupported for the given channel type.
    UnsupportedOperation {
        /// Channel type rejecting the action.
        channel_type: ChannelType,
        /// Action label rejected by policy.
        action: &'static str,
    },
}

impl fmt::Display for ChannelModelError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnauthorizedActor { actor, required } => {
                write_unauthorized_actor(f, actor, required)
            }
            Self::UnsupportedOperation {
                channel_type,
                action,
            } => write_unsupported_operation(f, *channel_type, action),
            _ => write!(f, "{}", self.static_message()),
        }
    }
}

impl std::error::Error for ChannelModelError {}

impl ChannelModelError {
    fn static_message(&self) -> String {
        match self {
            Self::EmptyChannelId => simple_channel_message("channel_id must not be empty"),
            Self::InvalidDid(value) => tagged_channel_message("invalid channel DID", value),
            Self::DuplicateChannelId(value) => {
                tagged_channel_message("duplicate channel id", value)
            }
            Self::InvalidDirectParticipants => {
                simple_channel_message("direct channels require two distinct participants")
            }
            Self::EmptyMembers => simple_channel_message("group channel members must not be empty"),
            Self::EmptyAdmins => simple_channel_message("group channel admins must not be empty"),
            Self::InvalidMetadata(value) => {
                tagged_channel_message("invalid channel metadata", value)
            }
            Self::InsufficientMembers {
                channel_type,
                minimum,
                actual,
            } => insufficient_members_message(*channel_type, *minimum, *actual),
            Self::CreatorNotMember(value) => {
                tagged_channel_message("creator must be a member", value)
            }
            Self::AdminNotMember(value) => tagged_channel_message("admin must be a member", value),
            Self::NotFound(value) => tagged_channel_message("channel not found", value),
            Self::MemberAlreadyPresent(value) => {
                tagged_channel_message("member already present", value)
            }
            Self::MemberNotFound(value) => tagged_channel_message("member not found", value),
            Self::AdminNotFound(value) => tagged_channel_message("admin not found", value),
            Self::LastAdminRemoval(value) => last_admin_removal_message(value),
            Self::UnauthorizedActor { .. } | Self::UnsupportedOperation { .. } => unreachable!(),
        }
    }
}

fn simple_channel_message(message: &str) -> String {
    message.to_owned()
}

fn tagged_channel_message(label: &str, value: &str) -> String {
    format!("{label}: {value}")
}

fn insufficient_members_message(
    channel_type: ChannelType,
    minimum: usize,
    actual: usize,
) -> String {
    format!("channel type {channel_type:?} requires at least {minimum} members, found {actual}")
}

fn last_admin_removal_message(value: &str) -> String {
    format!("cannot remove last admin from {value}")
}

fn write_unauthorized_actor(
    f: &mut fmt::Formatter<'_>,
    actor: &str,
    required: &str,
) -> fmt::Result {
    write!(f, "unauthorized actor {actor}, requires {required}")
}

fn write_unsupported_operation(
    f: &mut fmt::Formatter<'_>,
    channel_type: ChannelType,
    action: &str,
) -> fmt::Result {
    write!(
        f,
        "unsupported operation {action} for channel type {channel_type:?}"
    )
}
