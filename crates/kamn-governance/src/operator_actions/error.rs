use crate::operator_binding::OperatorBindingError;
use std::fmt;

/// Errors returned by permissioned operator action service operations.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OperatorActionServiceError {
    /// Required input field was empty.
    EmptyField(&'static str),
    /// Binding authorization or binding mutation failed.
    Binding(OperatorBindingError),
}

impl fmt::Display for OperatorActionServiceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyField(field) => write!(f, "field must not be empty: {field}"),
            Self::Binding(error) => write!(f, "operator binding error: {error}"),
        }
    }
}

impl std::error::Error for OperatorActionServiceError {}

impl From<OperatorBindingError> for OperatorActionServiceError {
    fn from(value: OperatorBindingError) -> Self {
        Self::Binding(value)
    }
}
