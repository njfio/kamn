use std::fmt;

use super::BridgeDirection;

/// Errors emitted by bridge adapter validation, policy, and conversion flows.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BridgeAdapterError {
    /// Inbound message ID was observed previously.
    DuplicateInboundMessageId(String),
    /// Outbound request ID was observed previously.
    DuplicateOutboundRequestId(String),
    /// A required field was empty after normalization.
    EmptyField(&'static str),
    /// DID failed canonical parsing/validation.
    InvalidDid {
        /// DID-carrying field name.
        field: &'static str,
        /// Stable reason marker for policy and contract tests.
        reason_code: &'static str,
        /// Canonical parser detail string.
        detail: String,
    },
    /// Timestamp value was invalid for the given field.
    InvalidTimestamp(&'static str),
    /// Envelope nonce must be positive.
    InvalidNonce(u64),
    /// Inbound message exceeded allowed freshness window.
    StaleInboundMessage {
        /// Qualified inbound message identifier.
        bridge_message_id: String,
        /// Original receive timestamp.
        received_at_unix: u64,
        /// Processing observation timestamp.
        observed_at_unix: u64,
        /// Maximum supported message age in seconds.
        max_age_secs: u64,
    },
    /// Policy hook rejected traffic in the given direction.
    PolicyDenied {
        /// Traffic direction denied by policy.
        direction: BridgeDirection,
        /// Human-readable policy denial reason.
        reason: String,
    },
    /// Adapter returned an outbound ID that does not match request ID.
    OutboundRequestIdMismatch {
        /// Request ID expected by engine.
        expected: String,
        /// Request ID returned by adapter.
        actual: String,
    },
    /// Canonical envelope validation failure.
    Envelope(String),
}

impl fmt::Display for BridgeAdapterError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let text = display_text(self);
        f.write_str(text.as_str())
    }
}

fn display_text(error: &BridgeAdapterError) -> String {
    match error {
        BridgeAdapterError::DuplicateInboundMessageId(_)
        | BridgeAdapterError::DuplicateOutboundRequestId(_)
        | BridgeAdapterError::PolicyDenied { .. }
        | BridgeAdapterError::OutboundRequestIdMismatch { .. }
        | BridgeAdapterError::Envelope(_) => operation_text(error),
        _ => validation_text(error),
    }
}

fn stale_message_text(error: &BridgeAdapterError) -> String {
    match error {
        BridgeAdapterError::StaleInboundMessage {
            bridge_message_id,
            received_at_unix,
            observed_at_unix,
            max_age_secs,
        } => format!(
            "stale inbound message: id={bridge_message_id}, received_at_unix={received_at_unix}, observed_at_unix={observed_at_unix}, max_age_secs={max_age_secs}"
        ),
        _ => fallback_display_text(error),
    }
}

fn operation_text(error: &BridgeAdapterError) -> String {
    match error {
        BridgeAdapterError::DuplicateInboundMessageId(message_id) => {
            format!("duplicate inbound message id: {message_id}")
        }
        BridgeAdapterError::DuplicateOutboundRequestId(request_id) => {
            format!("duplicate outbound request id: {request_id}")
        }
        BridgeAdapterError::PolicyDenied { direction, reason } => {
            format!("policy denied {direction:?} traffic: {reason}")
        }
        BridgeAdapterError::OutboundRequestIdMismatch { expected, actual } => {
            format!("outbound request id mismatch: expected {expected}, got {actual}")
        }
        BridgeAdapterError::Envelope(value) => format!("invalid canonical envelope: {value}"),
        _ => fallback_display_text(error),
    }
}

fn validation_text(error: &BridgeAdapterError) -> String {
    match error {
        BridgeAdapterError::EmptyField(field) => format!("field must not be empty: {field}"),
        BridgeAdapterError::InvalidDid {
            field,
            reason_code,
            detail,
        } => format!("invalid did field {field}: {reason_code} ({detail})"),
        BridgeAdapterError::InvalidTimestamp(field) => format!("timestamp must be > 0: {field}"),
        BridgeAdapterError::InvalidNonce(value) => {
            format!("nonce must be greater than zero: {value}")
        }
        BridgeAdapterError::StaleInboundMessage { .. } => stale_message_text(error),
        _ => fallback_display_text(error),
    }
}

fn fallback_display_text(error: &BridgeAdapterError) -> String {
    format!("bridge adapter error display route mismatch: {error:?}")
}

impl std::error::Error for BridgeAdapterError {}

impl BridgeAdapterError {
    /// Stable reason-code taxonomy for bridge adapter failures.
    pub fn reason_code(&self) -> &'static str {
        match self {
            Self::DuplicateInboundMessageId(_) => "bridge_adapter_duplicate_inbound_message_id",
            Self::DuplicateOutboundRequestId(_) => "bridge_adapter_duplicate_outbound_request_id",
            Self::EmptyField(_) => "bridge_adapter_empty_field",
            Self::InvalidDid { reason_code, .. } => reason_code,
            Self::InvalidTimestamp(_) => "bridge_adapter_invalid_timestamp",
            Self::InvalidNonce(_) => "bridge_adapter_invalid_nonce",
            Self::StaleInboundMessage { .. } => "bridge_adapter_stale_inbound_message",
            Self::PolicyDenied { .. } => "bridge_adapter_policy_denied",
            Self::OutboundRequestIdMismatch { .. } => "bridge_adapter_outbound_request_id_mismatch",
            Self::Envelope(_) => "bridge_adapter_invalid_envelope",
        }
    }
}
