use super::channel_types::ChannelType;
use std::fmt;

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

/// Errors emitted while validating/restoring snapshots.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ChannelSnapshotError {
    /// Snapshot schema version mismatched runtime expectation.
    SnapshotVersionMismatch {
        /// Expected schema version.
        expected: u16,
        /// Schema version found in snapshot payload.
        found: u16,
    },
    /// Duplicate channel identifier was found in snapshot records.
    DuplicateChannelId(String),
    /// Snapshot payload was malformed or semantically invalid.
    InvalidSnapshot(String),
    /// Snapshot record failed normal channel-model validation.
    Model(ChannelModelError),
}

impl fmt::Display for ChannelSnapshotError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::SnapshotVersionMismatch { expected, found } => {
                write!(
                    f,
                    "channel snapshot version mismatch: expected {expected}, found {found}"
                )
            }
            Self::DuplicateChannelId(value) => {
                write!(f, "duplicate channel id in snapshot: {value}")
            }
            Self::InvalidSnapshot(value) => write!(f, "invalid channel snapshot: {value}"),
            Self::Model(error) => write!(f, "{error}"),
        }
    }
}

impl std::error::Error for ChannelSnapshotError {}

/// Errors emitted by snapshot-store read/write and recovery operations.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ChannelSnapshotStoreError {
    /// Filesystem I/O operation failed.
    Io(String),
    /// Snapshot payload encoding/format was invalid.
    InvalidPayload(String),
    /// Snapshot payload failed semantic validation.
    Snapshot(ChannelSnapshotError),
}

impl fmt::Display for ChannelSnapshotStoreError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(value) => write!(f, "channel snapshot store I/O error: {value}"),
            Self::InvalidPayload(value) => {
                write!(f, "channel snapshot store invalid payload: {value}")
            }
            Self::Snapshot(error) => write!(f, "{error}"),
        }
    }
}

impl std::error::Error for ChannelSnapshotStoreError {}
