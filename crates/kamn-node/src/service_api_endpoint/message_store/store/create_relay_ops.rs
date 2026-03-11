use super::super::*;
use super::recipient_mailbox_channel_id;

mod create_support;
mod relay_support;

use create_support::{
    insert_created_message, message_create_body, next_channel_id, next_message_id,
};
use relay_support::{
    ensure_relay_mailbox_membership, relay_message_body, upsert_relay_message_record,
    validate_relay_request,
};

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
        let message_id = next_message_id(self, payload);
        let data_layer_runtime_evidence =
            build_created_message_runtime_evidence(message_id.as_str(), payload, sender_did, recipient_did)?;
        persist_created_message(
            self,
            message_id.as_str(),
            payload,
            channel_id,
            sender_did,
            recipient_did,
            data_layer_runtime_evidence,
        )?;
        self.persist()?;
        persist_message_created_audit_export(self, message_id.as_str(), sender_did)?;
        Ok(message_create_body(message_id, runtime_mode))
    }

    pub(crate) fn create_channel(
        &mut self,
        payload: &str,
    ) -> Result<ServiceApiChannelCreateBody, String> {
        self.refresh_from_disk()?;
        let channel_id = next_channel_id(self, payload);
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
        let request = validate_relay_request(message_id, sender_did, recipient_did)?;
        let mut mutated = upsert_relay_message_record(self, &request, body)?;
        mutated |= ensure_relay_mailbox_membership(self, &request);
        if mutated {
            self.persist()?;
            persist_message_relayed_audit_export(
                self,
                request.message_id,
                request.sender_did,
            )?;
        }
        Ok(relay_message_body(self, request.message_id))
    }
}

fn build_created_message_runtime_evidence(
    message_id: &str,
    payload: &str,
    sender_did: Option<&str>,
    recipient_did: Option<&str>,
) -> Result<ServiceApiDataLayerRuntimeEvidenceRecord, String> {
    build_data_layer_runtime_evidence(message_id, payload, sender_did, recipient_did)
}

fn persist_created_message(
    store: &mut ServiceApiMessageStore,
    message_id: &str,
    payload: &str,
    channel_id: Option<&str>,
    sender_did: Option<&str>,
    recipient_did: Option<&str>,
    data_layer_runtime_evidence: ServiceApiDataLayerRuntimeEvidenceRecord,
) -> Result<(), String> {
    insert_created_message(
        store,
        message_id,
        payload,
        channel_id,
        sender_did,
        recipient_did,
        data_layer_runtime_evidence,
    );
    Ok(())
}

fn persist_message_created_audit_export(
    store: &ServiceApiMessageStore,
    message_id: &str,
    sender_did: Option<&str>,
) -> Result<(), String> {
    let event = service_api_message_created_audit_event(message_id, sender_did);
    persist_service_api_audit_export_event(store.audit_export_file.as_deref(), event)
}

fn persist_message_relayed_audit_export(
    store: &ServiceApiMessageStore,
    message_id: &str,
    sender_did: Option<&str>,
) -> Result<(), String> {
    let event = service_api_message_relayed_audit_event(message_id, sender_did);
    persist_service_api_audit_export_event(store.audit_export_file.as_deref(), event)
}
