use crate::data_layer_m9_realtime_delivery::{
    DataLayerM9PresenceRelationshipRequest, DataLayerM9RealtimeDeliveryError,
    DataLayerM9RealtimeDeliveryRegistry, DATA_LAYER_M9_INVALID_COUNTERPARTY_AGENT_DID_REASON_CODE,
    DATA_LAYER_M9_INVALID_REQUESTER_AGENT_DID_REASON_CODE,
};
use crate::data_layer_m9_realtime_delivery::validation::{
    authorize_owner_scope, normalize_pair, parse_agent_did,
};

impl DataLayerM9RealtimeDeliveryRegistry {
    /// Registers prior-interaction linkage for scoped presence visibility.
    pub fn record_interaction_link(
        &mut self,
        request: DataLayerM9PresenceRelationshipRequest,
    ) -> Result<(), DataLayerM9RealtimeDeliveryError> {
        authorize_owner_scope(request.requester_owner_did.as_str(), request.owner_did.as_str())?;
        let requester_agent_did = parse_agent_did(
            request.requester_agent_did.as_str(),
            "requester_agent_did",
            DATA_LAYER_M9_INVALID_REQUESTER_AGENT_DID_REASON_CODE,
        )?;
        let counterparty_agent_did = parse_agent_did(
            request.counterparty_agent_did.as_str(),
            "counterparty_agent_did",
            DATA_LAYER_M9_INVALID_COUNTERPARTY_AGENT_DID_REASON_CODE,
        )?;
        if requester_agent_did.as_str() == counterparty_agent_did.as_str() {
            return Err(DataLayerM9RealtimeDeliveryError::SameAgentRelationship);
        }
        self.interaction_pairs.insert(normalize_pair(
            requester_agent_did.as_str(),
            counterparty_agent_did.as_str(),
        ));
        Ok(())
    }

    /// Registers shared-escrow linkage for scoped presence visibility.
    pub fn record_shared_escrow_link(
        &mut self,
        request: DataLayerM9PresenceRelationshipRequest,
    ) -> Result<(), DataLayerM9RealtimeDeliveryError> {
        authorize_owner_scope(request.requester_owner_did.as_str(), request.owner_did.as_str())?;
        let requester_agent_did = parse_agent_did(
            request.requester_agent_did.as_str(),
            "requester_agent_did",
            DATA_LAYER_M9_INVALID_REQUESTER_AGENT_DID_REASON_CODE,
        )?;
        let counterparty_agent_did = parse_agent_did(
            request.counterparty_agent_did.as_str(),
            "counterparty_agent_did",
            DATA_LAYER_M9_INVALID_COUNTERPARTY_AGENT_DID_REASON_CODE,
        )?;
        if requester_agent_did.as_str() == counterparty_agent_did.as_str() {
            return Err(DataLayerM9RealtimeDeliveryError::SameAgentRelationship);
        }
        self.shared_escrow_pairs.insert(normalize_pair(
            requester_agent_did.as_str(),
            counterparty_agent_did.as_str(),
        ));
        Ok(())
    }
}
