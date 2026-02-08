use crate::{
    AgentDid, ProcessorProofAdmissionEvaluator, ProcessorProofAdmissionInput,
    ProcessorProofArtifact, ZkDesignError,
};
use std::collections::{BTreeMap, BTreeSet};
use std::fmt;
use std::fs;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::PathBuf;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum MessageStatus {
    Created,
    Signed,
    Broadcast,
    Included,
    Delivered,
    Validated,
    Rejected,
    Expired,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct MessageRecord {
    sender: String,
    recipients: Vec<String>,
    created: String,
    expires: String,
    status: MessageStatus,
    history: Vec<MessageStatus>,
}

pub const MESSAGE_LIFECYCLE_SNAPSHOT_SCHEMA_VERSION: u16 = 1;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MessageRecordSnapshot {
    pub message_id: String,
    pub sender: String,
    pub recipients: Vec<String>,
    pub created: String,
    pub expires: String,
    pub status: MessageStatus,
    pub history: Vec<MessageStatus>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MessageLifecycleSnapshot {
    pub schema_version: u16,
    pub records: Vec<MessageRecordSnapshot>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct MessageLifecycleStore {
    records: BTreeMap<String, MessageRecord>,
    ids_by_status: BTreeMap<MessageStatus, BTreeSet<String>>,
    ids_by_sender: BTreeMap<String, BTreeSet<String>>,
    ids_by_recipient: BTreeMap<String, BTreeSet<String>>,
}

impl MessageLifecycleStore {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register(
        &mut self,
        message_id: &str,
        sender: &str,
        recipients: Vec<String>,
        created: &str,
        expires: &str,
    ) -> Result<(), MessageLifecycleError> {
        if message_id.trim().is_empty() {
            return Err(MessageLifecycleError::EmptyMessageId);
        }
        if self.records.contains_key(message_id) {
            return Err(MessageLifecycleError::DuplicateMessageId(
                message_id.to_owned(),
            ));
        }
        if let Err(error) = AgentDid::parse(sender) {
            return Err(MessageLifecycleError::InvalidSenderDid(error.to_string()));
        }
        if recipients.is_empty() {
            return Err(MessageLifecycleError::EmptyRecipients);
        }
        for recipient in &recipients {
            if let Err(error) = AgentDid::parse(recipient) {
                return Err(MessageLifecycleError::InvalidRecipientDid(
                    error.to_string(),
                ));
            }
        }
        if created.trim().is_empty() {
            return Err(MessageLifecycleError::EmptyTimestamp("created"));
        }
        if expires.trim().is_empty() {
            return Err(MessageLifecycleError::EmptyTimestamp("expires"));
        }
        if expires <= created {
            return Err(MessageLifecycleError::InvalidExpiryWindow {
                created: created.to_owned(),
                expires: expires.to_owned(),
            });
        }

        let id = message_id.to_owned();
        self.records.insert(
            id.clone(),
            MessageRecord {
                sender: sender.to_owned(),
                recipients: recipients.clone(),
                created: created.to_owned(),
                expires: expires.to_owned(),
                status: MessageStatus::Created,
                history: vec![MessageStatus::Created],
            },
        );

        self.ids_by_status
            .entry(MessageStatus::Created)
            .or_default()
            .insert(id.clone());
        self.ids_by_sender
            .entry(sender.to_owned())
            .or_default()
            .insert(id.clone());
        for recipient in recipients {
            self.ids_by_recipient
                .entry(recipient)
                .or_default()
                .insert(id.clone());
        }
        Ok(())
    }

    pub fn status(&self, message_id: &str) -> Result<MessageStatus, MessageLifecycleError> {
        self.records
            .get(message_id)
            .map(|record| record.status)
            .ok_or_else(|| MessageLifecycleError::NotFound(message_id.to_owned()))
    }

    pub fn transition(
        &mut self,
        message_id: &str,
        to: MessageStatus,
    ) -> Result<(), MessageLifecycleError> {
        let from = self
            .records
            .get(message_id)
            .map(|record| record.status)
            .ok_or_else(|| MessageLifecycleError::NotFound(message_id.to_owned()))?;
        if !is_valid_transition(from, to) {
            return Err(MessageLifecycleError::InvalidTransition { from, to });
        }

        self.apply_status(message_id, to)?;
        Ok(())
    }

    pub fn expire_message_if_overdue(
        &mut self,
        message_id: &str,
        observed_at: &str,
    ) -> Result<bool, MessageLifecycleError> {
        if observed_at.trim().is_empty() {
            return Err(MessageLifecycleError::EmptyTimestamp("observed_at"));
        }
        let record = self
            .records
            .get(message_id)
            .ok_or_else(|| MessageLifecycleError::NotFound(message_id.to_owned()))?;
        if !is_active_status(record.status) || observed_at <= record.expires.as_str() {
            return Ok(false);
        }

        self.apply_status(message_id, MessageStatus::Expired)?;
        Ok(true)
    }

    pub fn expire_overdue_messages(
        &mut self,
        observed_at: &str,
    ) -> Result<Vec<String>, MessageLifecycleError> {
        if observed_at.trim().is_empty() {
            return Err(MessageLifecycleError::EmptyTimestamp("observed_at"));
        }
        let overdue_ids: Vec<String> = self
            .records
            .iter()
            .filter_map(|(message_id, record)| {
                if is_active_status(record.status) && observed_at > record.expires.as_str() {
                    Some(message_id.clone())
                } else {
                    None
                }
            })
            .collect();
        for message_id in &overdue_ids {
            self.apply_status(message_id, MessageStatus::Expired)?;
        }
        Ok(overdue_ids)
    }

    pub fn ids_by_status(&self, status: MessageStatus) -> Vec<String> {
        self.ids_by_status
            .get(&status)
            .map(|ids| ids.iter().cloned().collect())
            .unwrap_or_default()
    }

    pub fn ids_by_sender(&self, sender: &str) -> Vec<String> {
        self.ids_by_sender
            .get(sender)
            .map(|ids| ids.iter().cloned().collect())
            .unwrap_or_default()
    }

    pub fn ids_by_recipient(&self, recipient: &str) -> Vec<String> {
        self.ids_by_recipient
            .get(recipient)
            .map(|ids| ids.iter().cloned().collect())
            .unwrap_or_default()
    }

    pub fn envelope_timestamps(
        &self,
        message_id: &str,
    ) -> Result<(&str, &str), MessageLifecycleError> {
        let record = self
            .records
            .get(message_id)
            .ok_or_else(|| MessageLifecycleError::NotFound(message_id.to_owned()))?;
        Ok((&record.created, &record.expires))
    }

    pub fn history(&self, message_id: &str) -> Result<&[MessageStatus], MessageLifecycleError> {
        let record = self
            .records
            .get(message_id)
            .ok_or_else(|| MessageLifecycleError::NotFound(message_id.to_owned()))?;
        Ok(&record.history)
    }

    pub fn participants(
        &self,
        message_id: &str,
    ) -> Result<(&str, &[String]), MessageLifecycleError> {
        let record = self
            .records
            .get(message_id)
            .ok_or_else(|| MessageLifecycleError::NotFound(message_id.to_owned()))?;
        Ok((&record.sender, &record.recipients))
    }

    pub fn export_snapshot(&self) -> MessageLifecycleSnapshot {
        let records = self
            .records
            .iter()
            .map(|(message_id, record)| MessageRecordSnapshot {
                message_id: message_id.clone(),
                sender: record.sender.clone(),
                recipients: record.recipients.clone(),
                created: record.created.clone(),
                expires: record.expires.clone(),
                status: record.status,
                history: record.history.clone(),
            })
            .collect();

        MessageLifecycleSnapshot {
            schema_version: MESSAGE_LIFECYCLE_SNAPSHOT_SCHEMA_VERSION,
            records,
        }
    }

    pub fn restore_snapshot(
        &mut self,
        snapshot: MessageLifecycleSnapshot,
    ) -> Result<(), MessageLifecycleSnapshotError> {
        if snapshot.schema_version != MESSAGE_LIFECYCLE_SNAPSHOT_SCHEMA_VERSION {
            return Err(MessageLifecycleSnapshotError::SnapshotVersionMismatch {
                expected: MESSAGE_LIFECYCLE_SNAPSHOT_SCHEMA_VERSION,
                found: snapshot.schema_version,
            });
        }

        let mut records = BTreeMap::new();
        let mut ids_by_status = BTreeMap::new();
        let mut ids_by_sender = BTreeMap::new();
        let mut ids_by_recipient = BTreeMap::new();

        for record_snapshot in snapshot.records {
            if records.contains_key(&record_snapshot.message_id) {
                return Err(MessageLifecycleSnapshotError::DuplicateMessageId(
                    record_snapshot.message_id,
                ));
            }

            validate_snapshot_record(&record_snapshot)
                .map_err(MessageLifecycleSnapshotError::Lifecycle)?;

            if record_snapshot.history.is_empty() {
                return Err(MessageLifecycleSnapshotError::InvalidSnapshot(format!(
                    "history cannot be empty for {}",
                    record_snapshot.message_id
                )));
            }
            if record_snapshot.history[0] != MessageStatus::Created {
                return Err(MessageLifecycleSnapshotError::InvalidSnapshot(format!(
                    "history must start with Created for {}",
                    record_snapshot.message_id
                )));
            }
            for transition in record_snapshot.history.windows(2) {
                let from = transition[0];
                let to = transition[1];
                if !is_valid_transition(from, to) {
                    return Err(MessageLifecycleSnapshotError::InvalidSnapshot(format!(
                        "invalid history transition for {}: {from:?}->{to:?}",
                        record_snapshot.message_id
                    )));
                }
            }

            let Some(last_status) = record_snapshot.history.last().copied() else {
                return Err(MessageLifecycleSnapshotError::InvalidSnapshot(format!(
                    "history cannot be empty for {}",
                    record_snapshot.message_id
                )));
            };
            if record_snapshot.status != last_status {
                return Err(MessageLifecycleSnapshotError::InvalidSnapshot(format!(
                    "status/history mismatch for {}",
                    record_snapshot.message_id
                )));
            }

            let message_id = record_snapshot.message_id.clone();
            ids_by_status
                .entry(record_snapshot.status)
                .or_insert_with(BTreeSet::new)
                .insert(message_id.clone());
            ids_by_sender
                .entry(record_snapshot.sender.clone())
                .or_insert_with(BTreeSet::new)
                .insert(message_id.clone());
            for recipient in &record_snapshot.recipients {
                ids_by_recipient
                    .entry(recipient.clone())
                    .or_insert_with(BTreeSet::new)
                    .insert(message_id.clone());
            }

            records.insert(
                message_id,
                MessageRecord {
                    sender: record_snapshot.sender,
                    recipients: record_snapshot.recipients,
                    created: record_snapshot.created,
                    expires: record_snapshot.expires,
                    status: record_snapshot.status,
                    history: record_snapshot.history,
                },
            );
        }

        self.records = records;
        self.ids_by_status = ids_by_status;
        self.ids_by_sender = ids_by_sender;
        self.ids_by_recipient = ids_by_recipient;
        Ok(())
    }

    fn apply_status(
        &mut self,
        message_id: &str,
        to: MessageStatus,
    ) -> Result<(), MessageLifecycleError> {
        let record = self
            .records
            .get_mut(message_id)
            .ok_or_else(|| MessageLifecycleError::NotFound(message_id.to_owned()))?;
        let from = record.status;
        if from == to {
            return Ok(());
        }
        record.status = to;
        record.history.push(to);

        if let Some(ids) = self.ids_by_status.get_mut(&from) {
            ids.remove(message_id);
        }
        self.ids_by_status
            .entry(to)
            .or_default()
            .insert(message_id.to_owned());
        Ok(())
    }

    pub fn validate_with_processor_proof(
        &mut self,
        message_id: &str,
        expected_payload_commitment: &str,
        artifact: ProcessorProofArtifact,
        evaluator: &mut ProcessorProofAdmissionEvaluator,
    ) -> Result<(), MessageProofAdmissionError> {
        let status = self
            .status(message_id)
            .map_err(MessageProofAdmissionError::Lifecycle)?;
        if status != MessageStatus::Delivered {
            return Err(MessageProofAdmissionError::InvalidValidationState { found: status });
        }

        let input =
            ProcessorProofAdmissionInput::new(message_id, expected_payload_commitment, artifact)
                .map_err(MessageProofAdmissionError::Proof)?;
        evaluator
            .evaluate(input)
            .map_err(MessageProofAdmissionError::Proof)?;

        self.transition(message_id, MessageStatus::Validated)
            .map_err(MessageProofAdmissionError::Lifecycle)?;
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MessageLifecycleError {
    EmptyMessageId,
    DuplicateMessageId(String),
    InvalidSenderDid(String),
    EmptyRecipients,
    InvalidRecipientDid(String),
    EmptyTimestamp(&'static str),
    InvalidExpiryWindow {
        created: String,
        expires: String,
    },
    NotFound(String),
    InvalidTransition {
        from: MessageStatus,
        to: MessageStatus,
    },
}

impl fmt::Display for MessageLifecycleError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyMessageId => write!(f, "message_id must not be empty"),
            Self::DuplicateMessageId(value) => write!(f, "duplicate message id: {value}"),
            Self::InvalidSenderDid(value) => write!(f, "invalid sender did: {value}"),
            Self::EmptyRecipients => write!(f, "recipients must not be empty"),
            Self::InvalidRecipientDid(value) => write!(f, "invalid recipient did: {value}"),
            Self::EmptyTimestamp(field) => write!(f, "{field} timestamp must not be empty"),
            Self::InvalidExpiryWindow { created, expires } => write!(
                f,
                "invalid message expiry window, created {created}, expires {expires}"
            ),
            Self::NotFound(value) => write!(f, "message not found: {value}"),
            Self::InvalidTransition { from, to } => {
                write!(
                    f,
                    "invalid message lifecycle transition from {from:?} to {to:?}"
                )
            }
        }
    }
}

impl std::error::Error for MessageLifecycleError {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MessageProofAdmissionError {
    Lifecycle(MessageLifecycleError),
    InvalidValidationState { found: MessageStatus },
    Proof(ZkDesignError),
}

impl fmt::Display for MessageProofAdmissionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Lifecycle(error) => write!(f, "{error}"),
            Self::InvalidValidationState { found } => write!(
                f,
                "message must be in Delivered state before processor proof validation (found {found:?})"
            ),
            Self::Proof(error) => write!(f, "{error}"),
        }
    }
}

impl std::error::Error for MessageProofAdmissionError {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MessageLifecycleSnapshotError {
    SnapshotVersionMismatch { expected: u16, found: u16 },
    DuplicateMessageId(String),
    InvalidSnapshot(String),
    Lifecycle(MessageLifecycleError),
}

impl fmt::Display for MessageLifecycleSnapshotError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::SnapshotVersionMismatch { expected, found } => write!(
                f,
                "message lifecycle snapshot version mismatch: expected {expected}, found {found}"
            ),
            Self::DuplicateMessageId(value) => {
                write!(f, "duplicate message id in snapshot: {value}")
            }
            Self::InvalidSnapshot(value) => {
                write!(f, "invalid message lifecycle snapshot: {value}")
            }
            Self::Lifecycle(error) => write!(f, "{error}"),
        }
    }
}

