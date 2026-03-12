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
        authorize_owner_scope(request.requester_owner_did.as_str(), request.owner_did.as_str())?;
        let recipient_agent_did = parse_agent_did(
            request.recipient_agent_did.as_str(),
            "recipient_agent_did",
            DATA_LAYER_M9_INVALID_RECIPIENT_AGENT_DID_REASON_CODE,
        )?;

        let queue_state = self.queue_by_recipient.get(recipient_agent_did.as_str());
        let pending_queue_depth = queue_state
            .map(|state| state.pending_message_ids.len())
            .unwrap_or_default();
        let deferred_count = queue_state
            .map(|state| state.deferred_message_ids.len())
            .unwrap_or_default();

        let policy = RuntimeBackpressurePolicy::new(
            request.slow_threshold_per_mille,
            request.reject_threshold_per_mille,
            request.purge_disconnected_with_pending_queue,
        )
        .map_err(map_runtime_backpressure_policy_error_to_m9_projection_error)?;

        let input = RuntimeBackpressureInput::new(
            recipient_agent_did.as_str(),
            pending_queue_depth,
            request.queue_capacity,
            request.lifecycle_state,
        )
        .map_err(map_runtime_backpressure_input_error_to_m9_projection_error)?;

        let runtime_decision = DeterministicBackpressureController::new(policy)
            .evaluate(input)
            .map_err(map_runtime_backpressure_evaluation_error_to_m9_projection_error)?;
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
