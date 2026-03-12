use std::fmt;

/// Error taxonomy for M9 realtime delivery contracts.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DataLayerM9RealtimeDeliveryError {
    /// Required field was empty.
    EmptyField(&'static str),
    /// DID failed validation.
    InvalidDid {
        /// Input field carrying DID value.
        field: &'static str,
        /// Stable reason marker.
        reason_code: &'static str,
        /// Canonical parser detail.
        detail: String,
    },
    /// Owner-scope authorization failed.
    OwnerScopeViolation {
        /// Stable reason marker.
        reason_code: &'static str,
    },
    /// Presence visibility policy denied the query.
    PresenceVisibilityDenied {
        /// Stable reason marker.
        reason_code: &'static str,
    },
    /// Channel policy query failed.
    ChannelPolicyCheckFailed {
        /// Stable reason marker.
        reason_code: &'static str,
        /// Stable string detail from channel policy module.
        detail: String,
    },
    /// Channel membership validation denied sender/recipient scope.
    ChannelMembershipDenied {
        /// Stable reason marker.
        reason_code: &'static str,
    },
    /// Anti-spam admission denied dispatch.
    AntiSpamAdmissionDenied {
        /// Stable reason marker.
        reason_code: &'static str,
    },
    /// Anti-spam engine failed to evaluate input.
    AntiSpamEngineError {
        /// Stable string detail from anti-spam module.
        detail: String,
    },
    /// Runtime backpressure policy projection failed validation.
    RuntimeBackpressurePolicyInvalid {
        /// Stable reason marker.
        reason_code: &'static str,
        /// Stable detail from runtime policy validation.
        detail: String,
    },
    /// Runtime backpressure input projection failed validation.
    RuntimeBackpressureInputInvalid {
        /// Stable reason marker.
        reason_code: &'static str,
        /// Stable detail from runtime input validation.
        detail: String,
    },
    /// Runtime backpressure evaluation failed.
    RuntimeBackpressureEvaluationFailed {
        /// Stable reason marker.
        reason_code: &'static str,
        /// Stable detail from runtime evaluation.
        detail: String,
    },
    /// Connected-since and heartbeat timestamps were ordered incorrectly.
    InvalidTimestampOrder {
        /// Connection start timestamp.
        connected_since_epoch_seconds: u64,
        /// Last heartbeat timestamp.
        last_heartbeat_epoch_seconds: u64,
    },
    /// Request attempted to link one agent to itself.
    SameAgentRelationship,
    /// Duplicate message id in recipient queue/deferred state.
    DuplicateMessageId(String),
}

impl fmt::Display for DataLayerM9RealtimeDeliveryError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyField(field) => write!(f, "field must not be empty: {field}"),
            Self::InvalidDid {
                field,
                reason_code,
                detail,
            } => write!(f, "invalid did field {field}: {reason_code} ({detail})"),
            Self::OwnerScopeViolation { reason_code } => {
                write!(f, "owner scope violation: {reason_code}")
            }
            Self::PresenceVisibilityDenied { reason_code } => {
                write!(f, "presence visibility denied: {reason_code}")
            }
            Self::ChannelPolicyCheckFailed {
                reason_code,
                detail,
            } => write!(f, "channel policy check failed: {reason_code} ({detail})"),
            Self::ChannelMembershipDenied { reason_code } => {
                write!(f, "channel membership denied: {reason_code}")
            }
            Self::AntiSpamAdmissionDenied { reason_code } => {
                write!(f, "anti-spam admission denied: {reason_code}")
            }
            Self::AntiSpamEngineError { detail } => {
                write!(f, "anti-spam engine evaluation failed: {detail}")
            }
            Self::RuntimeBackpressurePolicyInvalid {
                reason_code,
                detail,
            } => write!(
                f,
                "runtime backpressure policy projection failed: {reason_code} ({detail})"
            ),
            Self::RuntimeBackpressureInputInvalid {
                reason_code,
                detail,
            } => write!(
                f,
                "runtime backpressure input projection failed: {reason_code} ({detail})"
            ),
            Self::RuntimeBackpressureEvaluationFailed {
                reason_code,
                detail,
            } => write!(
                f,
                "runtime backpressure evaluation failed: {reason_code} ({detail})"
            ),
            Self::InvalidTimestampOrder {
                connected_since_epoch_seconds,
                last_heartbeat_epoch_seconds,
            } => write!(
                f,
                "invalid timestamp order: connected_since={connected_since_epoch_seconds}, last_heartbeat={last_heartbeat_epoch_seconds}"
            ),
            Self::SameAgentRelationship => {
                write!(f, "relationship requester and counterparty must differ")
            }
            Self::DuplicateMessageId(value) => write!(f, "duplicate message id: {value}"),
        }
    }
}

impl std::error::Error for DataLayerM9RealtimeDeliveryError {}
