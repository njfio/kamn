use super::validation::build_message_record;
use super::MessageLifecycleStore;
use crate::message_lifecycle::{MessageLifecycleError, MessageStatus};

impl MessageLifecycleStore {
    pub(super) fn ensure_message_id_is_new(
        &self,
        message_id: &str,
    ) -> Result<(), MessageLifecycleError> {
        if self.records.contains_key(message_id) {
            return Err(MessageLifecycleError::DuplicateMessageId(
                message_id.to_owned(),
            ));
        }
        Ok(())
    }

    pub(super) fn insert_registered_message(
        &mut self,
        message_id: &str,
        sender: &str,
        recipients: Vec<String>,
        created: &str,
        expires: &str,
    ) {
        let id = message_id.to_owned();
        let sender_value = sender.to_owned();
        let record = build_message_record(
            sender_value.clone(),
            recipients.clone(),
            created.to_owned(),
            expires.to_owned(),
            MessageStatus::Created,
            vec![MessageStatus::Created],
        );
        self.records.insert(id.clone(), record);
        self.index_new_message(&id, sender_value, recipients);
    }

    fn index_new_message(&mut self, id: &str, sender: String, recipients: Vec<String>) {
        self.ids_by_status
            .entry(MessageStatus::Created)
            .or_default()
            .insert(id.to_owned());
        self.ids_by_sender
            .entry(sender)
            .or_default()
            .insert(id.to_owned());
        for recipient in recipients {
            self.ids_by_recipient
                .entry(recipient)
                .or_default()
                .insert(id.to_owned());
        }
    }

    pub(super) fn update_record_status(
        &mut self,
        message_id: &str,
        to: MessageStatus,
    ) -> Result<(), MessageLifecycleError> {
        let record = self
            .records
            .get_mut(message_id)
            .ok_or_else(|| MessageLifecycleError::NotFound(message_id.to_owned()))?;
        record.status = to;
        record.history.push(to);
        Ok(())
    }

    pub(super) fn reindex_status(
        &mut self,
        message_id: &str,
        from: MessageStatus,
        to: MessageStatus,
    ) {
        if let Some(ids) = self.ids_by_status.get_mut(&from) {
            ids.remove(message_id);
        }
        self.ids_by_status
            .entry(to)
            .or_default()
            .insert(message_id.to_owned());
    }
}
