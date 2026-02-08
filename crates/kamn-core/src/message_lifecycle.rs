use crate::{
    AgentDid, ProcessorProofAdmissionEvaluator, ProcessorProofAdmissionInput,
    ProcessorProofArtifact, ZkDesignError,
};
use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

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

#[cfg(test)]
mod tests {
    use super::{
        MessageLifecycleError, MessageLifecycleStore, MessageProofAdmissionError, MessageStatus,
    };
    use crate::{ProcessorProofAdmissionEvaluator, ProcessorProofArtifact, ZkDesignError};

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
}
