use super::super::*;

impl ServiceApiMessageStore {
    pub(crate) fn get_message_for_requester(
        &mut self,
        message_id: &str,
        requester_did: Option<&str>,
    ) -> Result<Option<ServiceApiMessageGetBody>, String> {
        self.refresh_from_disk()?;
        let Some(record) = self.snapshot.messages.get_mut(message_id) else {
            return Ok(None);
        };
        let should_mark_delivered = record.status.as_str() == "relayed"
            && requester_did.is_some()
            && record.recipient_did.as_deref() == requester_did;
        let payload = if should_mark_delivered {
            record.status = "delivered".to_owned();
            ServiceApiMessageGetBody {
                message_id: record.message_id.clone(),
                status: record.status.clone(),
                sender_did: record.sender_did.clone(),
                recipient_did: record.recipient_did.clone(),
                body: record.body.clone(),
            }
        } else {
            ServiceApiMessageGetBody {
                message_id: record.message_id.clone(),
                status: record.status.clone(),
                sender_did: record.sender_did.clone(),
                recipient_did: record.recipient_did.clone(),
                body: record.body.clone(),
            }
        };
        let _ = record;
        if should_mark_delivered {
            self.persist()?;
        }
        Ok(Some(payload))
    }

    pub(crate) fn list_channel_messages(
        &mut self,
        channel_id: &str,
    ) -> Result<ServiceApiChannelMessagesBody, String> {
        self.refresh_from_disk()?;
        Ok(ServiceApiChannelMessagesBody {
            channel_id: channel_id.to_owned(),
            messages: self
                .snapshot
                .channel_messages
                .get(channel_id)
                .cloned()
                .unwrap_or_default(),
        })
    }

    pub(crate) fn relay_progress_counts(
        &mut self,
    ) -> Result<ServiceApiRelayProgressCounts, String> {
        self.refresh_from_disk()?;
        let mut created_message_count = 0_u64;
        let mut relayed_message_count = 0_u64;
        let mut delivered_message_count = 0_u64;
        for record in self.snapshot.messages.values() {
            match record.status.as_str() {
                "created" => {
                    created_message_count = created_message_count.saturating_add(1);
                }
                "relayed" => {
                    relayed_message_count = relayed_message_count.saturating_add(1);
                }
                "delivered" => {
                    delivered_message_count = delivered_message_count.saturating_add(1);
                }
                _ => {}
            }
        }
        Ok(ServiceApiRelayProgressCounts {
            created_message_count,
            relayed_message_count,
            delivered_message_count,
        })
    }

}

pub(crate) fn recipient_mailbox_channel_id(recipient_did: &str) -> String {
    format!("recipient:{recipient_did}")
}
