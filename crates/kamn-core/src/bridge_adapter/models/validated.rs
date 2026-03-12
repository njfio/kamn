use crate::AgentDid;

use super::{BridgeInboundEnvelope, BridgeOutboundRequest};
use crate::bridge_adapter::support::{
    parse_agent_did, validate_non_empty, validate_timestamp,
    BRIDGE_ADAPTER_INVALID_FROM_AGENT_DID_REASON_CODE,
    BRIDGE_ADAPTER_INVALID_TARGET_AGENT_DID_REASON_CODE,
};
use crate::bridge_adapter::BridgeAdapterError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct BridgeInboundEnvelopeValidated {
    pub(crate) external_message_id: String,
    pub(crate) external_sender_id: String,
    pub(crate) external_channel_id: String,
    pub(crate) target_agent_did: AgentDid,
    pub(crate) body: String,
    pub(crate) received_at: String,
    pub(crate) received_at_unix: u64,
}

impl TryFrom<&BridgeInboundEnvelope> for BridgeInboundEnvelopeValidated {
    type Error = BridgeAdapterError;

    fn try_from(inbound: &BridgeInboundEnvelope) -> Result<Self, Self::Error> {
        validate_inbound_required_fields(inbound)?;
        let target_agent_did = parse_inbound_target(inbound)?;
        Ok(build_validated_inbound(inbound, target_agent_did))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct BridgeOutboundRequestValidated {
    pub(crate) request_id: String,
    pub(crate) destination_channel_id: String,
    pub(crate) body: String,
}

impl TryFrom<&BridgeOutboundRequest> for BridgeOutboundRequestValidated {
    type Error = BridgeAdapterError;

    fn try_from(request: &BridgeOutboundRequest) -> Result<Self, Self::Error> {
        validate_non_empty("bridge_outbound_request.request_id", &request.request_id)?;
        let _ = parse_agent_did(
            request.from_agent_did.as_str(),
            "bridge_outbound_request.from_agent_did",
            BRIDGE_ADAPTER_INVALID_FROM_AGENT_DID_REASON_CODE,
        )?;
        validate_non_empty(
            "bridge_outbound_request.destination_channel_id",
            &request.destination_channel_id,
        )?;
        validate_non_empty("bridge_outbound_request.body", &request.body)?;
        validate_non_empty("bridge_outbound_request.created_at", &request.created_at)?;
        Ok(Self {
            request_id: request.request_id.clone(),
            destination_channel_id: request.destination_channel_id.clone(),
            body: request.body.clone(),
        })
    }
}

fn validate_inbound_required_fields(
    inbound: &BridgeInboundEnvelope,
) -> Result<(), BridgeAdapterError> {
    validate_non_empty(
        "bridge_inbound_envelope.external_message_id",
        &inbound.external_message_id,
    )?;
    validate_non_empty(
        "bridge_inbound_envelope.external_sender_id",
        &inbound.external_sender_id,
    )?;
    validate_non_empty(
        "bridge_inbound_envelope.external_channel_id",
        &inbound.external_channel_id,
    )?;
    validate_non_empty("bridge_inbound_envelope.body", &inbound.body)?;
    validate_non_empty("bridge_inbound_envelope.received_at", &inbound.received_at)?;
    validate_timestamp(
        "bridge_inbound_envelope.received_at_unix",
        inbound.received_at_unix,
    )
}

fn parse_inbound_target(inbound: &BridgeInboundEnvelope) -> Result<AgentDid, BridgeAdapterError> {
    parse_agent_did(
        inbound.target_agent_did.as_str(),
        "bridge_inbound_envelope.target_agent_did",
        BRIDGE_ADAPTER_INVALID_TARGET_AGENT_DID_REASON_CODE,
    )
}

fn build_validated_inbound(
    inbound: &BridgeInboundEnvelope,
    target_agent_did: AgentDid,
) -> BridgeInboundEnvelopeValidated {
    BridgeInboundEnvelopeValidated {
        external_message_id: inbound.external_message_id.clone(),
        external_sender_id: inbound.external_sender_id.clone(),
        external_channel_id: inbound.external_channel_id.clone(),
        target_agent_did,
        body: inbound.body.clone(),
        received_at: inbound.received_at.clone(),
        received_at_unix: inbound.received_at_unix,
    }
}
