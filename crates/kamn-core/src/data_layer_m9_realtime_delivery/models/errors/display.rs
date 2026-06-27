use super::DataLayerM9RealtimeDeliveryError;
use std::fmt;

pub(super) fn format_error(
    error: &DataLayerM9RealtimeDeliveryError,
    f: &mut fmt::Formatter<'_>,
) -> fmt::Result {
    match error {
        DataLayerM9RealtimeDeliveryError::EmptyField(field) => write_empty_field(f, field),
        DataLayerM9RealtimeDeliveryError::InvalidDid { .. } => write_did_error(error, f),
        DataLayerM9RealtimeDeliveryError::OwnerScopeViolation { .. }
        | DataLayerM9RealtimeDeliveryError::PresenceVisibilityDenied { .. }
        | DataLayerM9RealtimeDeliveryError::ChannelPolicyCheckFailed { .. } => {
            write_policy_error(error, f)
        }
        DataLayerM9RealtimeDeliveryError::ChannelMembershipDenied { .. }
        | DataLayerM9RealtimeDeliveryError::AntiSpamAdmissionDenied { .. }
        | DataLayerM9RealtimeDeliveryError::AntiSpamEngineError { .. } => {
            write_dispatch_error(error, f)
        }
        DataLayerM9RealtimeDeliveryError::RuntimeBackpressurePolicyInvalid { .. }
        | DataLayerM9RealtimeDeliveryError::RuntimeBackpressureInputInvalid { .. }
        | DataLayerM9RealtimeDeliveryError::RuntimeBackpressureEvaluationFailed { .. } => {
            write_backpressure_error(error, f)
        }
        DataLayerM9RealtimeDeliveryError::InvalidTimestampOrder { .. }
        | DataLayerM9RealtimeDeliveryError::SameAgentRelationship
        | DataLayerM9RealtimeDeliveryError::DuplicateMessageId(_) => write_terminal_error(error, f),
    }
}

fn write_did_error(
    error: &DataLayerM9RealtimeDeliveryError,
    f: &mut fmt::Formatter<'_>,
) -> fmt::Result {
    if let DataLayerM9RealtimeDeliveryError::InvalidDid {
        field,
        reason_code,
        detail,
    } = error
    {
        return write_reason_detail(f, "invalid did field", field, reason_code, detail);
    }
    write!(
        f,
        "realtime delivery error formatter route mismatch: {error:?}"
    )
}

fn write_policy_error(
    error: &DataLayerM9RealtimeDeliveryError,
    f: &mut fmt::Formatter<'_>,
) -> fmt::Result {
    match error {
        DataLayerM9RealtimeDeliveryError::OwnerScopeViolation { reason_code } => {
            write_reason_only(f, "owner scope violation", reason_code)
        }
        DataLayerM9RealtimeDeliveryError::PresenceVisibilityDenied { reason_code } => {
            write_reason_only(f, "presence visibility denied", reason_code)
        }
        DataLayerM9RealtimeDeliveryError::ChannelPolicyCheckFailed {
            reason_code,
            detail,
        } => write_context_detail(f, "channel policy check failed", reason_code, detail),
        _ => write!(
            f,
            "realtime delivery error formatter route mismatch: {error:?}"
        ),
    }
}

fn write_dispatch_error(
    error: &DataLayerM9RealtimeDeliveryError,
    f: &mut fmt::Formatter<'_>,
) -> fmt::Result {
    match error {
        DataLayerM9RealtimeDeliveryError::ChannelMembershipDenied { reason_code } => {
            write_reason_only(f, "channel membership denied", reason_code)
        }
        DataLayerM9RealtimeDeliveryError::AntiSpamAdmissionDenied { reason_code } => {
            write_reason_only(f, "anti-spam admission denied", reason_code)
        }
        DataLayerM9RealtimeDeliveryError::AntiSpamEngineError { detail } => {
            write!(f, "anti-spam engine evaluation failed: {detail}")
        }
        _ => write!(
            f,
            "realtime delivery error formatter route mismatch: {error:?}"
        ),
    }
}

fn write_backpressure_error(
    error: &DataLayerM9RealtimeDeliveryError,
    f: &mut fmt::Formatter<'_>,
) -> fmt::Result {
    match error {
        DataLayerM9RealtimeDeliveryError::RuntimeBackpressurePolicyInvalid {
            reason_code,
            detail,
        } => write_context_detail(
            f,
            "runtime backpressure policy projection failed",
            reason_code,
            detail,
        ),
        DataLayerM9RealtimeDeliveryError::RuntimeBackpressureInputInvalid {
            reason_code,
            detail,
        } => write_context_detail(
            f,
            "runtime backpressure input projection failed",
            reason_code,
            detail,
        ),
        DataLayerM9RealtimeDeliveryError::RuntimeBackpressureEvaluationFailed {
            reason_code,
            detail,
        } => write_context_detail(
            f,
            "runtime backpressure evaluation failed",
            reason_code,
            detail,
        ),
        _ => write!(
            f,
            "realtime delivery error formatter route mismatch: {error:?}"
        ),
    }
}

fn write_terminal_error(
    error: &DataLayerM9RealtimeDeliveryError,
    f: &mut fmt::Formatter<'_>,
) -> fmt::Result {
    match error {
        DataLayerM9RealtimeDeliveryError::InvalidTimestampOrder {
            connected_since_epoch_seconds,
            last_heartbeat_epoch_seconds,
        } => write_timestamp_order(
            f,
            *connected_since_epoch_seconds,
            *last_heartbeat_epoch_seconds,
        ),
        DataLayerM9RealtimeDeliveryError::SameAgentRelationship => {
            write!(f, "relationship requester and counterparty must differ")
        }
        DataLayerM9RealtimeDeliveryError::DuplicateMessageId(value) => {
            write!(f, "duplicate message id: {value}")
        }
        _ => write!(
            f,
            "realtime delivery error formatter route mismatch: {error:?}"
        ),
    }
}

fn write_empty_field(f: &mut fmt::Formatter<'_>, field: &str) -> fmt::Result {
    write!(f, "field must not be empty: {field}")
}

fn write_reason_only(f: &mut fmt::Formatter<'_>, prefix: &str, reason_code: &str) -> fmt::Result {
    write!(f, "{prefix}: {reason_code}")
}

fn write_context_detail(
    f: &mut fmt::Formatter<'_>,
    prefix: &str,
    reason_code: &str,
    detail: &str,
) -> fmt::Result {
    write!(f, "{prefix}: {reason_code} ({detail})")
}

fn write_reason_detail(
    f: &mut fmt::Formatter<'_>,
    prefix: &str,
    field: &str,
    reason_code: &str,
    detail: &str,
) -> fmt::Result {
    write!(f, "{prefix} {field}: {reason_code} ({detail})")
}

fn write_timestamp_order(
    f: &mut fmt::Formatter<'_>,
    connected_since_epoch_seconds: u64,
    last_heartbeat_epoch_seconds: u64,
) -> fmt::Result {
    write!(
        f,
        "invalid timestamp order: connected_since={connected_since_epoch_seconds}, last_heartbeat={last_heartbeat_epoch_seconds}"
    )
}
