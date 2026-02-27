use kamn_sdk::SdkError;
use kamn_types::AgentDidError;
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

impl From<AgentDidError> for AgentLibError {
    fn from(value: AgentDidError) -> Self {
        match value {
            AgentDidError::InvalidPrefix(_) => Self::InvalidInput {
                field: "did",
                reason: "must start with kamn:did:agent:".to_owned(),
            },
            AgentDidError::MissingMethodSpecificId => Self::InvalidInput {
                field: "did",
                reason: "method specific identifier is required".to_owned(),
            },
            AgentDidError::InvalidCharacter(_) => Self::InvalidInput {
                field: "did",
                reason: "contains unsupported characters".to_owned(),
            },
        }
    }
}
