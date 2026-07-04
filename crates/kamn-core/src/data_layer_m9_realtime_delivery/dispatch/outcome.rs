use crate::data_layer_m9_realtime_delivery::dispatch::outcome_support::{
    delivered_outcome, ensure_message_id_is_unique, queue_deferred_outcome, queue_pending_outcome,
    validate_dispatch_request,
};
use crate::{
    data_layer_m9_realtime_delivery::{
        DataLayerM9ChannelDispatchAuthorizationRequest, DataLayerM9DispatchOutcome,
        DataLayerM9DispatchRequest, DataLayerM9RealtimeDeliveryError,
        DataLayerM9RealtimeDeliveryRegistry,
        DATA_LAYER_M9_ANTI_SPAM_DUPLICATE_MESSAGE_ID_REASON_CODE,
        DATA_LAYER_M9_ANTI_SPAM_INSUFFICIENT_DEPOSIT_REASON_CODE,
        DATA_LAYER_M9_ANTI_SPAM_RATE_LIMITED_REASON_CODE,
        DATA_LAYER_M9_ANTI_SPAM_SUSPENDED_REASON_CODE,
        DATA_LAYER_M9_MAX_PENDING_PER_AGENT_MESSAGES,
    },
    AntiSpamDecision, AntiSpamEngine, AntiSpamRejection, ChannelStore,
};

impl DataLayerM9RealtimeDeliveryRegistry {
    /// Dispatches one message after channel-membership and anti-spam admission controls.
    pub fn dispatch_message_with_controls(
        &mut self,
        channel_store: &ChannelStore,
        anti_spam: &mut AntiSpamEngine,
        channel_id: &str,
        request: DataLayerM9DispatchRequest,
    ) -> Result<DataLayerM9DispatchOutcome, DataLayerM9RealtimeDeliveryError> {
        self.authorize_channel_dispatch(
            channel_store,
            authorization_request(channel_id, &request),
        )?;
        match evaluate_anti_spam(anti_spam, &request)? {
            AntiSpamDecision::Accepted => self.dispatch_message(request),
            AntiSpamDecision::Rejected(rejection) => {
                Err(DataLayerM9RealtimeDeliveryError::AntiSpamAdmissionDenied {
                    reason_code: anti_spam_rejection_reason_code(&rejection),
                })
            }
        }
    }

    /// Dispatches one message and computes deterministic ACK outcome.
    pub fn dispatch_message(
        &mut self,
        request: DataLayerM9DispatchRequest,
    ) -> Result<DataLayerM9DispatchOutcome, DataLayerM9RealtimeDeliveryError> {
        let recipient_agent_did = validate_dispatch_request(&request)?;
        let queue_state = self
            .queue_by_recipient
            .entry(recipient_agent_did.as_str().to_owned())
            .or_default();
        ensure_message_id_is_unique(queue_state, request.message_id.as_str())?;

        if delivery_is_immediate(
            &self.presence_by_agent,
            recipient_agent_did.as_str(),
            queue_state,
        ) {
            return Ok(delivered_outcome(
                request.message_id,
                queue_state.deferred_message_ids.len(),
            ));
        }
        Ok(queue_outcome(queue_state, &request))
    }
}

fn delivery_is_immediate(
    presence_by_agent: &std::collections::BTreeMap<
        String,
        crate::data_layer_m9_realtime_delivery::DataLayerM9PresenceRecord,
    >,
    recipient_agent_did: &str,
    queue_state: &crate::data_layer_m9_realtime_delivery::DataLayerM9RecipientQueueState,
) -> bool {
    presence_by_agent.contains_key(recipient_agent_did)
        && queue_state.pending_message_ids.is_empty()
}

pub(crate) fn queue_escalation(first_full_at: Option<u64>, now_epoch_seconds: u64) -> (bool, bool) {
    let Some(first_full_at_epoch_seconds) = first_full_at else {
        return (false, false);
    };
    let full_duration_seconds = now_epoch_seconds.saturating_sub(first_full_at_epoch_seconds);
    let warning = full_duration_seconds
        > crate::data_layer_m9_realtime_delivery::DATA_LAYER_M9_BACKPRESSURE_WARNING_AFTER_SECONDS;
    let extension = full_duration_seconds > crate::data_layer_m9_realtime_delivery::DATA_LAYER_M9_BACKPRESSURE_ESCROW_EXTENSION_AFTER_SECONDS;
    (warning, extension)
}

fn authorization_request(
    channel_id: &str,
    request: &DataLayerM9DispatchRequest,
) -> DataLayerM9ChannelDispatchAuthorizationRequest {
    DataLayerM9ChannelDispatchAuthorizationRequest {
        requester_owner_did: request.requester_owner_did.clone(),
        owner_did: request.owner_did.clone(),
        channel_id: channel_id.to_owned(),
        sender_agent_did: request.sender_agent_did.clone(),
        recipient_agent_did: request.recipient_agent_did.clone(),
    }
}

fn evaluate_anti_spam(
    anti_spam: &mut AntiSpamEngine,
    request: &DataLayerM9DispatchRequest,
) -> Result<AntiSpamDecision, DataLayerM9RealtimeDeliveryError> {
    anti_spam
        .evaluate(
            request.sender_agent_did.as_str(),
            request.message_id.as_str(),
            request.dispatched_at_epoch_seconds,
        )
        .map_err(
            |error| DataLayerM9RealtimeDeliveryError::AntiSpamEngineError {
                detail: error.to_string(),
            },
        )
}

fn queue_outcome(
    queue_state: &mut crate::data_layer_m9_realtime_delivery::DataLayerM9RecipientQueueState,
    request: &DataLayerM9DispatchRequest,
) -> DataLayerM9DispatchOutcome {
    if queue_state.pending_message_ids.len() < DATA_LAYER_M9_MAX_PENDING_PER_AGENT_MESSAGES {
        return queue_pending_outcome(queue_state, request);
    }
    queue_deferred_outcome(queue_state, request)
}

fn anti_spam_rejection_reason_code(rejection: &AntiSpamRejection) -> &'static str {
    match rejection {
        AntiSpamRejection::InsufficientDeposit { .. } => {
            DATA_LAYER_M9_ANTI_SPAM_INSUFFICIENT_DEPOSIT_REASON_CODE
        }
        AntiSpamRejection::RateLimitExceeded { .. } => {
            DATA_LAYER_M9_ANTI_SPAM_RATE_LIMITED_REASON_CODE
        }
        AntiSpamRejection::SenderSuspended { .. } => DATA_LAYER_M9_ANTI_SPAM_SUSPENDED_REASON_CODE,
        AntiSpamRejection::DuplicateMessageId(_) => {
            DATA_LAYER_M9_ANTI_SPAM_DUPLICATE_MESSAGE_ID_REASON_CODE
        }
    }
}
