use crate::data_layer_m9_realtime_delivery::{
    DataLayerM9PresenceConnectRequest, DataLayerM9PresenceRecord,
    DataLayerM9RealtimeDeliveryError, DataLayerM9RealtimeDeliveryRegistry,
    DATA_LAYER_M9_INVALID_AGENT_DID_REASON_CODE,
};

use crate::data_layer_m9_realtime_delivery::validation::{authorize_owner_scope, parse_agent_did, validate_non_empty};

impl DataLayerM9RealtimeDeliveryRegistry {
    /// Registers or refreshes active presence for one agent.
    pub fn connect_presence(
        &mut self,
        request: DataLayerM9PresenceConnectRequest,
    ) -> Result<DataLayerM9PresenceRecord, DataLayerM9RealtimeDeliveryError> {
        let parsed_agent_did = validate_presence_request(&request)?;
        let capabilities_active = normalize_capabilities(request.capabilities_active.clone())?;
        let record = build_presence_record(request, parsed_agent_did.as_str(), capabilities_active);
        self.presence_by_agent
            .insert(parsed_agent_did.as_str().to_owned(), record.clone());
        Ok(record)
    }
}

fn validate_presence_request(
    request: &DataLayerM9PresenceConnectRequest,
) -> Result<crate::AgentDid, DataLayerM9RealtimeDeliveryError> {
    authorize_owner_scope(request.requester_owner_did.as_str(), request.owner_did.as_str())?;
    let parsed_agent_did = parse_agent_did(
        request.agent_did.as_str(),
        "agent_did",
        DATA_LAYER_M9_INVALID_AGENT_DID_REASON_CODE,
    )?;
    validate_non_empty(request.gateway_node.as_str(), "gateway_node")?;
    if request.connected_since_epoch_seconds == 0 {
        return Err(DataLayerM9RealtimeDeliveryError::EmptyField(
            "connected_since_epoch_seconds",
        ));
    }
    if request.last_heartbeat_epoch_seconds < request.connected_since_epoch_seconds {
        return Err(DataLayerM9RealtimeDeliveryError::InvalidTimestampOrder {
            connected_since_epoch_seconds: request.connected_since_epoch_seconds,
            last_heartbeat_epoch_seconds: request.last_heartbeat_epoch_seconds,
        });
    }
    Ok(parsed_agent_did)
}

fn normalize_capabilities(
    mut capabilities_active: Vec<String>,
) -> Result<Vec<String>, DataLayerM9RealtimeDeliveryError> {
    capabilities_active.sort();
    capabilities_active.dedup();
    if capabilities_active.iter().any(|value| value.trim().is_empty()) {
        return Err(DataLayerM9RealtimeDeliveryError::EmptyField("capabilities_active"));
    }
    Ok(capabilities_active)
}

fn build_presence_record(
    request: DataLayerM9PresenceConnectRequest,
    agent_did: &str,
    capabilities_active: Vec<String>,
) -> DataLayerM9PresenceRecord {
    DataLayerM9PresenceRecord {
        owner_did: request.owner_did,
        agent_did: agent_did.to_owned(),
        connected_since_epoch_seconds: request.connected_since_epoch_seconds,
        last_heartbeat_epoch_seconds: request.last_heartbeat_epoch_seconds,
        gateway_node: request.gateway_node,
        capabilities_active,
    }
}
