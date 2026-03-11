use super::MessageLifecycleStore;
use crate::message_lifecycle::{MessageLifecycleError, MessageStatus};

impl MessageLifecycleStore {
    /// Returns message ids in the provided lifecycle status.
    pub fn ids_by_status(&self, status: MessageStatus) -> Vec<String> {
        self.collect_index_values(self.ids_by_status.get(&status))
    }

    /// Returns message ids registered by the sender DID.
    pub fn ids_by_sender(&self, sender: &str) -> Vec<String> {
        self.collect_index_values(self.ids_by_sender.get(sender))
    }

    /// Returns message ids that include the provided recipient DID.
    pub fn ids_by_recipient(&self, recipient: &str) -> Vec<String> {
        self.collect_index_values(self.ids_by_recipient.get(recipient))
    }

    /// Returns `(created, expires)` timestamps for a message envelope.
    pub fn envelope_timestamps(
        &self,
        message_id: &str,
    ) -> Result<(&str, &str), MessageLifecycleError> {
        let record = self.require_record(message_id)?;
        Ok((&record.created, &record.expires))
    }

    /// Returns the transition history for a message.
    pub fn history(&self, message_id: &str) -> Result<&[MessageStatus], MessageLifecycleError> {
        Ok(&self.require_record(message_id)?.history)
    }

    /// Returns `(sender, recipients)` for a message.
    pub fn participants(
        &self,
        message_id: &str,
    ) -> Result<(&str, &[String]), MessageLifecycleError> {
        let record = self.require_record(message_id)?;
        Ok((&record.sender, &record.recipients))
    }
}
