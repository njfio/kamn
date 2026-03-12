use crate::data_layer_m9_realtime_delivery::{
    DataLayerM9PresenceQuery, DataLayerM9PresenceRecord, DataLayerM9RealtimeDeliveryError,
    DataLayerM9RealtimeDeliveryRegistry, DATA_LAYER_M9_INVALID_REQUESTER_AGENT_DID_REASON_CODE,
    DATA_LAYER_M9_INVALID_TARGET_AGENT_DID_REASON_CODE,
    DATA_LAYER_M9_PRESENCE_VISIBILITY_DENIED_REASON_CODE,
};
use crate::data_layer_m9_realtime_delivery::validation::{
    authorize_owner_scope, normalize_pair, parse_agent_did,
};

impl DataLayerM9RealtimeDeliveryRegistry {
    /// Queries target presence with scoped visibility controls.
    pub fn query_presence(
        &self,
        query: DataLayerM9PresenceQuery,
    ) -> Result<Option<DataLayerM9PresenceRecord>, DataLayerM9RealtimeDeliveryError> {
        authorize_owner_scope(query.requester_owner_did.as_str(), query.owner_did.as_str())?;
        let requester_agent_did = parse_agent_did(
            query.requester_agent_did.as_str(),
            "requester_agent_did",
            DATA_LAYER_M9_INVALID_REQUESTER_AGENT_DID_REASON_CODE,
        )?;
        let target_agent_did = parse_agent_did(
            query.target_agent_did.as_str(),
            "target_agent_did",
            DATA_LAYER_M9_INVALID_TARGET_AGENT_DID_REASON_CODE,
        )?;

        let has_visibility = if requester_agent_did.as_str() == target_agent_did.as_str() {
            true
        } else {
            let pair = normalize_pair(requester_agent_did.as_str(), target_agent_did.as_str());
            self.interaction_pairs.contains(&pair) || self.shared_escrow_pairs.contains(&pair)
        };

        if !has_visibility {
            return Err(DataLayerM9RealtimeDeliveryError::PresenceVisibilityDenied {
                reason_code: DATA_LAYER_M9_PRESENCE_VISIBILITY_DENIED_REASON_CODE,
            });
        }

        Ok(self.presence_by_agent.get(target_agent_did.as_str()).cloned())
    }
}
