use super::{BridgeAdapterError, BridgeInboundEnvelope, BridgeInboundEnvelopeValidated};
use super::{BridgeOutboundRequest, BridgeOutboundRequestValidated, NormalizedInboundMessage};
use crate::AgentDid;

pub(crate) const DEFAULT_MAX_INBOUND_AGE_SECS: u64 = 300;
pub(crate) const BRIDGE_ADAPTER_INVALID_BRIDGE_AGENT_DID_REASON_CODE: &str =
    "bridge_adapter_invalid_bridge_agent_did";
pub(crate) const BRIDGE_ADAPTER_INVALID_TARGET_AGENT_DID_REASON_CODE: &str =
    "bridge_adapter_invalid_target_agent_did";
pub(crate) const BRIDGE_ADAPTER_INVALID_FROM_AGENT_DID_REASON_CODE: &str =
    "bridge_adapter_invalid_from_agent_did";
pub(crate) const BRIDGE_ADAPTER_INVALID_NORMALIZED_TARGET_AGENT_DID_REASON_CODE: &str =
    "bridge_adapter_invalid_normalized_target_agent_did";

pub(crate) fn validate_inbound_envelope(
    inbound: &BridgeInboundEnvelope,
) -> Result<BridgeInboundEnvelopeValidated, BridgeAdapterError> {
    BridgeInboundEnvelopeValidated::try_from(inbound)
}

pub(crate) fn validate_normalized_inbound(
    normalized: &NormalizedInboundMessage,
) -> Result<(), BridgeAdapterError> {
    validate_non_empty(
        "normalized_inbound_message.bridge_message_id",
        &normalized.bridge_message_id,
    )?;
    validate_non_empty(
        "normalized_inbound_message.sender_handle",
        &normalized.sender_handle,
    )?;
    validate_non_empty(
        "normalized_inbound_message.source_channel",
        &normalized.source_channel,
    )?;
    parse_agent_did(
        normalized.target_agent_did.as_str(),
        "normalized_inbound_message.target_agent_did",
        BRIDGE_ADAPTER_INVALID_NORMALIZED_TARGET_AGENT_DID_REASON_CODE,
    )?;
    validate_non_empty("normalized_inbound_message.body", &normalized.body)?;
    validate_non_empty(
        "normalized_inbound_message.received_at",
        &normalized.received_at,
    )?;
    validate_timestamp(
        "normalized_inbound_message.received_at_unix",
        normalized.received_at_unix,
    )?;
    if normalized.platform.label().is_empty() {
        return Err(BridgeAdapterError::EmptyField(
            "normalized_inbound_message.platform",
        ));
    }
    Ok(())
}

pub(crate) fn validate_outbound_request(
    request: &BridgeOutboundRequest,
) -> Result<BridgeOutboundRequestValidated, BridgeAdapterError> {
    BridgeOutboundRequestValidated::try_from(request)
}

pub(crate) fn validate_non_empty(
    field: &'static str,
    value: &str,
) -> Result<(), BridgeAdapterError> {
    if value.trim().is_empty() {
        return Err(BridgeAdapterError::EmptyField(field));
    }
    Ok(())
}

pub(crate) fn parse_agent_did(
    value: &str,
    field: &'static str,
    reason_code: &'static str,
) -> Result<AgentDid, BridgeAdapterError> {
    AgentDid::parse(value).map_err(|error| BridgeAdapterError::InvalidDid {
        field,
        reason_code,
        detail: error.to_string(),
    })
}

pub(crate) fn validate_timestamp(
    field: &'static str,
    value: u64,
) -> Result<(), BridgeAdapterError> {
    if value == 0 {
        return Err(BridgeAdapterError::InvalidTimestamp(field));
    }
    Ok(())
}

pub(crate) fn escape_json(value: &str) -> String {
    let mut escaped = String::with_capacity(value.len());
    for ch in value.chars() {
        match ch {
            '\\' => escaped.push_str("\\\\"),
            '"' => escaped.push_str("\\\""),
            '\n' => escaped.push_str("\\n"),
            '\r' => escaped.push_str("\\r"),
            '\t' => escaped.push_str("\\t"),
            _ => escaped.push(ch),
        }
    }
    escaped
}
