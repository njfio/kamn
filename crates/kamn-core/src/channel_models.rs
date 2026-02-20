//! Channel model contracts covering membership, admin policy, and snapshot recovery.

use crate::{AgentDid, SqliteStoreBackend, SqliteStoreBackendError};
use std::collections::{BTreeMap, BTreeSet};
use std::fmt;
use std::fs;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::{Path, PathBuf};

/// Supported channel categories in the KAMN messaging model.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChannelType {
    /// Two-party direct channel.
    Direct,
    /// Multi-party group channel.
    Group,
    /// One-to-many broadcast channel.
    Broadcast,
    /// Task-scoped collaboration channel.
    Task,
    /// Marketplace-scoped negotiation channel.
    Marketplace,
    /// Governance-scoped proposal channel.
    Governance,
}

/// Channel-type-specific metadata payload.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ChannelMetadata {
    /// Direct channel metadata (no additional payload).
    Direct,
    /// Group channel metadata (no additional payload).
    Group,
    /// Broadcast metadata with topic label.
    Broadcast {
        /// Broadcast topic label.
        topic: String,
    },
    /// Task metadata with bound task identifier.
    Task {
        /// Associated task identifier.
        task_id: String,
    },
    /// Marketplace metadata with scope identifier.
    Marketplace {
        /// Marketplace scope identifier.
        market_scope: String,
    },
    /// Governance metadata with proposal scope identifier.
    Governance {
        /// Governance proposal scope identifier.
        proposal_scope: String,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ChannelRecord {
    channel_type: ChannelType,
    metadata: ChannelMetadata,
    members: BTreeSet<String>,
    admins: BTreeSet<String>,
}

/// Schema version for serialized [`ChannelSnapshot`] payloads.
pub const CHANNEL_SNAPSHOT_SCHEMA_VERSION: u16 = 1;

/// Serializable channel record used for snapshot export/import.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChannelRecordSnapshot {
    /// Unique channel identifier.
    pub channel_id: String,
    /// Channel category.
    pub channel_type: ChannelType,
    /// Channel metadata payload.
    pub metadata: ChannelMetadata,
    /// Canonical member DID set.
    pub members: Vec<String>,
    /// Canonical admin DID set.
    pub admins: Vec<String>,
}

/// Serializable snapshot of all channel records.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChannelSnapshot {
    /// Snapshot schema version.
    pub schema_version: u16,
    /// Serialized channel records.
    pub records: Vec<ChannelRecordSnapshot>,
}

/// In-memory channel state store with membership and admin indexes.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ChannelStore {
    channels: BTreeMap<String, ChannelRecord>,
    channels_by_member: BTreeMap<String, BTreeSet<String>>,
}

impl ChannelStore {
    /// Construct an empty channel store.
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a direct channel between exactly two distinct participants.
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

    /// Create a group channel with explicit member/admin sets.
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

    /// Create a broadcast channel with topic metadata.
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

    /// Create a task channel bound to a task identifier.
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

    /// Create a marketplace channel bound to a market scope.
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

    /// Create a governance channel bound to a proposal scope.
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

    /// Look up the channel type for a channel identifier.
    pub fn channel_type(&self, channel_id: &str) -> Result<ChannelType, ChannelModelError> {
        self.channels
            .get(channel_id)
            .map(|record| record.channel_type)
            .ok_or_else(|| ChannelModelError::NotFound(channel_id.to_owned()))
    }

    /// Return all channel members for a channel identifier.
    pub fn members(&self, channel_id: &str) -> Result<Vec<String>, ChannelModelError> {
        let record = self
            .channels
            .get(channel_id)
            .ok_or_else(|| ChannelModelError::NotFound(channel_id.to_owned()))?;
        Ok(record.members.iter().cloned().collect())
    }

    /// Return all channel admins for a channel identifier.
    pub fn admins(&self, channel_id: &str) -> Result<Vec<String>, ChannelModelError> {
        let record = self
            .channels
            .get(channel_id)
            .ok_or_else(|| ChannelModelError::NotFound(channel_id.to_owned()))?;
        Ok(record.admins.iter().cloned().collect())
    }

    /// Return channel IDs where the given DID is currently a member.
    pub fn channels_for_member(&self, member: &str) -> Vec<String> {
        self.channels_by_member
            .get(member)
            .map(|values| values.iter().cloned().collect())
            .unwrap_or_default()
    }

    /// Export all channels into a deterministic snapshot payload.
    pub fn export_snapshot(&self) -> ChannelSnapshot {
        let records = self
            .channels
            .iter()
            .map(|(channel_id, record)| ChannelRecordSnapshot {
                channel_id: channel_id.clone(),
                channel_type: record.channel_type,
                metadata: record.metadata.clone(),
                members: record.members.iter().cloned().collect(),
                admins: record.admins.iter().cloned().collect(),
            })
            .collect();

        ChannelSnapshot {
            schema_version: CHANNEL_SNAPSHOT_SCHEMA_VERSION,
            records,
        }
    }

