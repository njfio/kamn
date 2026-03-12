use std::cell::RefCell;
use std::collections::BTreeSet;

use super::{
    BridgeAdapterError, BridgeInboundEnvelope, BridgeOutboundEnvelope, BridgeOutboundRequest,
    NormalizedInboundMessage,
};
use crate::bridge_adapter::support::validate_non_empty;

pub(super) fn ensure_fresh(
    normalized: &NormalizedInboundMessage,
    observed_at_unix: u64,
    max_inbound_age_secs: u64,
) -> Result<(), BridgeAdapterError> {
    let age_secs = observed_at_unix.saturating_sub(normalized.received_at_unix);
    if age_secs > max_inbound_age_secs {
        return Err(BridgeAdapterError::StaleInboundMessage {
            bridge_message_id: normalized.bridge_message_id.clone(),
            received_at_unix: normalized.received_at_unix,
            observed_at_unix,
            max_age_secs: max_inbound_age_secs,
        });
    }
    Ok(())
}

pub(super) fn record_inbound(
    seen: &RefCell<BTreeSet<String>>,
    bridge_message_id: String,
) -> Result<(), BridgeAdapterError> {
    if seen.borrow_mut().insert(bridge_message_id.clone()) {
        return Ok(());
    }
    Err(BridgeAdapterError::DuplicateInboundMessageId(
        bridge_message_id,
    ))
}

pub(super) fn validate_translated_outbound(
    request: &BridgeOutboundRequest,
    translated: &BridgeOutboundEnvelope,
) -> Result<(), BridgeAdapterError> {
    if translated.request_id != request.request_id {
        return Err(BridgeAdapterError::OutboundRequestIdMismatch {
            expected: request.request_id.clone(),
            actual: translated.request_id.clone(),
        });
    }
    validate_non_empty(
        "bridge_outbound_envelope.destination_channel_id",
        &translated.destination_channel_id,
    )?;
    validate_non_empty("bridge_outbound_envelope.payload", &translated.payload)
}

pub(super) fn record_outbound(
    seen: &RefCell<BTreeSet<String>>,
    request_id: String,
) -> Result<(), BridgeAdapterError> {
    if seen.borrow_mut().insert(request_id.clone()) {
        return Ok(());
    }
    Err(BridgeAdapterError::DuplicateOutboundRequestId(request_id))
}

pub(super) fn validate_envelope_inputs(
    recipient_keys: &[String],
    expires: &str,
    nonce: u64,
) -> Result<(), BridgeAdapterError> {
    if recipient_keys.is_empty() {
        return Err(BridgeAdapterError::EmptyField("recipient_keys"));
    }
    for key in recipient_keys {
        validate_non_empty("recipient_keys[]", key)?;
    }
    validate_non_empty("expires", expires)?;
    if nonce == 0 {
        return Err(BridgeAdapterError::InvalidNonce(nonce));
    }
    Ok(())
}
