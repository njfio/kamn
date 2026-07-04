use super::DidRegistryError;
use std::fmt;

impl fmt::Display for DidRegistryError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&display_message(self))
    }
}

impl std::error::Error for DidRegistryError {}

fn display_message(error: &DidRegistryError) -> String {
    if let Some(message) = simple_display_message(error) {
        return message;
    }
    if let Some(message) = finality_display_message(error) {
        return message;
    }
    mutation_display_message(error)
}

fn simple_display_message(error: &DidRegistryError) -> Option<String> {
    match error {
        DidRegistryError::AlreadyRegistered(value) => {
            Some(format!("did is already registered: {value}"))
        }
        DidRegistryError::NotFound(value) => Some(format!("did not found: {value}")),
        DidRegistryError::Revoked(value) => Some(format!("did is revoked: {value}")),
        DidRegistryError::PersistenceIo(value) => {
            Some(format!("did registry persistence I/O error: {value}"))
        }
        DidRegistryError::PersistenceInvalidPayload(value) => {
            Some(format!("did registry persistence invalid payload: {value}"))
        }
        DidRegistryError::ChainAdapterSubmitFailed { context, reason } => Some(format!(
            "did chain adapter submission failed for {context}: {reason}"
        )),
        _ => None,
    }
}

fn finality_display_message(error: &DidRegistryError) -> Option<String> {
    match error {
        DidRegistryError::ConflictingFinalityUpdate { did, sequence } => {
            Some(format!("conflicting finality update for did {did} at sequence {sequence}"))
        }
        DidRegistryError::ConflictingSubmissionIdempotencyKey { did, existing_key, provided_key } => Some(
            format!(
                "conflicting submission idempotency key for did {did}; existing {existing_key}, provided {provided_key}"
            ),
        ),
        DidRegistryError::StaleFinalityUpdate { did, current_sequence, attempted_sequence } => Some(
            format!(
                "stale finality update for did {did}; current sequence {current_sequence}, attempted {attempted_sequence}"
            ),
        ),
        DidRegistryError::UnknownSubmissionIdempotencyKey { did, idempotency_key } => {
            Some(format!("unknown submission idempotency key for did {did}: {idempotency_key}"))
        }
        _ => None,
    }
}

fn mutation_display_message(error: &DidRegistryError) -> String {
    if let Some(message) = nonce_display_message(error) {
        return message;
    }
    transition_display_message(error)
}

fn nonce_display_message(error: &DidRegistryError) -> Option<String> {
    match error {
        DidRegistryError::DocumentDidMismatch { expected, actual } => Some(format!(
            "did document id mismatch, expected {expected}, got {actual}"
        )),
        DidRegistryError::InvalidMutationNonce { did, nonce } => Some(format!(
            "invalid lifecycle mutation nonce for did {did}: {nonce}"
        )),
        DidRegistryError::ReplayedMutationNonce {
            did,
            last_nonce,
            found,
        } => Some(format!(
            "replayed lifecycle mutation nonce for did {did}; last {last_nonce}, found {found}"
        )),
        _ => None,
    }
}

fn transition_display_message(error: &DidRegistryError) -> String {
    match error {
        DidRegistryError::UnauthorizedMutationActor {
            did,
            actor_did,
            required_actor,
        } => format!(
            "unauthorized lifecycle mutation actor for did {did}; actor {actor_did}, required {required_actor}"
        ),
        DidRegistryError::InvalidLifecycleMutationTransition {
            did,
            action,
            from_revoked,
        } => format!(
            "invalid lifecycle mutation transition for did {did}; action {action}, revoked={from_revoked}"
        ),
        _ => format!("did registry error formatter route mismatch: {error:?}"),
    }
}
