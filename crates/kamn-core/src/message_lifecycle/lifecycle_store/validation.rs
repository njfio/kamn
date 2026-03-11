use super::{MessageRecord, MessageRecordSnapshot};
use crate::message_lifecycle::{MessageLifecycleError, MessageStatus};
use crate::AgentDid;

pub(super) fn is_valid_transition(from: MessageStatus, to: MessageStatus) -> bool {
    matches!(
        (from, to),
        (MessageStatus::Created, MessageStatus::Signed)
            | (MessageStatus::Created, MessageStatus::Expired)
            | (MessageStatus::Signed, MessageStatus::Broadcast)
            | (MessageStatus::Signed, MessageStatus::Expired)
            | (MessageStatus::Broadcast, MessageStatus::Included)
            | (MessageStatus::Broadcast, MessageStatus::Expired)
            | (MessageStatus::Included, MessageStatus::Delivered)
            | (MessageStatus::Included, MessageStatus::Expired)
            | (MessageStatus::Delivered, MessageStatus::Validated)
            | (MessageStatus::Delivered, MessageStatus::Expired)
            | (MessageStatus::Validated, MessageStatus::Rejected)
            | (MessageStatus::Validated, MessageStatus::Expired)
            | (MessageStatus::Rejected, MessageStatus::Expired)
    )
}

pub(super) fn is_expirable_status(status: MessageStatus) -> bool {
    matches!(
        status,
        MessageStatus::Created
            | MessageStatus::Signed
            | MessageStatus::Broadcast
            | MessageStatus::Included
            | MessageStatus::Delivered
            | MessageStatus::Validated
    )
}

pub(super) fn validate_registration_request(
    message_id: &str,
    sender: &str,
    recipients: &[String],
    created: &str,
    expires: &str,
) -> Result<(), MessageLifecycleError> {
    validate_message_identity(message_id, sender)?;
    validate_recipients(recipients)?;
    validate_timestamps(created, expires)
}

pub(super) fn validate_snapshot_record(
    record: &MessageRecordSnapshot,
) -> Result<(), MessageLifecycleError> {
    validate_message_identity(&record.message_id, &record.sender)?;
    validate_recipients(&record.recipients)?;
    validate_timestamps(&record.created, &record.expires)
}

pub(super) fn build_message_record(
    sender: String,
    recipients: Vec<String>,
    created: String,
    expires: String,
    status: MessageStatus,
    history: Vec<MessageStatus>,
) -> MessageRecord {
    MessageRecord {
        sender,
        recipients,
        created,
        expires,
        status,
        history,
    }
}

fn validate_message_identity(message_id: &str, sender: &str) -> Result<(), MessageLifecycleError> {
    if message_id.trim().is_empty() {
        return Err(MessageLifecycleError::EmptyMessageId);
    }
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

fn validate_timestamps(created: &str, expires: &str) -> Result<(), MessageLifecycleError> {
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