impl std::error::Error for MessageLifecycleSnapshotError {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MessageLifecycleSnapshotStoreError {
    Io(String),
    InvalidPayload(String),
    Snapshot(MessageLifecycleSnapshotError),
}

impl fmt::Display for MessageLifecycleSnapshotStoreError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(value) => write!(f, "message lifecycle snapshot store I/O error: {value}"),
            Self::InvalidPayload(value) => {
                write!(
                    f,
                    "message lifecycle snapshot store invalid payload: {value}"
                )
            }
            Self::Snapshot(error) => write!(f, "{error}"),
        }
    }
}

impl std::error::Error for MessageLifecycleSnapshotStoreError {}

pub trait MessageLifecycleSnapshotStore {
    fn write(
        &mut self,
        snapshot: MessageLifecycleSnapshot,
    ) -> Result<(), MessageLifecycleSnapshotStoreError>;
    fn read_latest(
        &self,
    ) -> Result<Option<MessageLifecycleSnapshot>, MessageLifecycleSnapshotStoreError>;
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct InMemoryMessageLifecycleSnapshotStore {
    latest: Option<MessageLifecycleSnapshot>,
}

impl MessageLifecycleSnapshotStore for InMemoryMessageLifecycleSnapshotStore {
    fn write(
        &mut self,
        snapshot: MessageLifecycleSnapshot,
    ) -> Result<(), MessageLifecycleSnapshotStoreError> {
        self.latest = Some(snapshot);
        Ok(())
    }