    /// Restore channel state from a validated snapshot payload.
    pub fn restore_snapshot(
        &mut self,
        snapshot: ChannelSnapshot,
    ) -> Result<(), ChannelSnapshotError> {
        if snapshot.schema_version != CHANNEL_SNAPSHOT_SCHEMA_VERSION {
            return Err(ChannelSnapshotError::SnapshotVersionMismatch {
                expected: CHANNEL_SNAPSHOT_SCHEMA_VERSION,
                found: snapshot.schema_version,
            });
        }

        let mut channels = BTreeMap::new();
        let mut channels_by_member = BTreeMap::new();
        for record_snapshot in snapshot.records {
            if channels.contains_key(&record_snapshot.channel_id) {
                return Err(ChannelSnapshotError::DuplicateChannelId(
                    record_snapshot.channel_id,
                ));
            }

            let record =
                validate_snapshot_record(&record_snapshot).map_err(ChannelSnapshotError::Model)?;

            let channel_id = record_snapshot.channel_id;
            for member in &record.members {
                channels_by_member
                    .entry(member.clone())
                    .or_insert_with(BTreeSet::new)
                    .insert(channel_id.clone());
            }
            channels.insert(channel_id, record);
        }

        self.channels = channels;
        self.channels_by_member = channels_by_member;
        Ok(())
    }

    /// Check whether a DID is currently a member of the given channel.
    pub fn is_member(&self, channel_id: &str, member: &str) -> Result<bool, ChannelModelError> {
        let record = self
            .channels
            .get(channel_id)
            .ok_or_else(|| ChannelModelError::NotFound(channel_id.to_owned()))?;
        Ok(record.members.contains(member))
    }

    /// Return metadata associated with the given channel.
    pub fn metadata(&self, channel_id: &str) -> Result<ChannelMetadata, ChannelModelError> {
        let record = self
            .channels
            .get(channel_id)
            .ok_or_else(|| ChannelModelError::NotFound(channel_id.to_owned()))?;
        Ok(record.metadata.clone())
    }

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

/// Errors emitted by channel creation, membership, and metadata workflows.
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

/// Persistence contract for channel snapshots.
pub trait ChannelSnapshotStore {
    /// Persist a complete snapshot payload.
    fn write(&mut self, snapshot: ChannelSnapshot) -> Result<(), ChannelSnapshotStoreError>;
    /// Load the latest persisted snapshot, if present.
    fn read_latest(&self) -> Result<Option<ChannelSnapshot>, ChannelSnapshotStoreError>;
}

/// In-memory snapshot store for deterministic tests and ephemeral flows.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct InMemoryChannelSnapshotStore {
    latest: Option<ChannelSnapshot>,
}

impl ChannelSnapshotStore for InMemoryChannelSnapshotStore {
    fn write(&mut self, snapshot: ChannelSnapshot) -> Result<(), ChannelSnapshotStoreError> {
        self.latest = Some(snapshot);
        Ok(())
    }

    fn read_latest(&self) -> Result<Option<ChannelSnapshot>, ChannelSnapshotStoreError> {
        Ok(self.latest.clone())
    }
}

/// File-backed snapshot store for durable channel-state persistence.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileChannelSnapshotStore {
    path: PathBuf,
    journal_path: PathBuf,
}

/// Result of file-store recovery attempt.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChannelRecoveryResult {
    /// Latest recovered snapshot, if one was valid.
    pub latest: Option<ChannelSnapshot>,
    /// Whether an invalid payload was repaired via truncation.
    pub repaired: bool,
    /// Deterministic recovery reason code.
    pub reason_code: &'static str,
}

impl ChannelRecoveryResult {
    /// Returns the deterministic recovery reason code.
    pub fn reason_code(&self) -> &'static str {
        self.reason_code
    }
}

impl FileChannelSnapshotStore {
    /// Create a file-backed store for the given snapshot path.
    pub fn new(path: PathBuf) -> Result<Self, ChannelSnapshotStoreError> {
        if path.as_os_str().is_empty() {
            return Err(ChannelSnapshotStoreError::InvalidPayload(
                "snapshot file path cannot be empty".to_owned(),
            ));
        }
        let journal_path = channel_snapshot_journal_path(&path);
        Ok(Self { path, journal_path })
    }

    /// Attempt to read latest snapshot and repair invalid persisted payloads.
    pub fn recover_latest_and_repair(
        &mut self,
    ) -> Result<ChannelRecoveryResult, ChannelSnapshotStoreError> {
        if !self.path.exists() && !self.journal_path.exists() {
            return Ok(ChannelRecoveryResult {
                latest: None,
                repaired: false,
                reason_code: "channel_snapshot_recovery_empty",
            });
        }

        match self.read_latest() {
            Ok(snapshot) => Ok(ChannelRecoveryResult {
                latest: snapshot,
                repaired: false,
                reason_code: "channel_snapshot_recovery_clean",
            }),
            Err(ChannelSnapshotStoreError::InvalidPayload(value))
                if value.starts_with(CHANNEL_SNAPSHOT_JOURNAL_CORRUPT_TAIL_PREFIX) =>
            {
                Err(ChannelSnapshotStoreError::InvalidPayload(value))
            }
            Err(ChannelSnapshotStoreError::InvalidPayload(_))
            | Err(ChannelSnapshotStoreError::Snapshot(_)) => {
                fs::write(&self.path, "")
                    .map_err(|error| ChannelSnapshotStoreError::Io(error.to_string()))?;
                fs::write(&self.journal_path, "")
                    .map_err(|error| ChannelSnapshotStoreError::Io(error.to_string()))?;
                Ok(ChannelRecoveryResult {
                    latest: None,
                    repaired: true,
                    reason_code: "channel_snapshot_recovery_repaired_corrupt_payload",
                })
            }
            Err(error) => Err(error),
        }
    }
}

