use crate::data_layer_m9_realtime_delivery::validation::{
    authorize_owner_scope, parse_agent_did, validate_non_empty,
};
use crate::{
    data_layer_m9_realtime_delivery::{
        DataLayerM9ChannelDispatchAuthorizationRequest, DataLayerM9RealtimeDeliveryError,
        DataLayerM9RealtimeDeliveryRegistry, DATA_LAYER_M9_CHANNEL_MEMBERSHIP_DENIED_REASON_CODE,
        DATA_LAYER_M9_CHANNEL_POLICY_QUERY_FAILED_REASON_CODE,
        DATA_LAYER_M9_INVALID_RECIPIENT_AGENT_DID_REASON_CODE,
        DATA_LAYER_M9_INVALID_SENDER_AGENT_DID_REASON_CODE,
    },
    ChannelStore,
};

impl DataLayerM9RealtimeDeliveryRegistry {
    /// Authorizes channel-scoped dispatch by enforcing sender/recipient membership.
    pub fn authorize_channel_dispatch(
        &self,
        channel_store: &ChannelStore,
        request: DataLayerM9ChannelDispatchAuthorizationRequest,
    ) -> Result<(), DataLayerM9RealtimeDeliveryError> {
        let (channel_id, sender_agent_did, recipient_agent_did) =
            validate_dispatch_authorization_request(&request)?;
        let sender_member = query_membership(
            channel_store,
            channel_id.as_str(),
            sender_agent_did.as_str(),
        )?;
        let recipient_member = query_membership(
            channel_store,
            channel_id.as_str(),
            recipient_agent_did.as_str(),
        )?;
        ensure_membership(sender_member, recipient_member)?;
        Ok(())
    }
}

fn validate_dispatch_authorization_request(
    request: &DataLayerM9ChannelDispatchAuthorizationRequest,
) -> Result<(String, crate::AgentDid, crate::AgentDid), DataLayerM9RealtimeDeliveryError> {
    authorize_owner_scope(
        request.requester_owner_did.as_str(),
        request.owner_did.as_str(),
    )?;
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
    Ok((
        request.channel_id.clone(),
        sender_agent_did,
        recipient_agent_did,
    ))
}

fn query_membership(
    channel_store: &ChannelStore,
    channel_id: &str,
    agent_did: &str,
) -> Result<bool, DataLayerM9RealtimeDeliveryError> {
    channel_store
        .is_member(channel_id, agent_did)
        .map_err(
            |error| DataLayerM9RealtimeDeliveryError::ChannelPolicyCheckFailed {
                reason_code: DATA_LAYER_M9_CHANNEL_POLICY_QUERY_FAILED_REASON_CODE,
                detail: error.to_string(),
            },
        )
}

fn ensure_membership(
    sender_member: bool,
    recipient_member: bool,
) -> Result<(), DataLayerM9RealtimeDeliveryError> {
    if !sender_member || !recipient_member {
        return Err(DataLayerM9RealtimeDeliveryError::ChannelMembershipDenied {
            reason_code: DATA_LAYER_M9_CHANNEL_MEMBERSHIP_DENIED_REASON_CODE,
        });
    }
    Ok(())
}
