#![allow(missing_docs)]

use super::store::ChannelType;
use std::fmt;

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
            Self::InvalidDirectParticipants => write_direct_participant_error(f),
            Self::EmptyMembers => write!(f, "group channel members must not be empty"),
            Self::EmptyAdmins => write!(f, "group channel admins must not be empty"),
            Self::InvalidMetadata(value) => write!(f, "invalid channel metadata: {value}"),
            Self::InsufficientMembers { .. } => write_member_count_error(self, f),
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
            Self::UnsupportedOperation { .. } => write_unsupported_operation(self, f),
        }
    }
}

impl std::error::Error for ChannelModelError {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ChannelSnapshotError {
    SnapshotVersionMismatch { expected: u16, found: u16 },
    DuplicateChannelId(String),
    InvalidSnapshot(String),
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ChannelSnapshotStoreError {
    Io(String),
    InvalidPayload(String),
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

fn write_direct_participant_error(f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "direct channels require two distinct participants")
}

fn write_member_count_error(error: &ChannelModelError, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match error {
        ChannelModelError::InsufficientMembers {
            channel_type,
            minimum,
            actual,
        } => write!(
            f,
            "channel type {channel_type:?} requires at least {minimum} members, found {actual}"
        ),
        _ => unreachable!("member-count formatter only handles insufficent-member errors"),
    }
}

fn write_unsupported_operation(
    error: &ChannelModelError,
    f: &mut fmt::Formatter<'_>,
) -> fmt::Result {
    match error {
        ChannelModelError::UnsupportedOperation {
            channel_type,
            action,
        } => write!(
            f,
            "unsupported operation {action} for channel type {channel_type:?}"
        ),
        _ => unreachable!("unsupported-operation formatter only handles unsupported operations"),
    }
}
