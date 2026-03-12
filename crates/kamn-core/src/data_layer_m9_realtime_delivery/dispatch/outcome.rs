use crate::{
    data_layer_m9_realtime_delivery::{
        DataLayerM9ChannelDispatchAuthorizationRequest, DataLayerM9DispatchAckStatus,
        DataLayerM9DispatchOutcome, DataLayerM9DispatchRequest,
        DataLayerM9RealtimeDeliveryError, DataLayerM9RealtimeDeliveryRegistry,
        DATA_LAYER_M9_ACK_DELIVERED_REASON_CODE, DATA_LAYER_M9_ACK_QUEUED_QUEUE_FULL_REASON_CODE,
        DATA_LAYER_M9_ACK_QUEUED_REASON_CODE,
        DATA_LAYER_M9_ANTI_SPAM_DUPLICATE_MESSAGE_ID_REASON_CODE,
        DATA_LAYER_M9_ANTI_SPAM_INSUFFICIENT_DEPOSIT_REASON_CODE,
        DATA_LAYER_M9_ANTI_SPAM_RATE_LIMITED_REASON_CODE,
        DATA_LAYER_M9_ANTI_SPAM_SUSPENDED_REASON_CODE,
        DATA_LAYER_M9_INVALID_RECIPIENT_AGENT_DID_REASON_CODE,
        DATA_LAYER_M9_INVALID_SENDER_AGENT_DID_REASON_CODE,
        DATA_LAYER_M9_MAX_PENDING_PER_AGENT_MESSAGES,
    },
    AntiSpamDecision, AntiSpamEngine, AntiSpamRejection, ChannelStore,
};
use crate::data_layer_m9_realtime_delivery::validation::{authorize_owner_scope, parse_agent_did, validate_non_empty};

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
            DataLayerM9ChannelDispatchAuthorizationRequest {
                requester_owner_did: request.requester_owner_did.clone(),
                owner_did: request.owner_did.clone(),
                channel_id: channel_id.to_owned(),
                sender_agent_did: request.sender_agent_did.clone(),
                recipient_agent_did: request.recipient_agent_did.clone(),
            },
        )?;

        let anti_spam_decision = anti_spam
            .evaluate(
                request.sender_agent_did.as_str(),
                request.message_id.as_str(),
                request.dispatched_at_epoch_seconds,
            )
            .map_err(|error| DataLayerM9RealtimeDeliveryError::AntiSpamEngineError {
                detail: error.to_string(),
            })?;
        match anti_spam_decision {
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
        authorize_owner_scope(request.requester_owner_did.as_str(), request.owner_did.as_str())?;
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

        let queue_state = self
            .queue_by_recipient
            .entry(recipient_agent_did.as_str().to_owned())
            .or_default();
        if queue_state.pending_message_ids.contains(&request.message_id)
            || queue_state.deferred_message_ids.contains(&request.message_id)
        {
            return Err(DataLayerM9RealtimeDeliveryError::DuplicateMessageId(
                request.message_id,
            ));
        }

        if delivery_is_immediate(&self.presence_by_agent, recipient_agent_did.as_str(), queue_state) {
            return Ok(DataLayerM9DispatchOutcome {
                message_id: request.message_id,
                ack_status: DataLayerM9DispatchAckStatus::Delivered,
                pending_queue_depth: 0,
                deferred_count: queue_state.deferred_message_ids.len(),
                reason_code: DATA_LAYER_M9_ACK_DELIVERED_REASON_CODE,
                backpressure_warning_event: false,
                escrow_timeout_extension_recommended: false,
            });
        }

        if queue_state.pending_message_ids.len() < DATA_LAYER_M9_MAX_PENDING_PER_AGENT_MESSAGES {
            queue_state.pending_message_ids.push(request.message_id.clone());
            if queue_state.pending_message_ids.len() == DATA_LAYER_M9_MAX_PENDING_PER_AGENT_MESSAGES
                && queue_state.first_full_at_epoch_seconds.is_none()
            {
                queue_state.first_full_at_epoch_seconds = Some(request.dispatched_at_epoch_seconds);
            }
            let (warning, extension) = queue_escalation(
                queue_state.first_full_at_epoch_seconds,
                request.dispatched_at_epoch_seconds,
            );
            return Ok(DataLayerM9DispatchOutcome {
                message_id: request.message_id,
                ack_status: DataLayerM9DispatchAckStatus::Queued,
                pending_queue_depth: queue_state.pending_message_ids.len(),
                deferred_count: queue_state.deferred_message_ids.len(),
                reason_code: DATA_LAYER_M9_ACK_QUEUED_REASON_CODE,
                backpressure_warning_event: warning,
                escrow_timeout_extension_recommended: extension,
            });
        }

        if queue_state.first_full_at_epoch_seconds.is_none() {
            queue_state.first_full_at_epoch_seconds = Some(request.dispatched_at_epoch_seconds);
        }
        queue_state.deferred_message_ids.push(request.message_id.clone());
        let (warning, extension) = queue_escalation(
            queue_state.first_full_at_epoch_seconds,
            request.dispatched_at_epoch_seconds,
        );
        Ok(DataLayerM9DispatchOutcome {
            message_id: request.message_id,
            ack_status: DataLayerM9DispatchAckStatus::Queued,
            pending_queue_depth: DATA_LAYER_M9_MAX_PENDING_PER_AGENT_MESSAGES,
            deferred_count: queue_state.deferred_message_ids.len(),
            reason_code: DATA_LAYER_M9_ACK_QUEUED_QUEUE_FULL_REASON_CODE,
            backpressure_warning_event: warning,
            escrow_timeout_extension_recommended: extension,
        })
    }
}

