use crate::data_layer_m9_realtime_delivery::dispatch::outcome::queue_escalation;
use crate::data_layer_m9_realtime_delivery::validation::{
    authorize_owner_scope, parse_agent_did, validate_non_empty,
};
use crate::{
    data_layer_m9_realtime_delivery::{
        DataLayerM9DispatchAckStatus, DataLayerM9DispatchOutcome, DataLayerM9DispatchRequest,
        DataLayerM9RealtimeDeliveryError, DataLayerM9RecipientQueueState,
        DATA_LAYER_M9_ACK_DELIVERED_REASON_CODE, DATA_LAYER_M9_ACK_QUEUED_QUEUE_FULL_REASON_CODE,
        DATA_LAYER_M9_ACK_QUEUED_REASON_CODE,
        DATA_LAYER_M9_INVALID_RECIPIENT_AGENT_DID_REASON_CODE,
        DATA_LAYER_M9_INVALID_SENDER_AGENT_DID_REASON_CODE,
        DATA_LAYER_M9_MAX_PENDING_PER_AGENT_MESSAGES,
    },
    AgentDid,
};

pub(super) fn validate_dispatch_request(
    request: &DataLayerM9DispatchRequest,
) -> Result<AgentDid, DataLayerM9RealtimeDeliveryError> {
    authorize_owner_scope(
        request.requester_owner_did.as_str(),
        request.owner_did.as_str(),
    )?;
    parse_agent_did(
        request.sender_agent_did.as_str(),
        "sender_agent_did",
        DATA_LAYER_M9_INVALID_SENDER_AGENT_DID_REASON_CODE,
    )?;
    let recipient_agent_did = parse_agent_did(
        request.recipient_agent_did.as_str(),
        "recipient_agent_did",
        DATA_LAYER_M9_INVALID_RECIPIENT_AGENT_DID_REASON_CODE,
    )?;
    validate_non_empty(request.message_id.as_str(), "message_id")?;
    if request.dispatched_at_epoch_seconds == 0 {
        return Err(DataLayerM9RealtimeDeliveryError::EmptyField(
            "dispatched_at_epoch_seconds",
        ));
    }
    Ok(recipient_agent_did)
}

pub(super) fn ensure_message_id_is_unique(
    queue_state: &DataLayerM9RecipientQueueState,
    message_id: &str,
) -> Result<(), DataLayerM9RealtimeDeliveryError> {
    if queue_state
        .pending_message_ids
        .iter()
        .any(|value| value == message_id)
        || queue_state
            .deferred_message_ids
            .iter()
            .any(|value| value == message_id)
    {
        return Err(DataLayerM9RealtimeDeliveryError::DuplicateMessageId(
            message_id.to_owned(),
        ));
    }
    Ok(())
}

pub(super) fn delivered_outcome(
    message_id: String,
    deferred_count: usize,
) -> DataLayerM9DispatchOutcome {
    DataLayerM9DispatchOutcome {
        message_id,
        ack_status: DataLayerM9DispatchAckStatus::Delivered,
        pending_queue_depth: 0,
        deferred_count,
        reason_code: DATA_LAYER_M9_ACK_DELIVERED_REASON_CODE,
        backpressure_warning_event: false,
        escrow_timeout_extension_recommended: false,
    }
}

pub(super) fn queue_pending_outcome(
    queue_state: &mut DataLayerM9RecipientQueueState,
    request: &DataLayerM9DispatchRequest,
) -> DataLayerM9DispatchOutcome {
    queue_state
        .pending_message_ids
        .push(request.message_id.clone());
    if queue_reached_cap(queue_state) && queue_state.first_full_at_epoch_seconds.is_none() {
        queue_state.first_full_at_epoch_seconds = Some(request.dispatched_at_epoch_seconds);
    }
    let (warning, extension) = queue_escalation(
        queue_state.first_full_at_epoch_seconds,
        request.dispatched_at_epoch_seconds,
    );
    DataLayerM9DispatchOutcome {
        message_id: request.message_id.clone(),
        ack_status: DataLayerM9DispatchAckStatus::Queued,
        pending_queue_depth: queue_state.pending_message_ids.len(),
        deferred_count: queue_state.deferred_message_ids.len(),
        reason_code: DATA_LAYER_M9_ACK_QUEUED_REASON_CODE,
        backpressure_warning_event: warning,
        escrow_timeout_extension_recommended: extension,
    }
}

pub(super) fn queue_deferred_outcome(
    queue_state: &mut DataLayerM9RecipientQueueState,
    request: &DataLayerM9DispatchRequest,
) -> DataLayerM9DispatchOutcome {
    if queue_state.first_full_at_epoch_seconds.is_none() {
        queue_state.first_full_at_epoch_seconds = Some(request.dispatched_at_epoch_seconds);
    }
    queue_state
        .deferred_message_ids
        .push(request.message_id.clone());
    let (warning, extension) = queue_escalation(
        queue_state.first_full_at_epoch_seconds,
        request.dispatched_at_epoch_seconds,
    );
    DataLayerM9DispatchOutcome {
        message_id: request.message_id.clone(),
        ack_status: DataLayerM9DispatchAckStatus::Queued,
        pending_queue_depth: DATA_LAYER_M9_MAX_PENDING_PER_AGENT_MESSAGES,
        deferred_count: queue_state.deferred_message_ids.len(),
        reason_code: DATA_LAYER_M9_ACK_QUEUED_QUEUE_FULL_REASON_CODE,
        backpressure_warning_event: warning,
        escrow_timeout_extension_recommended: extension,
    }
}

fn queue_reached_cap(queue_state: &DataLayerM9RecipientQueueState) -> bool {
    queue_state.pending_message_ids.len() == DATA_LAYER_M9_MAX_PENDING_PER_AGENT_MESSAGES
}