impl ChannelSnapshotStore for FileChannelSnapshotStore {
    fn write(&mut self, snapshot: ChannelSnapshot) -> Result<(), ChannelSnapshotStoreError> {
        let mut verifier = ChannelStore::new();
        verifier
            .restore_snapshot(snapshot.clone())
            .map_err(ChannelSnapshotStoreError::Snapshot)?;
        let payload = serialize_channel_snapshot(&snapshot)?;
        let mut file = OpenOptions::new()
            .create(true)
            .truncate(true)
            .write(true)
            .open(&self.path)
            .map_err(|error| ChannelSnapshotStoreError::Io(error.to_string()))?;
        file.write_all(payload.as_bytes())
            .map_err(|error| ChannelSnapshotStoreError::Io(error.to_string()))?;
        append_channel_snapshot_journal_record(&self.journal_path, &payload)
    }

    fn read_latest(&self) -> Result<Option<ChannelSnapshot>, ChannelSnapshotStoreError> {
        let snapshot_payload = read_channel_snapshot_file(&self.path)?;
        let journal_snapshot = replay_channel_snapshot_journal(&self.journal_path)?;
        Ok(journal_snapshot.or(snapshot_payload))
    }
}

/// Sqlite-backed snapshot store for durable channel-state persistence.
#[derive(Debug)]
pub struct SqliteChannelSnapshotStore {
    backend: SqliteStoreBackend,
}

impl SqliteChannelSnapshotStore {
    /// Creates a sqlite-backed channel snapshot store rooted at `path`.
    pub fn new(path: PathBuf) -> Result<Self, ChannelSnapshotStoreError> {
        let backend = SqliteStoreBackend::open(path.as_path()).map_err(map_sqlite_store_error)?;
        Ok(Self { backend })
    }
}

impl ChannelSnapshotStore for SqliteChannelSnapshotStore {
    fn write(&mut self, snapshot: ChannelSnapshot) -> Result<(), ChannelSnapshotStoreError> {
        let mut verifier = ChannelStore::new();
        verifier
            .restore_snapshot(snapshot.clone())
            .map_err(ChannelSnapshotStoreError::Snapshot)?;
        let payload = serialize_channel_snapshot(&snapshot)?;
        self.backend
            .put("channel_snapshot_store", "latest", payload.as_bytes())
            .map_err(map_sqlite_store_error)?;
        Ok(())
    }

    fn read_latest(&self) -> Result<Option<ChannelSnapshot>, ChannelSnapshotStoreError> {
        let Some(payload_bytes) = self
            .backend
            .get("channel_snapshot_store", "latest")
            .map_err(map_sqlite_store_error)?
        else {
            return Ok(None);
        };
        let payload = String::from_utf8(payload_bytes).map_err(|_| {
            ChannelSnapshotStoreError::InvalidPayload(
                "channel snapshot sqlite payload is not utf-8".to_owned(),
            )
        })?;
        if payload.trim().is_empty() {
            return Ok(None);
        }
        let snapshot = parse_channel_snapshot_payload(&payload)?;
        let mut verifier = ChannelStore::new();
        verifier
            .restore_snapshot(snapshot.clone())
            .map_err(ChannelSnapshotStoreError::Snapshot)?;
        Ok(Some(snapshot))
    }
}

fn map_sqlite_store_error(error: SqliteStoreBackendError) -> ChannelSnapshotStoreError {
    match error {
        SqliteStoreBackendError::SchemaVersionMissing => ChannelSnapshotStoreError::InvalidPayload(
            "channel snapshot sqlite schema missing".to_owned(),
        ),
        SqliteStoreBackendError::SchemaVersionInvalid(value) => {
            ChannelSnapshotStoreError::InvalidPayload(format!(
                "channel snapshot sqlite schema invalid: {value}"
            ))
        }
        SqliteStoreBackendError::SchemaVersionMismatch { expected, found } => {
            ChannelSnapshotStoreError::InvalidPayload(format!(
                "channel snapshot sqlite schema mismatch: expected {expected}, found {found}"
            ))
        }
        SqliteStoreBackendError::InvalidPath => ChannelSnapshotStoreError::InvalidPayload(
            "snapshot file path cannot be empty".to_owned(),
        ),
        other => ChannelSnapshotStoreError::Io(other.to_string()),
    }
}

const CHANNEL_SNAPSHOT_JOURNAL_CORRUPT_TAIL_PREFIX: &str = "channel_snapshot_journal_corrupt_tail";

