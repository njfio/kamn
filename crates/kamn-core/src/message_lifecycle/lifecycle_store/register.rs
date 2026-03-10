use super::transitions::is_expirable_status;
use super::*;
use crate::AgentDid;

impl MessageLifecycleStore {
    /// Registers a new message with sender/recipient metadata and lifecycle timestamps.
    pub fn register(
        &mut self,
        message_id: &str,
        sender: &str,
        recipients: Vec<String>,
        created: &str,
        expires: &str,
    ) -> Result<(), MessageLifecycleError> {
        validate_registration_request(self, message_id, sender, &recipients, created, expires)?;
        let id = message_id.to_owned();
        let recipients_for_indexes = recipients.clone();
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
        index_registered_message(self, &id, sender, recipients_for_indexes);
        Ok(())
    }

    /// Expires one message when `observed_at` is after the stored expiry timestamp.
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
        if !is_expirable_status(record.status) || observed_at <= record.expires.as_str() {
            return Ok(false);
        }

        self.transition(message_id, MessageStatus::Expired)?;
        Ok(true)
    }

    /// Expires all active messages that are overdue at `observed_at`.
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
                if is_expirable_status(record.status) && observed_at > record.expires.as_str() {
                    Some(message_id.clone())
                } else {
                    None
                }
            })
            .collect();
        for message_id in &overdue_ids {
            self.transition(message_id, MessageStatus::Expired)?;
        }
        Ok(overdue_ids)
    }
}

fn validate_registration_request(
    store: &MessageLifecycleStore,
    message_id: &str,
    sender: &str,
    recipients: &[String],
    created: &str,
    expires: &str,
) -> Result<(), MessageLifecycleError> {
    validate_message_id(store, message_id)?;
    validate_sender(sender)?;
    validate_recipients(recipients)?;
    validate_expiry_window(created, expires)
}

fn validate_message_id(
    store: &MessageLifecycleStore,
    message_id: &str,
) -> Result<(), MessageLifecycleError> {
    if message_id.trim().is_empty() {
        return Err(MessageLifecycleError::EmptyMessageId);
    }
    if store.records.contains_key(message_id) {
        return Err(MessageLifecycleError::DuplicateMessageId(
            message_id.to_owned(),
        ));
    }
    Ok(())
}

fn validate_sender(sender: &str) -> Result<(), MessageLifecycleError> {
    AgentDid::parse(sender)
        .map(|_| ())
        .map_err(|error| MessageLifecycleError::InvalidSenderDid(error.to_string()))
}

fn validate_recipients(recipients: &[String]) -> Result<(), MessageLifecycleError> {
    if recipients.is_empty() {
        return Err(MessageLifecycleError::EmptyRecipients);
    }
    for recipient in recipients {
        AgentDid::parse(recipient)
            .map_err(|error| MessageLifecycleError::InvalidRecipientDid(error.to_string()))?;
    }
    Ok(())
}

fn validate_expiry_window(created: &str, expires: &str) -> Result<(), MessageLifecycleError> {
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
    Ok(())
}

fn index_registered_message(
    store: &mut MessageLifecycleStore,
    message_id: &str,
    sender: &str,
    recipients: Vec<String>,
) {
    index_message_status(store, message_id, MessageStatus::Created);
    index_message_sender(store, message_id, sender);
    index_message_recipients(store, message_id, recipients);
}

fn index_message_status(
    store: &mut MessageLifecycleStore,
    message_id: &str,
    status: MessageStatus,
) {
    store
        .ids_by_status
        .entry(status)
        .or_default()
        .insert(message_id.to_owned());
}

fn index_message_sender(store: &mut MessageLifecycleStore, message_id: &str, sender: &str) {
    store
        .ids_by_sender
        .entry(sender.to_owned())
        .or_default()
        .insert(message_id.to_owned());
}

fn index_message_recipients(
    store: &mut MessageLifecycleStore,
    message_id: &str,
    recipients: Vec<String>,
) {
    for recipient in recipients {
        store
            .ids_by_recipient
            .entry(recipient)
            .or_default()
            .insert(message_id.to_owned());
    }
}
