use super::*;

impl MessageLifecycleStore {
    /// Returns the current status for a message id.
    pub fn status(&self, message_id: &str) -> Result<MessageStatus, MessageLifecycleError> {
        self.records
            .get(message_id)
            .map(|record| record.status)
            .ok_or_else(|| MessageLifecycleError::NotFound(message_id.to_owned()))
    }

    /// Returns message ids in the provided lifecycle status.
    pub fn ids_by_status(&self, status: MessageStatus) -> Vec<String> {
        self.ids_by_status
            .get(&status)
            .map(|ids| ids.iter().cloned().collect())
            .unwrap_or_default()
    }

    /// Returns message ids registered by the sender DID.
    pub fn ids_by_sender(&self, sender: &str) -> Vec<String> {
        self.ids_by_sender
            .get(sender)
            .map(|ids| ids.iter().cloned().collect())
            .unwrap_or_default()
    }

    /// Returns message ids that include the provided recipient DID.
    pub fn ids_by_recipient(&self, recipient: &str) -> Vec<String> {
        self.ids_by_recipient
            .get(recipient)
            .map(|ids| ids.iter().cloned().collect())
            .unwrap_or_default()
    }

    /// Returns `(created, expires)` timestamps for a message envelope.
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

    /// Returns the transition history for a message.
    pub fn history(&self, message_id: &str) -> Result<&[MessageStatus], MessageLifecycleError> {
        let record = self
            .records
            .get(message_id)
            .ok_or_else(|| MessageLifecycleError::NotFound(message_id.to_owned()))?;
        Ok(&record.history)
    }

    /// Returns `(sender, recipients)` for a message.
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