fn channel_snapshot_journal_path(path: &Path) -> PathBuf {
    let mut journal = path.as_os_str().to_os_string();
    journal.push(".journal");
    PathBuf::from(journal)
}

fn read_channel_snapshot_file(
    path: &Path,
) -> Result<Option<ChannelSnapshot>, ChannelSnapshotStoreError> {
    if !path.exists() {
        return Ok(None);
    }

    let payload = fs::read_to_string(path)
        .map_err(|error| ChannelSnapshotStoreError::Io(error.to_string()))?;
    if payload.trim().is_empty() {
        return Ok(None);
    }
    let snapshot = parse_channel_snapshot_payload(&payload)?;
    let mut verifier = ChannelStore::new();
    verifier
        .restore_snapshot(snapshot.clone())
        .map_err(ChannelSnapshotStoreError::Snapshot)?;
    Ok(Some(snapshot))
}

fn append_channel_snapshot_journal_record(
    journal_path: &Path,
    payload: &str,
) -> Result<(), ChannelSnapshotStoreError> {
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(journal_path)
        .map_err(|error| ChannelSnapshotStoreError::Io(error.to_string()))?;
    let record = format!("entry|1|{}\n", encode_journal_hex(payload.as_bytes()));
    file.write_all(record.as_bytes())
        .map_err(|error| ChannelSnapshotStoreError::Io(error.to_string()))
}

fn replay_channel_snapshot_journal(
    journal_path: &Path,
) -> Result<Option<ChannelSnapshot>, ChannelSnapshotStoreError> {
    if !journal_path.exists() {
        return Ok(None);
    }

    let payload = fs::read_to_string(journal_path)
        .map_err(|error| ChannelSnapshotStoreError::Io(error.to_string()))?;
    let mut latest = None;

    for (index, line) in payload.lines().enumerate() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        let payload_hex = parse_channel_snapshot_journal_record(trimmed, index + 1)?;
        let payload_bytes = decode_journal_hex(payload_hex)
            .ok_or_else(|| channel_snapshot_journal_corrupt_tail(index + 1))?;
        let payload = String::from_utf8(payload_bytes)
            .map_err(|_| channel_snapshot_journal_corrupt_tail(index + 1))?;
        let snapshot = parse_channel_snapshot_payload(&payload)
            .map_err(|_| channel_snapshot_journal_corrupt_tail(index + 1))?;
        let mut verifier = ChannelStore::new();
        verifier
            .restore_snapshot(snapshot.clone())
            .map_err(|_| channel_snapshot_journal_corrupt_tail(index + 1))?;
        latest = Some(snapshot);
    }

    Ok(latest)
}

fn parse_channel_snapshot_journal_record(
    line: &str,
    index: usize,
) -> Result<&str, ChannelSnapshotStoreError> {
    let mut parts = line.split('|');
    let Some(prefix) = parts.next() else {
        return Err(channel_snapshot_journal_corrupt_tail(index));
    };
    let Some(version) = parts.next() else {
        return Err(channel_snapshot_journal_corrupt_tail(index));
    };
    let Some(payload_hex) = parts.next() else {
        return Err(channel_snapshot_journal_corrupt_tail(index));
    };
    if prefix != "entry" || version != "1" || payload_hex.is_empty() || parts.next().is_some() {
        return Err(channel_snapshot_journal_corrupt_tail(index));
    }
    Ok(payload_hex)
}

fn channel_snapshot_journal_corrupt_tail(index: usize) -> ChannelSnapshotStoreError {
    ChannelSnapshotStoreError::InvalidPayload(format!(
        "{CHANNEL_SNAPSHOT_JOURNAL_CORRUPT_TAIL_PREFIX}:{index}"
    ))
}

fn encode_journal_hex(bytes: &[u8]) -> String {
    let mut encoded = String::with_capacity(bytes.len() * 2);
    const HEX: &[u8; 16] = b"0123456789abcdef";
    for byte in bytes {
        encoded.push(HEX[(byte >> 4) as usize] as char);
        encoded.push(HEX[(byte & 0x0f) as usize] as char);
    }
    encoded
}

fn decode_journal_hex(value: &str) -> Option<Vec<u8>> {
    if !value.len().is_multiple_of(2) {
        return None;
    }
    let bytes = value.as_bytes();
    let mut decoded = Vec::with_capacity(bytes.len() / 2);
    let mut index = 0usize;
    while index < bytes.len() {
        let high = decode_journal_nibble(bytes[index])?;
        let low = decode_journal_nibble(bytes[index + 1])?;
        decoded.push((high << 4) | low);
        index += 2;
    }
    Some(decoded)
}

fn decode_journal_nibble(value: u8) -> Option<u8> {
    match value {
        b'0'..=b'9' => Some(value - b'0'),
        b'a'..=b'f' => Some(value - b'a' + 10),
        b'A'..=b'F' => Some(value - b'A' + 10),
        _ => None,
    }
}

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

