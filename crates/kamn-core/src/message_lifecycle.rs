use crate::AgentDid;
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
        let record = self
            .records
            .get_mut(message_id)
            .ok_or_else(|| MessageLifecycleError::NotFound(message_id.to_owned()))?;
        let from = record.status;
        if !is_valid_transition(from, to) {
            return Err(MessageLifecycleError::InvalidTransition { from, to });
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

#[cfg(test)]
mod tests {
    use super::{MessageLifecycleError, MessageLifecycleStore, MessageStatus};

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
}
