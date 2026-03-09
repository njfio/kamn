use crate::operator_binding::OperatorBindingAction;
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
/// Errors emitted by operator binding registration and authorization.
pub enum OperatorBindingError {
    /// Permission set was empty.
    EmptyPermissions,
    /// Proof field was empty.
    EmptyProofField(&'static str),
    /// Agent DID failed validation.
    InvalidAgentDid {
        /// Input field carrying the DID value.
        field: &'static str,
        /// Stable reason marker.
        reason_code: &'static str,
        /// Canonical parser detail.
        detail: String,
    },
    /// Operator DID failed validation.
    InvalidOperatorDid {
        /// Input field carrying the DID value.
        field: &'static str,
        /// Stable reason marker.
        reason_code: &'static str,
        /// Canonical parser detail.
        detail: String,
    },
    /// Proof type did not match required canonical type.
    InvalidProofType(String),
    /// Proof verification method prefix did not match operator DID.
    ProofVerificationMethodMismatch {
        /// Expected verification method prefix.
        expected_prefix: String,
        /// Actual verification method value.
        actual: String,
    },
    /// Binding already exists for `(agent_did, operator_did)`.
    DuplicateBinding {
        /// Agent DID.
        agent_did: String,
        /// Operator DID.
        operator_did: String,
    },
    /// Binding not found for `(agent_did, operator_did)`.
    MissingBinding {
        /// Agent DID.
        agent_did: String,
        /// Operator DID.
        operator_did: String,
    },
    /// Binding already revoked for `(agent_did, operator_did)`.
    RevokedBinding {
        /// Agent DID.
        agent_did: String,
        /// Operator DID.
        operator_did: String,
    },
    /// Operator attempted an unauthorized action.
    UnauthorizedAction {
        /// Operator DID.
        operator_did: String,
        /// Unauthorized action.
        action: OperatorBindingAction,
    },
}

impl fmt::Display for OperatorBindingError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyPermissions => write!(f, "permissions must not be empty"),
            Self::EmptyProofField(field) => write!(f, "proof field must not be empty: {field}"),
            Self::InvalidAgentDid {
                field,
                reason_code,
                detail,
            } => write!(f, "invalid did field {field}: {reason_code} ({detail})"),
            Self::InvalidOperatorDid {
                field,
                reason_code,
                detail,
            } => write!(f, "invalid did field {field}: {reason_code} ({detail})"),
            Self::InvalidProofType(value) => write!(f, "invalid proof type: {value}"),
            Self::ProofVerificationMethodMismatch {
                expected_prefix,
                actual,
            } => write!(
                f,
                "proof verification method mismatch, expected prefix {expected_prefix}, got {actual}"
            ),
            Self::DuplicateBinding {
                agent_did,
                operator_did,
            } => write!(f, "duplicate operator binding: {agent_did} + {operator_did}"),
            Self::MissingBinding {
                agent_did,
                operator_did,
            } => write!(f, "operator binding not found: {agent_did} + {operator_did}"),
            Self::RevokedBinding {
                agent_did,
                operator_did,
            } => write!(f, "operator binding revoked: {agent_did} + {operator_did}"),
            Self::UnauthorizedAction {
                operator_did,
                action,
            } => write!(
                f,
                "operator {operator_did} is unauthorized for action {action:?}"
            ),
        }
    }
}

impl std::error::Error for OperatorBindingError {}