fn metadata_matches_channel_type(channel_type: ChannelType, metadata: &ChannelMetadata) -> bool {
    matches!(
        (channel_type, metadata),
        (ChannelType::Direct, ChannelMetadata::Direct)
            | (ChannelType::Group, ChannelMetadata::Group)
            | (ChannelType::Broadcast, ChannelMetadata::Broadcast { .. })
            | (ChannelType::Task, ChannelMetadata::Task { .. })
            | (
                ChannelType::Marketplace,
                ChannelMetadata::Marketplace { .. }
            )
            | (ChannelType::Governance, ChannelMetadata::Governance { .. })
    )
}

fn validate_snapshot_record(
    record: &ChannelRecordSnapshot,
) -> Result<ChannelRecord, ChannelModelError> {
    validate_channel_id(&record.channel_id)?;
    if !metadata_matches_channel_type(record.channel_type, &record.metadata) {
        return Err(ChannelModelError::InvalidMetadata(
            "channel type and metadata variant mismatch".to_owned(),
        ));
    }
    validate_metadata(&record.metadata)?;
    if record.members.is_empty() {
        return Err(ChannelModelError::EmptyMembers);
    }
    if record.admins.is_empty() {
        return Err(ChannelModelError::EmptyAdmins);
    }

    let mut members = BTreeSet::new();
    for member in &record.members {
        validate_did(member)?;
        if !members.insert(member.clone()) {
            return Err(ChannelModelError::InvalidMetadata(
                "duplicate member DID in snapshot".to_owned(),
            ));
        }
    }

    let mut admins = BTreeSet::new();
    for admin in &record.admins {
        validate_did(admin)?;
        if !members.contains(admin) {
            return Err(ChannelModelError::AdminNotMember(admin.clone()));
        }
        if !admins.insert(admin.clone()) {
            return Err(ChannelModelError::InvalidMetadata(
                "duplicate admin DID in snapshot".to_owned(),
            ));
        }
    }

    if record.channel_type == ChannelType::Direct {
        if members.len() != 2 || admins != members {
            return Err(ChannelModelError::InvalidDirectParticipants);
        }
    } else {
        enforce_specialized_member_requirements(record.channel_type, members.len())?;
    }

    Ok(ChannelRecord {
        channel_type: record.channel_type,
        metadata: record.metadata.clone(),
        members,
        admins,
    })
}

fn channel_type_code(channel_type: ChannelType) -> &'static str {
    match channel_type {
        ChannelType::Direct => "0",
        ChannelType::Group => "1",
        ChannelType::Broadcast => "2",
        ChannelType::Task => "3",
        ChannelType::Marketplace => "4",
        ChannelType::Governance => "5",
    }
}

fn parse_channel_type_code(raw: &str) -> Option<ChannelType> {
    match raw {
        "0" => Some(ChannelType::Direct),
        "1" => Some(ChannelType::Group),
        "2" => Some(ChannelType::Broadcast),
        "3" => Some(ChannelType::Task),
        "4" => Some(ChannelType::Marketplace),
        "5" => Some(ChannelType::Governance),
        _ => None,
    }
}

fn metadata_snapshot_value(metadata: &ChannelMetadata) -> &str {
    match metadata {
        ChannelMetadata::Direct | ChannelMetadata::Group => "",
        ChannelMetadata::Broadcast { topic } => topic,
        ChannelMetadata::Task { task_id } => task_id,
        ChannelMetadata::Marketplace { market_scope } => market_scope,
        ChannelMetadata::Governance { proposal_scope } => proposal_scope,
    }
}

fn parse_metadata_snapshot_value(
    channel_type: ChannelType,
    value: &str,
) -> Result<ChannelMetadata, ChannelSnapshotStoreError> {
    match channel_type {
        ChannelType::Direct => {
            if !value.is_empty() {
                return Err(ChannelSnapshotStoreError::InvalidPayload(
                    "direct channel metadata payload must be empty".to_owned(),
                ));
            }
            Ok(ChannelMetadata::Direct)
        }
        ChannelType::Group => {
            if !value.is_empty() {
                return Err(ChannelSnapshotStoreError::InvalidPayload(
                    "group channel metadata payload must be empty".to_owned(),
                ));
            }
            Ok(ChannelMetadata::Group)
        }
        ChannelType::Broadcast => Ok(ChannelMetadata::Broadcast {
            topic: value.to_owned(),
        }),
        ChannelType::Task => Ok(ChannelMetadata::Task {
            task_id: value.to_owned(),
        }),
        ChannelType::Marketplace => Ok(ChannelMetadata::Marketplace {
            market_scope: value.to_owned(),
        }),
        ChannelType::Governance => Ok(ChannelMetadata::Governance {
            proposal_scope: value.to_owned(),
        }),
    }
}

fn ensure_snapshot_token(value: &str, field: &str) -> Result<(), ChannelSnapshotStoreError> {
    if value.contains('|') || value.contains('\n') || value.contains('\r') || value.contains(',') {
        return Err(ChannelSnapshotStoreError::InvalidPayload(format!(
            "{field} contains unsupported delimiter characters"
        )));
    }
    Ok(())
}

