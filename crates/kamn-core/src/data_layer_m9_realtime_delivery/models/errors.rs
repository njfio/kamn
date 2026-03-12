mod display;

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
        display::format_error(self, f)
    }
}

impl std::error::Error for DataLayerM9RealtimeDeliveryError {}
