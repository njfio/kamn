use crate::{ChannelStore, data_layer_m9_realtime_delivery::{
    DataLayerM9ChannelDispatchAuthorizationRequest, DataLayerM9RealtimeDeliveryError,
    DataLayerM9RealtimeDeliveryRegistry, DATA_LAYER_M9_CHANNEL_MEMBERSHIP_DENIED_REASON_CODE,
    DATA_LAYER_M9_CHANNEL_POLICY_QUERY_FAILED_REASON_CODE,
    DATA_LAYER_M9_INVALID_RECIPIENT_AGENT_DID_REASON_CODE,
    DATA_LAYER_M9_INVALID_SENDER_AGENT_DID_REASON_CODE,
},};
use crate::data_layer_m9_realtime_delivery::validation::{authorize_owner_scope, parse_agent_did, validate_non_empty};

impl DataLayerM9RealtimeDeliveryRegistry {
    /// Authorizes channel-scoped dispatch by enforcing sender/recipient membership.
    pub fn authorize_channel_dispatch(
        &self,
        channel_store: &ChannelStore,
        request: DataLayerM9ChannelDispatchAuthorizationRequest,
    ) -> Result<(), DataLayerM9RealtimeDeliveryError> {
        authorize_owner_scope(request.requester_owner_did.as_str(), request.owner_did.as_str())?;
        validate_non_empty(request.channel_id.as_str(), "channel_id")?;
        let sender_agent_did = parse_agent_did(
            request.sender_agent_did.as_str(),
            "sender_agent_did",
            DATA_LAYER_M9_INVALID_SENDER_AGENT_DID_REASON_CODE,
        )?;
        let recipient_agent_did = parse_agent_did(
            request.recipient_agent_did.as_str(),
            "recipient_agent_did",
            DATA_LAYER_M9_INVALID_RECIPIENT_AGENT_DID_REASON_CODE,
        )?;

        let sender_member = channel_store
            .is_member(request.channel_id.as_str(), sender_agent_did.as_str())
            .map_err(|error| DataLayerM9RealtimeDeliveryError::ChannelPolicyCheckFailed {
                reason_code: DATA_LAYER_M9_CHANNEL_POLICY_QUERY_FAILED_REASON_CODE,
                detail: error.to_string(),
            })?;
        let recipient_member = channel_store
            .is_member(request.channel_id.as_str(), recipient_agent_did.as_str())
            .map_err(|error| DataLayerM9RealtimeDeliveryError::ChannelPolicyCheckFailed {
                reason_code: DATA_LAYER_M9_CHANNEL_POLICY_QUERY_FAILED_REASON_CODE,
                detail: error.to_string(),
            })?;

        if !sender_member || !recipient_member {
            return Err(DataLayerM9RealtimeDeliveryError::ChannelMembershipDenied {
                reason_code: DATA_LAYER_M9_CHANNEL_MEMBERSHIP_DENIED_REASON_CODE,
            });
        }

        Ok(())
    }
}