fn serialize_channel_snapshot(
    snapshot: &ChannelSnapshot,
) -> Result<String, ChannelSnapshotStoreError> {
    let mut payload = format!("schema|{}\n", snapshot.schema_version);
    for record in &snapshot.records {
        ensure_snapshot_token(&record.channel_id, "channel_id")?;
        let metadata_value = metadata_snapshot_value(&record.metadata);
        ensure_snapshot_token(metadata_value, "metadata")?;
        for member in &record.members {
            ensure_snapshot_token(member, "member")?;
        }
        for admin in &record.admins {
            ensure_snapshot_token(admin, "admin")?;
        }
        payload.push_str(&format!(
            "record|{}|{}|{}|{}|{}\n",
            record.channel_id,
            channel_type_code(record.channel_type),
            metadata_value,
            record.members.join(","),
            record.admins.join(",")
        ));
    }
    Ok(payload)
}

fn parse_channel_snapshot_payload(
    payload: &str,
) -> Result<ChannelSnapshot, ChannelSnapshotStoreError> {
    let mut lines = payload.lines().filter(|line| !line.trim().is_empty());
    let Some(schema_line) = lines.next() else {
        return Err(ChannelSnapshotStoreError::InvalidPayload(
            "missing schema line".to_owned(),
        ));
    };

    let mut schema_parts = schema_line.split('|');
    let Some(schema_prefix) = schema_parts.next() else {
        return Err(ChannelSnapshotStoreError::InvalidPayload(
            schema_line.to_owned(),
        ));
    };
    let Some(schema_version_raw) = schema_parts.next() else {
        return Err(ChannelSnapshotStoreError::InvalidPayload(
            schema_line.to_owned(),
        ));
    };
    if schema_prefix != "schema" || schema_parts.next().is_some() {
        return Err(ChannelSnapshotStoreError::InvalidPayload(
            schema_line.to_owned(),
        ));
    }
    let schema_version = schema_version_raw
        .parse::<u16>()
        .map_err(|_| ChannelSnapshotStoreError::InvalidPayload(schema_line.to_owned()))?;

    let mut records = Vec::new();
    for line in lines {
        let mut parts = line.split('|');
        let Some(prefix) = parts.next() else {
            return Err(ChannelSnapshotStoreError::InvalidPayload(line.to_owned()));
        };
        if prefix != "record" {
            return Err(ChannelSnapshotStoreError::InvalidPayload(line.to_owned()));
        }
        let Some(channel_id) = parts.next() else {
            return Err(ChannelSnapshotStoreError::InvalidPayload(line.to_owned()));
        };
        let Some(type_code) = parts.next() else {
            return Err(ChannelSnapshotStoreError::InvalidPayload(line.to_owned()));
        };
        let Some(metadata_raw) = parts.next() else {
            return Err(ChannelSnapshotStoreError::InvalidPayload(line.to_owned()));
        };
        let Some(members_raw) = parts.next() else {
            return Err(ChannelSnapshotStoreError::InvalidPayload(line.to_owned()));
        };
        let Some(admins_raw) = parts.next() else {
            return Err(ChannelSnapshotStoreError::InvalidPayload(line.to_owned()));
        };
        if parts.next().is_some() {
            return Err(ChannelSnapshotStoreError::InvalidPayload(line.to_owned()));
        }

        let channel_type = parse_channel_type_code(type_code)
            .ok_or_else(|| ChannelSnapshotStoreError::InvalidPayload(line.to_owned()))?;
        let metadata = parse_metadata_snapshot_value(channel_type, metadata_raw)?;
        let members = if members_raw.is_empty() {
            Vec::new()
        } else {
            members_raw
                .split(',')
                .map(|value| value.to_owned())
                .collect::<Vec<_>>()
        };
        let admins = if admins_raw.is_empty() {
            Vec::new()
        } else {
            admins_raw
                .split(',')
                .map(|value| value.to_owned())
                .collect::<Vec<_>>()
        };

        records.push(ChannelRecordSnapshot {
            channel_id: channel_id.to_owned(),
            channel_type,
            metadata,
            members,
            admins,
        });
    }

    Ok(ChannelSnapshot {
        schema_version,
        records,
    })
}

#[cfg(test)]
mod tests {
    use super::{
        serialize_channel_snapshot, ChannelMetadata, ChannelModelError, ChannelRecordSnapshot,
        ChannelSnapshot, ChannelSnapshotError, ChannelSnapshotStore, ChannelSnapshotStoreError,
        ChannelStore, ChannelType, FileChannelSnapshotStore,
    };
    use std::env;
    use std::fs;
    use std::fs::OpenOptions;
    use std::io::Write;
    use std::path::PathBuf;
    use std::time::{Instant, SystemTime, UNIX_EPOCH};

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

    #[test]
    fn functional_channel_snapshot_roundtrip_restores_member_index() {
        let mut store = ChannelStore::new();
        store
            .create_group(
                "channel:group:snapshot-1",
                "kamn:did:agent:owner",
                vec![
                    "kamn:did:agent:owner".to_owned(),
                    "kamn:did:agent:member-1".to_owned(),
                ],
                vec!["kamn:did:agent:owner".to_owned()],
            )
            .expect("group should be created");
        store
            .invite_member(
                "channel:group:snapshot-1",
                "kamn:did:agent:owner",
                "kamn:did:agent:member-2",
            )
            .expect("invite should succeed");

        let snapshot = store.export_snapshot();
        let mut restored = ChannelStore::new();
        restored
            .restore_snapshot(snapshot)
            .expect("snapshot restore should succeed");

        assert_eq!(
            restored
                .members("channel:group:snapshot-1")
                .expect("members should exist"),
            vec![
                "kamn:did:agent:member-1".to_owned(),
                "kamn:did:agent:member-2".to_owned(),
                "kamn:did:agent:owner".to_owned(),
            ]
        );
        assert_eq!(
            restored.channels_for_member("kamn:did:agent:member-2"),
            vec!["channel:group:snapshot-1".to_owned()]
        );
    }

