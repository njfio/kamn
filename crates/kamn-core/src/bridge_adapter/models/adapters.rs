use super::{
    BridgeInboundEnvelope, BridgeOutboundEnvelope, BridgeOutboundRequest, BridgePlatform,
    NormalizedInboundMessage,
};
use crate::bridge_adapter::support::{
    escape_json, parse_agent_did, validate_inbound_envelope, validate_outbound_request,
    BRIDGE_ADAPTER_INVALID_BRIDGE_AGENT_DID_REASON_CODE,
};
use crate::bridge_adapter::BridgeAdapterError;
use crate::AgentDid;

/// Adapter contract for platform-specific ingress/egress translation.
pub trait BridgeAdapter {
    /// Runs the platform contract operation.
    fn platform(&self) -> BridgePlatform;
    /// Runs the bridge agent did contract operation.
    fn bridge_agent_did(&self) -> &str;
    /// Runs the normalize inbound contract operation.
    fn normalize_inbound(
        &self,
        inbound: &BridgeInboundEnvelope,
    ) -> Result<NormalizedInboundMessage, BridgeAdapterError>;
    /// Runs the translate outbound contract operation.
    fn translate_outbound(
        &self,
        request: &BridgeOutboundRequest,
    ) -> Result<BridgeOutboundEnvelope, BridgeAdapterError>;
}

/// Policy hook contract for inbound and outbound authorization checks.
pub trait BridgePolicyHook {
    /// Runs the authorize inbound contract operation.
    fn authorize_inbound(
        &self,
        normalized: &NormalizedInboundMessage,
    ) -> Result<(), BridgeAdapterError>;
    /// Runs the authorize outbound contract operation.
    fn authorize_outbound(&self, request: &BridgeOutboundRequest)
        -> Result<(), BridgeAdapterError>;
}

/// Permissive policy hook that authorizes all bridge traffic.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct AllowAllBridgePolicy;

impl AllowAllBridgePolicy {
    /// Creates a new value for this public contract type.
    pub fn new() -> Self {
        Self
    }
}

impl BridgePolicyHook for AllowAllBridgePolicy {
    fn authorize_inbound(
        &self,
        _normalized: &NormalizedInboundMessage,
    ) -> Result<(), BridgeAdapterError> {
        Ok(())
    }

    fn authorize_outbound(
        &self,
        _request: &BridgeOutboundRequest,
    ) -> Result<(), BridgeAdapterError> {
        Ok(())
    }
}

/// Default adapter that passes fields through with deterministic normalization.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PassThroughBridgeAdapter {
    platform: BridgePlatform,
    bridge_agent_did: AgentDid,
}

impl PassThroughBridgeAdapter {
    /// Creates a new value for this public contract type.
    pub fn new(
        platform: BridgePlatform,
        bridge_agent_did: &str,
    ) -> Result<Self, BridgeAdapterError> {
        if platform.label().is_empty() {
            return Err(BridgeAdapterError::EmptyField("platform"));
        }
        let bridge_agent_did = parse_agent_did(
            bridge_agent_did,
            "bridge_agent_did",
            BRIDGE_ADAPTER_INVALID_BRIDGE_AGENT_DID_REASON_CODE,
        )?;
        Ok(Self {
            platform,
            bridge_agent_did,
        })
    }
}

impl BridgeAdapter for PassThroughBridgeAdapter {
    fn platform(&self) -> BridgePlatform {
        self.platform.clone()
    }

    fn bridge_agent_did(&self) -> &str {
        self.bridge_agent_did.as_str()
    }

    fn normalize_inbound(
        &self,
        inbound: &BridgeInboundEnvelope,
    ) -> Result<NormalizedInboundMessage, BridgeAdapterError> {
        let validated_inbound = validate_inbound_envelope(inbound)?;
        Ok(NormalizedInboundMessage {
            bridge_message_id: format!(
                "{}:{}",
                self.platform.label(),
                validated_inbound.external_message_id
            ),
            sender_handle: validated_inbound.external_sender_id,
            source_channel: validated_inbound.external_channel_id,
            target_agent_did: validated_inbound.target_agent_did.as_str().to_owned(),
            body: validated_inbound.body,
            received_at: validated_inbound.received_at,
            received_at_unix: validated_inbound.received_at_unix,
            platform: self.platform(),
        })
    }

    fn translate_outbound(
        &self,
        request: &BridgeOutboundRequest,
    ) -> Result<BridgeOutboundEnvelope, BridgeAdapterError> {
        let validated_request = validate_outbound_request(request)?;
        Ok(BridgeOutboundEnvelope {
            request_id: validated_request.request_id.clone(),
            destination_channel_id: validated_request.destination_channel_id.clone(),
            payload: format!(
                "{{\"platform\":\"{}\",\"channel\":\"{}\",\"message\":\"{}\"}}",
                self.platform.label(),
                escape_json(&validated_request.destination_channel_id),
                escape_json(&validated_request.body),
            ),
            platform: self.platform(),
        })
    }
}
