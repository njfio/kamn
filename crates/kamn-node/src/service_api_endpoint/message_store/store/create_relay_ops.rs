use super::super::*;
use super::recipient_mailbox_channel_id;

impl ServiceApiMessageStore {
    pub(crate) fn create_message(
        &mut self,
        payload: &str,
        runtime_mode: &str,
        channel_id: Option<&str>,
        sender_did: Option<&str>,
        recipient_did: Option<&str>,
    ) -> Result<ServiceApiMessageCreateBody, String> {
        self.refresh_from_disk()?;
        let base = format!(
            "msg-local-{:016x}",
            deterministic_body_tag(payload.as_bytes())
        );
        let mut message_id = base.clone();
        let mut suffix = 1_u64;
        while self.snapshot.messages.contains_key(message_id.as_str()) {
            message_id = format!("{base}-{suffix}");
            suffix = suffix.saturating_add(1);
        }
        let data_layer_runtime_evidence = build_data_layer_runtime_evidence(
            message_id.as_str(),
            payload,
            sender_did,
            recipient_did,
        )?;

        self.snapshot.messages.insert(
            message_id.clone(),
            ServiceApiPersistedMessageRecord {
                message_id: message_id.clone(),
                status: "created".to_owned(),
                channel_id: channel_id.map(str::to_owned),
                sender_did: sender_did.map(str::to_owned),
                recipient_did: recipient_did.map(str::to_owned),
                body: Some(payload.to_owned()),
                data_layer_runtime_evidence: Some(data_layer_runtime_evidence),
            },
        );
        if let Some(channel_id) = channel_id {
            self.snapshot
                .channel_messages
                .entry(channel_id.to_owned())
                .or_default()
                .push(message_id.clone());
        }
        if let Some(recipient_did) = recipient_did {
            self.snapshot
                .channel_messages
                .entry(recipient_mailbox_channel_id(recipient_did))
                .or_default()
                .push(message_id.clone());
        }
        self.persist()?;
        Ok(ServiceApiMessageCreateBody {
            message_id,
            status: "created".to_owned(),
            runtime_mode: runtime_mode.to_owned(),
        })
    }

    pub(crate) fn create_channel(
        &mut self,
        payload: &str,
    ) -> Result<ServiceApiChannelCreateBody, String> {
        self.refresh_from_disk()?;
        let base = format!(
            "channel-local-{:016x}",
            deterministic_body_tag(payload.as_bytes())
        );
        let mut channel_id = base.clone();
        let mut suffix = 1_u64;
        while self
            .snapshot
            .channel_messages
            .contains_key(channel_id.as_str())
        {
            channel_id = format!("{base}-{suffix}");
            suffix = suffix.saturating_add(1);
        }
        self.snapshot
            .channel_messages
            .entry(channel_id.clone())
            .or_default();
        self.persist()?;
        Ok(ServiceApiChannelCreateBody {
            channel_id,
            status: "created".to_owned(),
        })
    }

    pub(crate) fn upsert_relayed_message(
        &mut self,
        message_id: &str,
        sender_did: Option<&str>,
        recipient_did: &str,
        body: &str,
    ) -> Result<ServiceApiMessageRelayBody, String> {
        self.refresh_from_disk()?;
        let normalized_message_id = message_id.trim();
        if normalized_message_id.is_empty() {
            return Err("relay message id must not be empty".to_owned());
        }
        let normalized_recipient_did = recipient_did.trim();
        if normalized_recipient_did.is_empty() {
            return Err("relay recipient did must not be empty".to_owned());
        }
        let normalized_sender_did = sender_did.map(str::trim).filter(|value| !value.is_empty());

        let mut mutated = false;
        if let Some(record) = self.snapshot.messages.get_mut(normalized_message_id) {
            if record.recipient_did.as_deref() != Some(normalized_recipient_did) {
                return Err(format!(
                    "relay recipient mismatch for {normalized_message_id}: expected={}, actual={normalized_recipient_did}",
                    record.recipient_did.as_deref().unwrap_or("none")
                ));
            }
            if record.body.as_deref() != Some(body) {
                return Err(format!(
                    "relay body mismatch for {normalized_message_id}: existing payload differs"
                ));
            }
            if let Some(sender) = normalized_sender_did {
                match record.sender_did.as_deref() {
                    Some(existing) if existing != sender => {
                        return Err(format!(
                            "relay sender mismatch for {normalized_message_id}: expected={existing}, actual={sender}"
                        ));
                    }
                    None => {
                        record.sender_did = Some(sender.to_owned());
                        mutated = true;
                    }
                    _ => {}
                }
            }
            if record.status.as_str() == "created" {
                record.status = "relayed".to_owned();
                mutated = true;
            }
        } else {
            self.snapshot.messages.insert(
                normalized_message_id.to_owned(),
                ServiceApiPersistedMessageRecord {
                    message_id: normalized_message_id.to_owned(),
                    status: "relayed".to_owned(),
                    channel_id: None,
                    sender_did: normalized_sender_did.map(str::to_owned),
                    recipient_did: Some(normalized_recipient_did.to_owned()),
                    body: Some(body.to_owned()),
                    data_layer_runtime_evidence: None,
                },
            );
            mutated = true;
        }

        let mailbox_channel_id = recipient_mailbox_channel_id(normalized_recipient_did);
        let mailbox = self
            .snapshot
            .channel_messages
            .entry(mailbox_channel_id)
            .or_default();
        if !mailbox
            .iter()
            .any(|candidate| candidate == normalized_message_id)
        {
            mailbox.push(normalized_message_id.to_owned());
            mutated = true;
        }

        if mutated {
            self.persist()?;
        }

        let status = self
            .snapshot
            .messages
            .get(normalized_message_id)
            .map(|record| record.status.clone())
            .unwrap_or_else(|| "relayed".to_owned());
        Ok(ServiceApiMessageRelayBody {
            message_id: normalized_message_id.to_owned(),
            status,
        })
    }

}