    #[test]
    fn regression_channel_snapshot_restore_rejects_duplicate_channel_ids() {
        // Regression: #617
        let mut store = ChannelStore::new();
        store
            .create_direct(
                "channel:direct:snapshot-2",
                "kamn:did:agent:alice",
                "kamn:did:agent:bob",
            )
            .expect("direct should be created");

        let mut snapshot = store.export_snapshot();
        snapshot.records.push(snapshot.records[0].clone());

        let mut restored = ChannelStore::new();
        assert_eq!(
            restored.restore_snapshot(snapshot),
            Err(ChannelSnapshotError::DuplicateChannelId(
                "channel:direct:snapshot-2".to_owned()
            ))
        );
    }

    #[test]
    fn regression_channel_snapshot_restore_rejects_admin_not_member_state() {
        // Regression: #617
        let snapshot = ChannelSnapshot {
            schema_version: 1,
            records: vec![ChannelRecordSnapshot {
                channel_id: "channel:group:snapshot-3".to_owned(),
                channel_type: ChannelType::Group,
                metadata: ChannelMetadata::Group,
                members: vec![
                    "kamn:did:agent:owner".to_owned(),
                    "kamn:did:agent:member-1".to_owned(),
                ],
                admins: vec![
                    "kamn:did:agent:owner".to_owned(),
                    "kamn:did:agent:ghost-admin".to_owned(),
                ],
            }],
        };

        let mut restored = ChannelStore::new();
        assert_eq!(
            restored.restore_snapshot(snapshot),
            Err(ChannelSnapshotError::Model(
                ChannelModelError::AdminNotMember("kamn:did:agent:ghost-admin".to_owned())
            ))
        );
    }

    #[test]
    fn integration_file_channel_snapshot_store_roundtrips_snapshot() {
        let path = temp_channel_snapshot_path("roundtrip");
        let journal_path = temp_channel_snapshot_journal_path(&path);
        let _ = fs::remove_file(&path);
        let _ = fs::remove_file(&journal_path);

        let mut store = ChannelStore::new();
        store
            .create_group(
                "channel:group:snapshot-4",
                "kamn:did:agent:owner",
                vec![
                    "kamn:did:agent:owner".to_owned(),
                    "kamn:did:agent:member-1".to_owned(),
                ],
                vec!["kamn:did:agent:owner".to_owned()],
            )
            .expect("group should be created");
        let snapshot = store.export_snapshot();

        let mut file_store = FileChannelSnapshotStore::new(path.clone()).expect("store");
        file_store
            .write(snapshot.clone())
            .expect("write should succeed");
        assert_eq!(
            file_store.read_latest().expect("read should succeed"),
            Some(snapshot)
        );

        let _ = fs::remove_file(path);
        let _ = fs::remove_file(journal_path);
    }

    #[test]
    fn integration_file_channel_snapshot_store_replays_journal_when_snapshot_is_stale() {
        let path = temp_channel_snapshot_path("journal-replay");
        let journal_path = temp_channel_snapshot_journal_path(&path);
        let _ = fs::remove_file(&path);
        let _ = fs::remove_file(&journal_path);

        let mut store = ChannelStore::new();
        store
            .create_group(
                "channel:group:journal-1",
                "kamn:did:agent:owner",
                vec![
                    "kamn:did:agent:owner".to_owned(),
                    "kamn:did:agent:member-1".to_owned(),
                ],
                vec!["kamn:did:agent:owner".to_owned()],
            )
            .expect("group should be created");
        let first_snapshot = store.export_snapshot();

        let mut file_store = FileChannelSnapshotStore::new(path.clone()).expect("store");
        file_store
            .write(first_snapshot.clone())
            .expect("write should succeed");

        store
            .invite_member(
                "channel:group:journal-1",
                "kamn:did:agent:owner",
                "kamn:did:agent:member-2",
            )
            .expect("invite should succeed");
        let second_snapshot = store.export_snapshot();
        file_store
            .write(second_snapshot.clone())
            .expect("second write should succeed");

        let stale_payload =
            serialize_channel_snapshot(&first_snapshot).expect("first snapshot should serialize");
        assert!(fs::write(&path, stale_payload).is_ok());
        assert_eq!(
            file_store.read_latest().expect("journal replay should win"),
            Some(second_snapshot)
        );

        let _ = fs::remove_file(path);
        let _ = fs::remove_file(journal_path);
    }

