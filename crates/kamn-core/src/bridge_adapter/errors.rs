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
        match self {
            Self::DuplicateInboundMessageId(message_id) => {
                write!(f, "duplicate inbound message id: {message_id}")
            }
            Self::DuplicateOutboundRequestId(request_id) => {
                write!(f, "duplicate outbound request id: {request_id}")
            }
            Self::EmptyField(field) => write!(f, "field must not be empty: {field}"),
            Self::InvalidDid {
                field,
                reason_code,
                detail,
            } => write!(f, "invalid did field {field}: {reason_code} ({detail})"),
            Self::InvalidTimestamp(field) => write!(f, "timestamp must be > 0: {field}"),
            Self::InvalidNonce(value) => write!(f, "nonce must be greater than zero: {value}"),
            Self::StaleInboundMessage {
                bridge_message_id,
                received_at_unix,
                observed_at_unix,
                max_age_secs,
            } => write!(
                f,
                "stale inbound message: id={bridge_message_id}, received_at_unix={received_at_unix}, observed_at_unix={observed_at_unix}, max_age_secs={max_age_secs}"
            ),
            Self::PolicyDenied { direction, reason } => {
                write!(f, "policy denied {direction:?} traffic: {reason}")
            }
            Self::OutboundRequestIdMismatch { expected, actual } => write!(
                f,
                "outbound request id mismatch: expected {expected}, got {actual}"
            ),
            Self::Envelope(value) => write!(f, "invalid canonical envelope: {value}"),
        }
    }
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
