use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SdkError {
    NotImplemented(&'static str),
    InvalidInput {
        field: &'static str,
        reason: &'static str,
    },
    NotFound {
        entity: &'static str,
        id: String,
    },
    Conflict(&'static str),
    InsufficientFunds {
        available: u64,
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
