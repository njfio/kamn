use crate::{DeterministicBackpressureController, RuntimeBackpressureInput, RuntimeBackpressurePolicy};
use crate::data_layer_m9_realtime_delivery::{
    DataLayerM9RealtimeDeliveryError, DataLayerM9RealtimeDeliveryRegistry,
    DataLayerM9RuntimeBackpressureProjection, DataLayerM9RuntimeBackpressureProjectionRequest,
    DATA_LAYER_M9_INVALID_RECIPIENT_AGENT_DID_REASON_CODE,
};
use crate::data_layer_m9_realtime_delivery::validation::{
    authorize_owner_scope, map_runtime_backpressure_evaluation_error_to_m9_projection_error,
    map_runtime_backpressure_input_error_to_m9_projection_error,
    map_runtime_backpressure_policy_error_to_m9_projection_error, parse_agent_did,
};

impl DataLayerM9RealtimeDeliveryRegistry {
    /// Projects one recipient queue through runtime backpressure contracts.
    pub fn project_runtime_backpressure_for_recipient(
        &self,
        request: DataLayerM9RuntimeBackpressureProjectionRequest,
    ) -> Result<DataLayerM9RuntimeBackpressureProjection, DataLayerM9RealtimeDeliveryError> {
        let recipient_agent_did = validate_projection_request(&request)?;
        let (pending_queue_depth, deferred_count) =
            queue_depths(self, recipient_agent_did.as_str());
        let runtime_decision =
            evaluate_runtime_decision(&request, recipient_agent_did.as_str(), pending_queue_depth)?;
        let reason_code = runtime_decision.reason_code();

        Ok(DataLayerM9RuntimeBackpressureProjection {
            recipient_agent_did: recipient_agent_did.as_str().to_owned(),
            pending_queue_depth,
            deferred_count,
            runtime_decision,
            reason_code,
        })
    }
}

fn validate_projection_request(
    request: &DataLayerM9RuntimeBackpressureProjectionRequest,
) -> Result<crate::AgentDid, DataLayerM9RealtimeDeliveryError> {
    authorize_owner_scope(request.requester_owner_did.as_str(), request.owner_did.as_str())?;
    parse_agent_did(
        request.recipient_agent_did.as_str(),
        "recipient_agent_did",
        DATA_LAYER_M9_INVALID_RECIPIENT_AGENT_DID_REASON_CODE,
    )
}

fn queue_depths(
    registry: &DataLayerM9RealtimeDeliveryRegistry,
    recipient_agent_did: &str,
) -> (usize, usize) {
    let queue_state = registry.queue_by_recipient.get(recipient_agent_did);
    let pending_queue_depth = queue_state
        .map(|state| state.pending_message_ids.len())
        .unwrap_or_default();
    let deferred_count = queue_state
        .map(|state| state.deferred_message_ids.len())
        .unwrap_or_default();
    (pending_queue_depth, deferred_count)
}

fn evaluate_runtime_decision(
    request: &DataLayerM9RuntimeBackpressureProjectionRequest,
    recipient_agent_did: &str,
    pending_queue_depth: usize,
) -> Result<crate::RuntimeBackpressureDecision, DataLayerM9RealtimeDeliveryError> {
    let policy = RuntimeBackpressurePolicy::new(
        request.slow_threshold_per_mille,
        request.reject_threshold_per_mille,
        request.purge_disconnected_with_pending_queue,
    )
    .map_err(map_runtime_backpressure_policy_error_to_m9_projection_error)?;
    let input = RuntimeBackpressureInput::new(
        recipient_agent_did,
        pending_queue_depth,
        request.queue_capacity,
        request.lifecycle_state,
    )
    .map_err(map_runtime_backpressure_input_error_to_m9_projection_error)?;
    DeterministicBackpressureController::new(policy)
        .evaluate(input)
        .map_err(map_runtime_backpressure_evaluation_error_to_m9_projection_error)
}
