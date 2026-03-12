use crate::data_layer_m9_realtime_delivery::{
    DataLayerM9RealtimeDeliveryError, DataLayerM9RealtimeDeliveryRegistry,
    DataLayerM9RecipientQueueSnapshot, DATA_LAYER_M9_INVALID_RECIPIENT_AGENT_DID_REASON_CODE,
};
use crate::data_layer_m9_realtime_delivery::validation::{authorize_owner_scope, parse_agent_did};

impl DataLayerM9RealtimeDeliveryRegistry {
    /// Snapshots one recipient queue preserving insertion ordering for pending/deferred IDs.
    pub fn snapshot_recipient_queue(
        &self,
        requester_owner_did: &str,
        owner_did: &str,
        recipient_agent_did: &str,
    ) -> Result<DataLayerM9RecipientQueueSnapshot, DataLayerM9RealtimeDeliveryError> {
        authorize_owner_scope(requester_owner_did, owner_did)?;
        let recipient_agent_did = validate_snapshot_recipient(recipient_agent_did)?;
        let (pending_message_ids, deferred_message_ids, first_full_at_epoch_seconds) =
            snapshot_parts(self, recipient_agent_did.as_str());

        Ok(DataLayerM9RecipientQueueSnapshot {
            recipient_agent_did: recipient_agent_did.as_str().to_owned(),
            pending_queue_depth: pending_message_ids.len(),
            deferred_count: deferred_message_ids.len(),
            pending_message_ids,
            deferred_message_ids,
            first_full_at_epoch_seconds,
        })
    }
}

fn validate_snapshot_recipient(
    recipient_agent_did: &str,
) -> Result<crate::AgentDid, DataLayerM9RealtimeDeliveryError> {
    parse_agent_did(
        recipient_agent_did,
        "recipient_agent_did",
        DATA_LAYER_M9_INVALID_RECIPIENT_AGENT_DID_REASON_CODE,
    )
}

fn snapshot_parts(
    registry: &DataLayerM9RealtimeDeliveryRegistry,
    recipient_agent_did: &str,
) -> (Vec<String>, Vec<String>, Option<u64>) {
    let queue_state = registry.queue_by_recipient.get(recipient_agent_did);
    let pending_message_ids = queue_state
        .map(|state| state.pending_message_ids.clone())
        .unwrap_or_default();
    let deferred_message_ids = queue_state
        .map(|state| state.deferred_message_ids.clone())
        .unwrap_or_default();
    let first_full_at_epoch_seconds = queue_state.and_then(|state| state.first_full_at_epoch_seconds);
    (
        pending_message_ids,
        deferred_message_ids,
        first_full_at_epoch_seconds,
    )
}
