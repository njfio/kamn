use super::*;
use crate::AgentDid;

impl MessageLifecycleStore {
    /// Applies a lifecycle transition when the edge is valid under policy.
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
}

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

pub(super) fn validate_snapshot_record(
    record: &MessageRecordSnapshot,
) -> Result<(), MessageLifecycleError> {
    validate_snapshot_message_id(record)?;
    validate_snapshot_sender(record)?;
    validate_snapshot_recipients(record)?;
    validate_snapshot_expiry_window(record)
}

fn validate_snapshot_message_id(
    record: &MessageRecordSnapshot,
) -> Result<(), MessageLifecycleError> {
    if record.message_id.trim().is_empty() {
        return Err(MessageLifecycleError::EmptyMessageId);
    }
    Ok(())
}

fn validate_snapshot_sender(record: &MessageRecordSnapshot) -> Result<(), MessageLifecycleError> {
    AgentDid::parse(&record.sender)
        .map(|_| ())
        .map_err(|error| MessageLifecycleError::InvalidSenderDid(error.to_string()))
}

fn validate_snapshot_recipients(
    record: &MessageRecordSnapshot,
) -> Result<(), MessageLifecycleError> {
    if record.recipients.is_empty() {
        return Err(MessageLifecycleError::EmptyRecipients);
    }
    for recipient in &record.recipients {
        AgentDid::parse(recipient)
            .map_err(|error| MessageLifecycleError::InvalidRecipientDid(error.to_string()))?;
    }
    Ok(())
}

fn validate_snapshot_expiry_window(
    record: &MessageRecordSnapshot,
) -> Result<(), MessageLifecycleError> {
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
