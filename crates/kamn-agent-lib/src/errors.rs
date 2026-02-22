use kamn_sdk::SdkError;
use std::fmt;

/// Errors returned by `kamn-agent-lib` APIs.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AgentLibError {
    /// Input validation failure.
    InvalidInput {
        /// Input field name.
        field: &'static str,
        /// Failure reason.
        reason: String,
    },
    /// Operation is intentionally unavailable in the current phase.
    UnsupportedOperation(&'static str),
    /// Internal state or synchronization failure.
    Internal(String),
    /// Propagated SDK-layer failure.
    Sdk(SdkError),
}

impl fmt::Display for AgentLibError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidInput { field, reason } => {
                write!(f, "invalid input for {field}: {reason}")
            }
            Self::UnsupportedOperation(operation) => {
                write!(f, "unsupported operation in phase-1: {operation}")
            }
            Self::Internal(reason) => write!(f, "internal error: {reason}"),
            Self::Sdk(error) => write!(f, "sdk error: {error}"),
        }
    }
}

impl std::error::Error for AgentLibError {}

impl From<SdkError> for AgentLibError {
    fn from(value: SdkError) -> Self {
        Self::Sdk(value)
    }
}