    fn read_latest(
        &self,
    ) -> Result<Option<MessageLifecycleSnapshot>, MessageLifecycleSnapshotStoreError> {
        Ok(self.latest.clone())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileMessageLifecycleSnapshotStore {
    path: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MessageLifecycleRecoveryResult {
    pub latest: Option<MessageLifecycleSnapshot>,
    pub repaired: bool,
}

impl FileMessageLifecycleSnapshotStore {
    pub fn new(path: PathBuf) -> Result<Self, MessageLifecycleSnapshotStoreError> {
        if path.as_os_str().is_empty() {
            return Err(MessageLifecycleSnapshotStoreError::InvalidPayload(
                "snapshot file path cannot be empty".to_owned(),
            ));
        }
        Ok(Self { path })
    }

    pub fn recover_latest_and_repair(
        &mut self,
    ) -> Result<MessageLifecycleRecoveryResult, MessageLifecycleSnapshotStoreError> {
        if !self.path.exists() {
            return Ok(MessageLifecycleRecoveryResult {
                latest: None,
                repaired: false,
            });
        }

        match self.read_latest() {
            Ok(snapshot) => Ok(MessageLifecycleRecoveryResult {
                latest: snapshot,
                repaired: false,
            }),
            Err(MessageLifecycleSnapshotStoreError::InvalidPayload(_))
            | Err(MessageLifecycleSnapshotStoreError::Snapshot(_)) => {
                fs::write(&self.path, "")
                    .map_err(|error| MessageLifecycleSnapshotStoreError::Io(error.to_string()))?;
                Ok(MessageLifecycleRecoveryResult {
                    latest: None,
                    repaired: true,
                })
            }
            Err(error) => Err(error),
        }
    }
}

impl MessageLifecycleSnapshotStore for FileMessageLifecycleSnapshotStore {
    fn write(
        &mut self,
        snapshot: MessageLifecycleSnapshot,
    ) -> Result<(), MessageLifecycleSnapshotStoreError> {
        let mut verifier = MessageLifecycleStore::new();
        verifier
            .restore_snapshot(snapshot.clone())
            .map_err(MessageLifecycleSnapshotStoreError::Snapshot)?;
        let payload = serialize_message_lifecycle_snapshot(&snapshot)?;
        let mut file = OpenOptions::new()
            .create(true)
            .truncate(true)
            .write(true)
            .open(&self.path)
            .map_err(|error| MessageLifecycleSnapshotStoreError::Io(error.to_string()))?;
        file.write_all(payload.as_bytes())
            .map_err(|error| MessageLifecycleSnapshotStoreError::Io(error.to_string()))
    }

    fn read_latest(
        &self,
    ) -> Result<Option<MessageLifecycleSnapshot>, MessageLifecycleSnapshotStoreError> {
        if !self.path.exists() {
            return Ok(None);
        }

        let payload = fs::read_to_string(&self.path)
            .map_err(|error| MessageLifecycleSnapshotStoreError::Io(error.to_string()))?;
        if payload.trim().is_empty() {
            return Ok(None);
        }
        let snapshot = parse_message_lifecycle_snapshot_payload(&payload)?;
        let mut verifier = MessageLifecycleStore::new();
        verifier
            .restore_snapshot(snapshot.clone())
            .map_err(MessageLifecycleSnapshotStoreError::Snapshot)?;
        Ok(Some(snapshot))
    }
}

fn is_valid_transition(from: MessageStatus, to: MessageStatus) -> bool {
    matches!(
        (from, to),
        (MessageStatus::Created, MessageStatus::Signed)
            | (MessageStatus::Signed, MessageStatus::Broadcast)
            | (MessageStatus::Broadcast, MessageStatus::Included)
            | (MessageStatus::Included, MessageStatus::Delivered)
            | (MessageStatus::Delivered, MessageStatus::Validated)
            | (MessageStatus::Validated, MessageStatus::Rejected)
            | (MessageStatus::Rejected, MessageStatus::Expired)
    )
}

fn is_active_status(status: MessageStatus) -> bool {
    matches!(
        status,
        MessageStatus::Created
            | MessageStatus::Signed
            | MessageStatus::Broadcast
            | MessageStatus::Included
            | MessageStatus::Delivered
    )
}

fn validate_snapshot_record(record: &MessageRecordSnapshot) -> Result<(), MessageLifecycleError> {
    if record.message_id.trim().is_empty() {
        return Err(MessageLifecycleError::EmptyMessageId);
    }
    if let Err(error) = AgentDid::parse(&record.sender) {
        return Err(MessageLifecycleError::InvalidSenderDid(error.to_string()));
    }
    if record.recipients.is_empty() {
        return Err(MessageLifecycleError::EmptyRecipients);
    }
    for recipient in &record.recipients {
        if let Err(error) = AgentDid::parse(recipient) {
            return Err(MessageLifecycleError::InvalidRecipientDid(
                error.to_string(),
            ));
        }
    }
    if record.created.trim().is_empty() {
        return Err(MessageLifecycleError::EmptyTimestamp("created"));
    }
    if record.expires.trim().is_empty() {
        return Err(MessageLifecycleError::EmptyTimestamp("expires"));
    }
    if record.expires <= record.created {
        return Err(MessageLifecycleError::InvalidExpiryWindow {
            created: record.created.clone(),
            expires: record.expires.clone(),
        });
    }
    Ok(())
}

fn message_status_code(status: MessageStatus) -> &'static str {
    match status {
        MessageStatus::Created => "0",
        MessageStatus::Signed => "1",
        MessageStatus::Broadcast => "2",
        MessageStatus::Included => "3",
        MessageStatus::Delivered => "4",
        MessageStatus::Validated => "5",
        MessageStatus::Rejected => "6",
        MessageStatus::Expired => "7",
    }
}

fn parse_message_status_code(raw: &str) -> Option<MessageStatus> {
    match raw {
        "0" => Some(MessageStatus::Created),
        "1" => Some(MessageStatus::Signed),
        "2" => Some(MessageStatus::Broadcast),
        "3" => Some(MessageStatus::Included),
        "4" => Some(MessageStatus::Delivered),
        "5" => Some(MessageStatus::Validated),
        "6" => Some(MessageStatus::Rejected),
        "7" => Some(MessageStatus::Expired),
        _ => None,
    }
}

fn ensure_snapshot_token(
    value: &str,
    field: &str,
    allow_comma: bool,
) -> Result<(), MessageLifecycleSnapshotStoreError> {
    let has_comma = !allow_comma && value.contains(',');
    if value.contains('|') || value.contains('\n') || value.contains('\r') || has_comma {
        return Err(MessageLifecycleSnapshotStoreError::InvalidPayload(format!(
            "{field} contains unsupported delimiter characters"
        )));
    }
    Ok(())
}

fn serialize_message_lifecycle_snapshot(
    snapshot: &MessageLifecycleSnapshot,
) -> Result<String, MessageLifecycleSnapshotStoreError> {
    let mut payload = format!("schema|{}\n", snapshot.schema_version);
    for record in &snapshot.records {
        ensure_snapshot_token(&record.message_id, "message_id", false)?;
        ensure_snapshot_token(&record.sender, "sender", false)?;
        ensure_snapshot_token(&record.created, "created", false)?;
        ensure_snapshot_token(&record.expires, "expires", false)?;
        for recipient in &record.recipients {
            ensure_snapshot_token(recipient, "recipient", false)?;
        }

        let recipients = record.recipients.join(",");
        let history = record
            .history
            .iter()
            .map(|status| message_status_code(*status))
            .collect::<Vec<_>>()
            .join(",");
        payload.push_str(&format!(
            "record|{}|{}|{}|{}|{}|{}|{}\n",
            record.message_id,
            record.sender,
            recipients,
            record.created,
            record.expires,
            message_status_code(record.status),
            history
        ));
    }
    Ok(payload)
}

fn parse_message_lifecycle_snapshot_payload(
    payload: &str,
) -> Result<MessageLifecycleSnapshot, MessageLifecycleSnapshotStoreError> {
    let mut lines = payload.lines().filter(|line| !line.trim().is_empty());
    let Some(schema_line) = lines.next() else {
        return Err(MessageLifecycleSnapshotStoreError::InvalidPayload(
            "missing schema line".to_owned(),
        ));
    };

    let mut schema_parts = schema_line.split('|');
    let Some(schema_prefix) = schema_parts.next() else {
        return Err(MessageLifecycleSnapshotStoreError::InvalidPayload(
            schema_line.to_owned(),
        ));
    };
    let Some(schema_version_raw) = schema_parts.next() else {
        return Err(MessageLifecycleSnapshotStoreError::InvalidPayload(
            schema_line.to_owned(),
        ));
    };
    if schema_prefix != "schema" || schema_parts.next().is_some() {
        return Err(MessageLifecycleSnapshotStoreError::InvalidPayload(
            schema_line.to_owned(),
        ));
    }
    let schema_version = schema_version_raw
        .parse::<u16>()
        .map_err(|_| MessageLifecycleSnapshotStoreError::InvalidPayload(schema_line.to_owned()))?;

    let mut records = Vec::new();
    for line in lines {
        let mut parts = line.split('|');
        let Some(prefix) = parts.next() else {
            return Err(MessageLifecycleSnapshotStoreError::InvalidPayload(
                line.to_owned(),
            ));
        };
        if prefix != "record" {
            return Err(MessageLifecycleSnapshotStoreError::InvalidPayload(
                line.to_owned(),
            ));
        }
        let Some(message_id) = parts.next() else {
            return Err(MessageLifecycleSnapshotStoreError::InvalidPayload(
                line.to_owned(),
            ));
        };
        let Some(sender) = parts.next() else {
            return Err(MessageLifecycleSnapshotStoreError::InvalidPayload(
                line.to_owned(),
            ));
        };
        let Some(recipients_raw) = parts.next() else {
            return Err(MessageLifecycleSnapshotStoreError::InvalidPayload(
                line.to_owned(),
            ));
        };
        let Some(created) = parts.next() else {
            return Err(MessageLifecycleSnapshotStoreError::InvalidPayload(
                line.to_owned(),
            ));
        };
        let Some(expires) = parts.next() else {
            return Err(MessageLifecycleSnapshotStoreError::InvalidPayload(
                line.to_owned(),
            ));
        };
        let Some(status_raw) = parts.next() else {
            return Err(MessageLifecycleSnapshotStoreError::InvalidPayload(
                line.to_owned(),
            ));
        };
        let Some(history_raw) = parts.next() else {
            return Err(MessageLifecycleSnapshotStoreError::InvalidPayload(
                line.to_owned(),
            ));
        };
        if parts.next().is_some() {
            return Err(MessageLifecycleSnapshotStoreError::InvalidPayload(
                line.to_owned(),
            ));
        }

        let recipients = if recipients_raw.is_empty() {
            Vec::new()
        } else {
            recipients_raw
                .split(',')
                .map(|value| value.to_owned())
                .collect::<Vec<_>>()
        };
        let status = parse_message_status_code(status_raw)
            .ok_or_else(|| MessageLifecycleSnapshotStoreError::InvalidPayload(line.to_owned()))?;
        let history = if history_raw.is_empty() {
            Vec::new()
        } else {
            history_raw
                .split(',')
                .map(|raw| {
                    parse_message_status_code(raw).ok_or_else(|| {
                        MessageLifecycleSnapshotStoreError::InvalidPayload(line.to_owned())
                    })
                })
                .collect::<Result<Vec<_>, _>>()?
        };

        records.push(MessageRecordSnapshot {
            message_id: message_id.to_owned(),
            sender: sender.to_owned(),
            recipients,
            created: created.to_owned(),
            expires: expires.to_owned(),
            status,
            history,
        });
    }

    Ok(MessageLifecycleSnapshot {
        schema_version,
        records,
    })
}

#[cfg(test)]
mod tests {
    use super::{
        FileMessageLifecycleSnapshotStore, MessageLifecycleError, MessageLifecycleSnapshot,
        MessageLifecycleSnapshotError, MessageLifecycleSnapshotStore,
        MessageLifecycleSnapshotStoreError, MessageLifecycleStore, MessageProofAdmissionError,
        MessageStatus,
    };
    use crate::{ProcessorProofAdmissionEvaluator, ProcessorProofArtifact, ZkDesignError};
    use std::fs;
    use std::path::PathBuf;
    use std::time::{Instant, SystemTime, UNIX_EPOCH};

    #[test]
    fn register_rejects_duplicate_id() {
        let mut store = MessageLifecycleStore::new();
        store
            .register(
                "urn:uuid:msg-1",
                "kamn:did:agent:sender-1",
                vec!["kamn:did:agent:recipient-1".to_owned()],
                "2026-02-07T20:15:30.123Z",
                "2026-02-07T20:45:30.123Z",
            )
            .expect("initial register should succeed");

        assert_eq!(
            store.register(
                "urn:uuid:msg-1",
                "kamn:did:agent:sender-1",
                vec!["kamn:did:agent:recipient-1".to_owned()],
                "2026-02-07T20:15:30.123Z",
                "2026-02-07T20:45:30.123Z",
            ),
            Err(MessageLifecycleError::DuplicateMessageId(
                "urn:uuid:msg-1".to_owned()
            ))
        );
    }

    #[test]
    fn transition_updates_status_index() {
        let mut store = MessageLifecycleStore::new();
        store
            .register(
                "urn:uuid:msg-2",
                "kamn:did:agent:sender-1",
                vec!["kamn:did:agent:recipient-1".to_owned()],
                "2026-02-07T20:15:30.123Z",
                "2026-02-07T20:45:30.123Z",
            )
            .expect("register should succeed");

        store
            .transition("urn:uuid:msg-2", MessageStatus::Signed)
            .expect("created->signed should succeed");
        assert!(store.ids_by_status(MessageStatus::Created).is_empty());
        assert_eq!(
            store.ids_by_status(MessageStatus::Signed),
            vec!["urn:uuid:msg-2".to_owned()]
        );
    }

    #[test]
    fn expire_message_if_overdue_rejects_empty_observed_timestamp() {
        let mut store = MessageLifecycleStore::new();
        store
            .register(
                "urn:uuid:msg-2a",
                "kamn:did:agent:sender-1",
                vec!["kamn:did:agent:recipient-1".to_owned()],
                "2026-02-07T20:15:30.123Z",
                "2026-02-07T20:45:30.123Z",
            )
            .expect("register should succeed");

        assert_eq!(
            store.expire_message_if_overdue("urn:uuid:msg-2a", " "),
            Err(MessageLifecycleError::EmptyTimestamp("observed_at"))
        );
    }

    #[test]
    fn expire_overdue_messages_expires_active_records_only() {
        let mut store = MessageLifecycleStore::new();
        store
            .register(
                "urn:uuid:msg-2b",
                "kamn:did:agent:sender-1",
                vec!["kamn:did:agent:recipient-1".to_owned()],
                "2026-02-07T20:15:30.123Z",
                "2026-02-07T20:45:30.123Z",
            )
            .expect("register should succeed");
        store
            .register(
                "urn:uuid:msg-2c",
                "kamn:did:agent:sender-1",
                vec!["kamn:did:agent:recipient-1".to_owned()],
                "2026-02-07T20:15:30.123Z",
                "2026-02-07T20:45:30.123Z",
            )
            .expect("register should succeed");
        store
            .transition("urn:uuid:msg-2c", MessageStatus::Signed)
            .expect("created->signed should succeed");
        store
            .transition("urn:uuid:msg-2c", MessageStatus::Broadcast)
            .expect("signed->broadcast should succeed");
        store
            .transition("urn:uuid:msg-2c", MessageStatus::Included)
            .expect("broadcast->included should succeed");
        store
            .transition("urn:uuid:msg-2c", MessageStatus::Delivered)
            .expect("included->delivered should succeed");
        store
            .transition("urn:uuid:msg-2c", MessageStatus::Validated)
            .expect("delivered->validated should succeed");

        assert_eq!(
            store
                .expire_overdue_messages("2026-02-07T20:50:30.123Z")
                .expect("sweep should succeed"),
            vec!["urn:uuid:msg-2b".to_owned()]
        );
        assert_eq!(
            store
                .status("urn:uuid:msg-2b")
                .expect("status should exist"),
            MessageStatus::Expired
        );
        assert_eq!(
            store
                .status("urn:uuid:msg-2c")
                .expect("status should exist"),
            MessageStatus::Validated
        );
    }

    #[test]
    fn validate_with_processor_proof_rejects_non_delivered_state() {
        let mut store = MessageLifecycleStore::new();
        store
            .register(
                "urn:uuid:msg-3",
                "kamn:did:agent:sender-1",
                vec!["kamn:did:agent:recipient-1".to_owned()],
                "2026-02-07T20:15:30.123Z",
                "2026-02-07T20:45:30.123Z",
            )
            .expect("register should succeed");

        let mut evaluator = ProcessorProofAdmissionEvaluator::new();
        let artifact = ProcessorProofArtifact::new(
            "artifact-1",
            "urn:uuid:msg-3",
            "fnv1a64:abc",
            "proof:ok:artifact-1",
        )
        .expect("artifact should parse");

        assert_eq!(
            store.validate_with_processor_proof(
                "urn:uuid:msg-3",
                "fnv1a64:abc",
                artifact,
                &mut evaluator
            ),
            Err(MessageProofAdmissionError::InvalidValidationState {
                found: MessageStatus::Created
            })
        );
    }

    #[test]
    fn validate_with_processor_proof_maps_proof_errors() {
        let mut store = MessageLifecycleStore::new();
        store
            .register(
                "urn:uuid:msg-4",
                "kamn:did:agent:sender-1",
                vec!["kamn:did:agent:recipient-1".to_owned()],
                "2026-02-07T20:15:30.123Z",
                "2026-02-07T20:45:30.123Z",
            )
            .expect("register should succeed");
        store
            .transition("urn:uuid:msg-4", MessageStatus::Signed)
            .expect("created->signed should succeed");
        store
            .transition("urn:uuid:msg-4", MessageStatus::Broadcast)
            .expect("signed->broadcast should succeed");
        store
            .transition("urn:uuid:msg-4", MessageStatus::Included)
            .expect("broadcast->included should succeed");
        store
            .transition("urn:uuid:msg-4", MessageStatus::Delivered)
            .expect("included->delivered should succeed");

        let mut evaluator = ProcessorProofAdmissionEvaluator::new();
        let artifact = ProcessorProofArtifact::new(
            "artifact-2",
            "urn:uuid:msg-4",
            "fnv1a64:abc",
            "proof:tampered:artifact-2",
        )
        .expect("artifact should parse");

        assert_eq!(
            store.validate_with_processor_proof(
                "urn:uuid:msg-4",
                "fnv1a64:abc",
                artifact,
                &mut evaluator
            ),
            Err(MessageProofAdmissionError::Proof(
                ZkDesignError::ProofVerificationFailed {
                    artifact_id: "artifact-2".to_owned(),
                    reason: "proof value failed deterministic verification".to_owned(),
                }
            ))
        );
    }

    #[test]
    fn functional_message_lifecycle_snapshot_roundtrip_restores_indexes() {
        let mut store = MessageLifecycleStore::new();
        store
            .register(
                "urn:uuid:msg-snapshot-1",
                "kamn:did:agent:sender-1",
                vec![
                    "kamn:did:agent:recipient-1".to_owned(),
                    "kamn:did:agent:recipient-2".to_owned(),
                ],
                "2026-02-07T20:15:30.123Z",
                "2026-02-07T20:45:30.123Z",
            )
            .expect("register should succeed");
        store
            .transition("urn:uuid:msg-snapshot-1", MessageStatus::Signed)
            .expect("created->signed should succeed");

        let snapshot = store.export_snapshot();
        let mut restored = MessageLifecycleStore::new();
        restored
            .restore_snapshot(snapshot)
            .expect("snapshot restore should pass");

        assert_eq!(
            restored
                .status("urn:uuid:msg-snapshot-1")
                .expect("status should exist"),
            MessageStatus::Signed
        );
        assert_eq!(
            restored.ids_by_sender("kamn:did:agent:sender-1"),
            vec!["urn:uuid:msg-snapshot-1".to_owned()]
        );
        assert_eq!(
            restored.ids_by_recipient("kamn:did:agent:recipient-1"),
            vec!["urn:uuid:msg-snapshot-1".to_owned()]
        );
    }

    #[test]
    fn regression_message_lifecycle_snapshot_restore_rejects_duplicate_message_ids() {
        // Regression: #617
        let mut store = MessageLifecycleStore::new();
        store
            .register(
                "urn:uuid:msg-snapshot-2",
                "kamn:did:agent:sender-1",
                vec!["kamn:did:agent:recipient-1".to_owned()],
                "2026-02-07T20:15:30.123Z",
                "2026-02-07T20:45:30.123Z",
            )
            .expect("register should succeed");
        let mut snapshot = store.export_snapshot();
        snapshot.records.push(snapshot.records[0].clone());

        let mut restored = MessageLifecycleStore::new();
        assert_eq!(
            restored.restore_snapshot(snapshot),
            Err(MessageLifecycleSnapshotError::DuplicateMessageId(
                "urn:uuid:msg-snapshot-2".to_owned()
            ))
        );
    }

    #[test]
    fn regression_message_lifecycle_snapshot_restore_rejects_status_history_mismatch() {
        // Regression: #617
        let snapshot = MessageLifecycleSnapshot {
            schema_version: 1,
            records: vec![super::MessageRecordSnapshot {
                message_id: "urn:uuid:msg-snapshot-3".to_owned(),
                sender: "kamn:did:agent:sender-1".to_owned(),
                recipients: vec!["kamn:did:agent:recipient-1".to_owned()],
                created: "2026-02-07T20:15:30.123Z".to_owned(),
                expires: "2026-02-07T20:45:30.123Z".to_owned(),
                status: MessageStatus::Delivered,
                history: vec![MessageStatus::Created, MessageStatus::Signed],
            }],
        };

        let mut restored = MessageLifecycleStore::new();
        assert_eq!(
            restored.restore_snapshot(snapshot),
            Err(MessageLifecycleSnapshotError::InvalidSnapshot(
                "status/history mismatch for urn:uuid:msg-snapshot-3".to_owned()
            ))
        );
    }

    #[test]
    fn integration_file_message_lifecycle_snapshot_store_roundtrips_snapshot() {
        let path = temp_message_lifecycle_snapshot_path("roundtrip");
        let _ = fs::remove_file(&path);

        let mut store = MessageLifecycleStore::new();
        store
            .register(
                "urn:uuid:msg-snapshot-4",
                "kamn:did:agent:sender-1",
                vec!["kamn:did:agent:recipient-1".to_owned()],
                "2026-02-07T20:15:30.123Z",
                "2026-02-07T20:45:30.123Z",
            )
            .expect("register should succeed");
        let snapshot = store.export_snapshot();

        let mut file_store =
            FileMessageLifecycleSnapshotStore::new(path.clone()).expect("store should build");
        assert!(file_store.write(snapshot.clone()).is_ok());
        assert_eq!(
            file_store.read_latest().expect("read should pass"),
            Some(snapshot)
        );

        let _ = fs::remove_file(path);
    }

    #[test]
    fn regression_file_message_lifecycle_snapshot_store_rejects_malformed_payload() {
        // Regression: #617
        let path = temp_message_lifecycle_snapshot_path("malformed");
        let _ = fs::remove_file(&path);
        assert!(fs::write(&path, "schema|1\nrecord|broken\n").is_ok());

        let file_store = FileMessageLifecycleSnapshotStore::new(path.clone()).expect("store");
        assert_eq!(
            file_store.read_latest(),
            Err(MessageLifecycleSnapshotStoreError::InvalidPayload(
                "record|broken".to_owned()
            ))
        );

        let _ = fs::remove_file(path);
    }

    #[test]
    fn functional_file_message_lifecycle_snapshot_store_recovery_repairs_corrupt_payload() {
        let path = temp_message_lifecycle_snapshot_path("recover");
        let _ = fs::remove_file(&path);
        assert!(fs::write(&path, "schema|1\nrecord|broken\n").is_ok());

        let mut file_store = FileMessageLifecycleSnapshotStore::new(path.clone()).expect("store");
        let recovery = file_store
            .recover_latest_and_repair()
            .expect("recovery should pass");
        assert!(recovery.latest.is_none());
        assert!(recovery.repaired);
        assert_eq!(
            fs::read_to_string(&path).expect("repaired file should be readable"),
            ""
        );

        let _ = fs::remove_file(path);
    }

    #[test]
    fn performance_message_lifecycle_snapshot_roundtrip_stays_within_ci_budget() {
        let mut store = MessageLifecycleStore::new();
        for index in 0..256 {
            store
                .register(
                    &format!("urn:uuid:msg-snapshot-perf-{index}"),
                    "kamn:did:agent:sender-1",
                    vec!["kamn:did:agent:recipient-1".to_owned()],
                    "2026-02-07T20:15:30.123Z",
                    "2026-02-07T20:45:30.123Z",
                )
                .expect("register should succeed");
        }

        let snapshot = store.export_snapshot();
        let mut restored = MessageLifecycleStore::new();
        let start = Instant::now();
        restored
            .restore_snapshot(snapshot)
            .expect("snapshot restore should pass");
        let elapsed_millis = start.elapsed().as_millis();
        assert!(
            elapsed_millis < 250,
            "message lifecycle snapshot roundtrip exceeded CI budget: {elapsed_millis}ms"
        );
    }

    #[test]
    #[ignore = "scheduled message lifecycle deep lane"]
    fn performance_message_lifecycle_snapshot_deep_lane_stress() {
        let mut store = MessageLifecycleStore::new();
        for index in 0..6000 {
            store
                .register(
                    &format!("urn:uuid:msg-snapshot-deep-{index}"),
                    "kamn:did:agent:sender-1",
                    vec!["kamn:did:agent:recipient-1".to_owned()],
                    "2026-02-07T20:15:30.123Z",
                    "2026-02-07T20:45:30.123Z",
                )
                .expect("register should succeed");
        }
        let snapshot = store.export_snapshot();
        let mut restored = MessageLifecycleStore::new();
        restored
            .restore_snapshot(snapshot)
            .expect("snapshot restore should pass");
    }

    fn temp_message_lifecycle_snapshot_path(tag: &str) -> PathBuf {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock should be monotonic")
            .as_nanos();
        std::env::temp_dir().join(format!("kamn-message-lifecycle-snapshot-{tag}-{nonce}.log"))
    }
}