fn delivery_is_immediate(
    presence_by_agent: &std::collections::BTreeMap<String, crate::data_layer_m9_realtime_delivery::DataLayerM9PresenceRecord>,
    recipient_agent_did: &str,
    queue_state: &crate::data_layer_m9_realtime_delivery::DataLayerM9RecipientQueueState,
) -> bool {
    presence_by_agent.contains_key(recipient_agent_did) && queue_state.pending_message_ids.is_empty()
}

pub(crate) fn queue_escalation(first_full_at: Option<u64>, now_epoch_seconds: u64) -> (bool, bool) {
    let Some(first_full_at_epoch_seconds) = first_full_at else {
        return (false, false);
    };
    let full_duration_seconds = now_epoch_seconds.saturating_sub(first_full_at_epoch_seconds);
    let warning = full_duration_seconds > crate::data_layer_m9_realtime_delivery::DATA_LAYER_M9_BACKPRESSURE_WARNING_AFTER_SECONDS;
    let extension = full_duration_seconds > crate::data_layer_m9_realtime_delivery::DATA_LAYER_M9_BACKPRESSURE_ESCROW_EXTENSION_AFTER_SECONDS;
    (warning, extension)
}

fn anti_spam_rejection_reason_code(rejection: &AntiSpamRejection) -> &'static str {
    match rejection {
        AntiSpamRejection::InsufficientDeposit { .. } => {
            DATA_LAYER_M9_ANTI_SPAM_INSUFFICIENT_DEPOSIT_REASON_CODE
        }
        AntiSpamRejection::RateLimitExceeded { .. } => DATA_LAYER_M9_ANTI_SPAM_RATE_LIMITED_REASON_CODE,
        AntiSpamRejection::SenderSuspended { .. } => DATA_LAYER_M9_ANTI_SPAM_SUSPENDED_REASON_CODE,
        AntiSpamRejection::DuplicateMessageId(_) => {
            DATA_LAYER_M9_ANTI_SPAM_DUPLICATE_MESSAGE_ID_REASON_CODE
        }
    }
}
