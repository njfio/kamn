//! Transport request helper policies for Kolme runtime-commit transports.

use std::error::Error;
use std::fmt;

/// Error raised by transport request helper policy contracts.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum KolmeTransportRequestPolicyError {
    /// Request field failed deterministic validation.
    InvalidRequest {
        /// Field name.
        field: &'static str,
        /// Validation reason.
        reason: &'static str,
    },
}

impl fmt::Display for KolmeTransportRequestPolicyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidRequest { field, reason } => {
                write!(f, "invalid request {field}: {reason}")
            }
        }
    }
}

impl Error for KolmeTransportRequestPolicyError {}

/// Returns whether idempotency-key input is non-empty after trimming.
pub fn is_valid_transport_idempotency_key_input(idempotency_key: &str) -> bool {
    !idempotency_key.trim().is_empty()
}

/// Parses one authorization header value with deterministic trim + CRLF safeguards.
pub fn parse_authorization_header_value(
    value: &str,
) -> Result<String, KolmeTransportRequestPolicyError> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Err(KolmeTransportRequestPolicyError::InvalidRequest {
            field: "transport_authorization_header",
            reason: "must not be empty",
        });
    }
    if trimmed.contains('\r') || trimmed.contains('\n') {
        return Err(KolmeTransportRequestPolicyError::InvalidRequest {
            field: "transport_authorization_header",
            reason: "must be single-line",
        });
    }
    Ok(trimmed.to_owned())
}

/// Returns whether submit path resolves to the Kolme fork `/broadcast` route.
pub fn is_broadcast_submit_path(submit_path: &str) -> bool {
    let trimmed = submit_path.trim();
    if trimmed.is_empty() {
        return false;
    }
    let without_query = trimmed
        .split('?')
        .next()
        .unwrap_or(trimmed)
        .trim_end_matches('/');
    without_query == "/broadcast"
}
