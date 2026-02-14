use std::fmt;

/// Errors returned by SDK transport, validation, and workflow operations.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SdkError {
    /// Indicates the requested capability is not implemented by this client.
    NotImplemented(&'static str),
    /// Input validation failed for a specific field and reason.
    InvalidInput {
        /// Input field name that failed validation.
        field: &'static str,
        /// Validation failure reason.
        reason: &'static str,
    },
    /// Underlying transport operation failed.
    TransportFailure(&'static str),
    /// Service API returned structured error envelope.
    ServiceApiError {
        /// HTTP status code returned by service route.
        status: u16,
        /// Service error class marker.
        error: String,
        /// Deterministic machine-readable reason code.
        reason_code: String,
        /// Human-readable failure detail.
        message: String,
    },
    /// The caller expected one transport mode but the client uses another.
    TransportModeMismatch {
        /// Expected transport mode identifier.
        expected: &'static str,
        /// Actual transport mode identifier.
        found: &'static str,
    },
    /// Requested entity could not be located.
    NotFound {
        /// Entity type label.
        entity: &'static str,
        /// Entity identifier string.
        id: String,
    },
    /// Operation violated a conflict rule.
    Conflict(&'static str),
    /// Account balance is below required amount.
    InsufficientFunds {
        /// Available funds.
        available: u64,
        /// Required funds.
        required: u64,
    },
}

impl fmt::Display for SdkError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotImplemented(feature) => write!(f, "feature not implemented: {feature}"),
            Self::InvalidInput { field, reason } => {
                write!(f, "invalid input for {field}: {reason}")
            }
            Self::TransportFailure(reason) => write!(f, "transport failure: {reason}"),
            Self::ServiceApiError {
                status,
                error,
                reason_code,
                message,
            } => write!(
                f,
                "service api rejected request: status={status} error={error} reason_code={reason_code} message={message}"
            ),
            Self::TransportModeMismatch { expected, found } => {
                write!(
                    f,
                    "transport mode mismatch, expected {expected}, found {found}"
                )
            }
            Self::NotFound { entity, id } => write!(f, "{entity} not found: {id}"),
            Self::Conflict(reason) => write!(f, "conflict: {reason}"),
            Self::InsufficientFunds {
                available,
                required,
            } => write!(
                f,
                "insufficient funds, required {required}, available {available}"
            ),
        }
    }
}

impl std::error::Error for SdkError {}