    #[test]
    fn regression_file_channel_snapshot_store_rejects_malformed_payload() {
        // Regression: #617
        let path = temp_channel_snapshot_path("malformed");
        let _ = fs::remove_file(&path);
        assert!(fs::write(&path, "schema|1\nrecord|broken\n").is_ok());

        let file_store = FileChannelSnapshotStore::new(path.clone()).expect("store");
        assert_eq!(
            file_store.read_latest(),
            Err(ChannelSnapshotStoreError::InvalidPayload(
                "record|broken".to_owned()
            ))
        );

        let _ = fs::remove_file(path);
    }

    #[test]
    fn functional_file_channel_snapshot_store_recovery_repairs_corrupt_payload() {
        let path = temp_channel_snapshot_path("recover");
        let journal_path = temp_channel_snapshot_journal_path(&path);
        let _ = fs::remove_file(&path);
        let _ = fs::remove_file(&journal_path);
        assert!(fs::write(&path, "schema|1\nrecord|broken\n").is_ok());

        let mut file_store = FileChannelSnapshotStore::new(path.clone()).expect("store");
        let recovery = file_store
            .recover_latest_and_repair()
            .expect("recovery should succeed");
        assert!(recovery.latest.is_none());
        assert!(recovery.repaired);
        assert_eq!(
            fs::read_to_string(&path).expect("repaired file should be readable"),
            ""
        );

        let _ = fs::remove_file(path);
        let _ = fs::remove_file(journal_path);
    }

    #[test]
    fn regression_file_channel_snapshot_store_rejects_corrupt_journal_tail() {
        // Regression: #2690
        let path = temp_channel_snapshot_path("corrupt-journal-tail");
        let journal_path = temp_channel_snapshot_journal_path(&path);
        let _ = fs::remove_file(&path);
        let _ = fs::remove_file(&journal_path);

        let mut store = ChannelStore::new();
        store
            .create_group(
                "channel:group:journal-tail",
                "kamn:did:agent:owner",
                vec![
                    "kamn:did:agent:owner".to_owned(),
                    "kamn:did:agent:member-1".to_owned(),
                ],
                vec!["kamn:did:agent:owner".to_owned()],
            )
            .expect("group should be created");
        let snapshot = store.export_snapshot();
        let mut file_store = FileChannelSnapshotStore::new(path.clone()).expect("store");
        file_store.write(snapshot).expect("write should succeed");

        let mut journal = OpenOptions::new()
            .append(true)
            .open(&journal_path)
            .expect("journal should exist");
        assert!(journal.write_all(b"entry|1|deadbeefz\n").is_ok());
        assert_eq!(
            file_store.recover_latest_and_repair(),
            Err(ChannelSnapshotStoreError::InvalidPayload(
                "channel_snapshot_journal_corrupt_tail:2".to_owned()
            ))
        );

        let _ = fs::remove_file(path);
        let _ = fs::remove_file(journal_path);
    }

    #[test]
    fn performance_channel_snapshot_roundtrip_stays_within_ci_budget() {
        let mut store = ChannelStore::new();
        for index in 0..256 {
            store
                .create_group(
                    &format!("channel:group:perf-{index}"),
                    "kamn:did:agent:owner",
                    vec![
                        "kamn:did:agent:owner".to_owned(),
                        format!("kamn:did:agent:member-{index}"),
                    ],
                    vec!["kamn:did:agent:owner".to_owned()],
                )
                .expect("group should be created");
        }

        let snapshot = store.export_snapshot();
        let mut restored = ChannelStore::new();
        let start = Instant::now();
        restored
            .restore_snapshot(snapshot)
            .expect("snapshot restore should succeed");
        let elapsed_millis = start.elapsed().as_millis();
        assert!(
            elapsed_millis < 300,
            "channel snapshot roundtrip exceeded CI budget: {elapsed_millis}ms"
        );
    }

    #[test]
    fn performance_channel_snapshot_deep_lane_stress() {
        if env::var("KAMN_KOLME_LOCAL_HEAVY").ok().as_deref() != Some("1") {
            eprintln!(
                "skipping deep-lane channel snapshot stress test; set KAMN_KOLME_LOCAL_HEAVY=1 to run"
            );
            return;
        }

        let mut store = ChannelStore::new();
        for index in 0..6000 {
            store
                .create_group(
                    &format!("channel:group:deep-{index}"),
                    "kamn:did:agent:owner",
                    vec![
                        "kamn:did:agent:owner".to_owned(),
                        format!("kamn:did:agent:member-{index}"),
                    ],
                    vec!["kamn:did:agent:owner".to_owned()],
                )
                .expect("group should be created");
        }

        let snapshot = store.export_snapshot();
        let mut restored = ChannelStore::new();
        restored
            .restore_snapshot(snapshot)
            .expect("snapshot restore should succeed");
    }

    fn temp_channel_snapshot_path(tag: &str) -> PathBuf {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock should be monotonic")
            .as_nanos();
        std::env::temp_dir().join(format!("kamn-channel-snapshot-{tag}-{nonce}.log"))
    }

    fn temp_channel_snapshot_journal_path(path: &std::path::Path) -> PathBuf {
        let mut journal = path.as_os_str().to_os_string();
        journal.push(".journal");
        PathBuf::from(journal)
    }
}
