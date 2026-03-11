use super::DidRegistryError;
use std::fmt;

impl fmt::Display for DidRegistryError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::AlreadyRegistered(value) => write!(f, "did is already registered: {value}"),
            Self::ConflictingFinalityUpdate { did, sequence } => write!(
                f,
                "conflicting finality update for did {did} at sequence {sequence}"
            ),
            Self::ConflictingSubmissionIdempotencyKey {
                did,
                existing_key,
                provided_key,
            } => write!(
                f,
                "conflicting submission idempotency key for did {did}; existing {existing_key}, provided {provided_key}"
            ),
            Self::NotFound(value) => write!(f, "did not found: {value}"),
            Self::StaleFinalityUpdate {
                did,
                current_sequence,
                attempted_sequence,
            } => write!(
                f,
                "stale finality update for did {did}; current sequence {current_sequence}, attempted {attempted_sequence}"
            ),
            Self::UnknownSubmissionIdempotencyKey { did, idempotency_key } => write!(
                f,
                "unknown submission idempotency key for did {did}: {idempotency_key}"
            ),
            Self::Revoked(value) => write!(f, "did is revoked: {value}"),
            Self::DocumentDidMismatch { expected, actual } => {
                write!(f, "did document id mismatch, expected {expected}, got {actual}")
            }
            Self::InvalidMutationNonce { did, nonce } => {
                write!(f, "invalid lifecycle mutation nonce for did {did}: {nonce}")
            }
            Self::ReplayedMutationNonce {
                did,
                last_nonce,
                found,
            } => write!(
                f,
                "replayed lifecycle mutation nonce for did {did}; last {last_nonce}, found {found}"
            ),
            Self::UnauthorizedMutationActor {
                did,
                actor_did,
                required_actor,
            } => write!(
                f,
                "unauthorized lifecycle mutation actor for did {did}; actor {actor_did}, required {required_actor}"
            ),
            Self::InvalidLifecycleMutationTransition {
                did,
                action,
                from_revoked,
            } => write!(
                f,
                "invalid lifecycle mutation transition for did {did}; action {action}, revoked={from_revoked}"
            ),
            Self::ChainAdapterSubmitFailed { context, reason } => {
                write!(f, "did chain adapter submission failed for {context}: {reason}")
            }
            Self::PersistenceIo(value) => write!(f, "did registry persistence I/O error: {value}"),
            Self::PersistenceInvalidPayload(value) => {
                write!(f, "did registry persistence invalid payload: {value}")
            }
        }
    }
}

impl std::error::Error for DidRegistryError {}
