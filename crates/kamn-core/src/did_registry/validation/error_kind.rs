#[derive(Debug, Clone, PartialEq, Eq)]
/// DID registry error taxonomy.
pub enum DidRegistryError {
    /// DID already exists in active state.
    AlreadyRegistered(String),
    /// Finality update conflicts with existing record at same/newer sequence.
    ConflictingFinalityUpdate {
        /// DID whose finality update conflicted.
        did: String,
        /// Sequence that could not be applied.
        sequence: u64,
    },
    /// Provided idempotency key conflicts with existing key for DID.
    ConflictingSubmissionIdempotencyKey {
        /// DID associated with the conflicting submission.
        did: String,
        /// Existing stored idempotency key.
        existing_key: String,
        /// Newly supplied idempotency key that conflicted.
        provided_key: String,
    },
    /// DID not found in registry.
    NotFound(String),
    /// Finality update sequence is older than current sequence.
    StaleFinalityUpdate {
        /// DID whose finality record is stale.
        did: String,
        /// Current accepted finality sequence.
        current_sequence: u64,
        /// Older attempted sequence.
        attempted_sequence: u64,
    },
    /// Finality update references unknown idempotency key.
    UnknownSubmissionIdempotencyKey {
        /// DID associated with the missing submission key.
        did: String,
        /// Idempotency key that was not found.
        idempotency_key: String,
    },
    /// DID exists but is revoked.
    Revoked(String),
    /// DID in document payload does not match target DID.
    DocumentDidMismatch {
        /// DID expected by the registry operation.
        expected: String,
        /// DID found in the submitted document.
        actual: String,
    },
    /// Mutation nonce is invalid (zero).
    InvalidMutationNonce {
        /// DID targeted by the mutation.
        did: String,
        /// Invalid nonce value.
        nonce: u64,
    },
    /// Mutation nonce replay/non-monotonic value detected.
    ReplayedMutationNonce {
        /// DID targeted by the mutation.
        did: String,
        /// Most recent accepted nonce.
        last_nonce: u64,
        /// Replayed or non-monotonic nonce.
        found: u64,
    },
    /// Actor DID is not authorized to mutate lifecycle state.
    UnauthorizedMutationActor {
        /// DID targeted by the mutation.
        did: String,
        /// Actor DID that attempted the mutation.
        actor_did: String,
        /// DID required to authorize the mutation.
        required_actor: String,
    },
    /// Requested lifecycle action is invalid for current state.
    InvalidLifecycleMutationTransition {
        /// DID targeted by the lifecycle action.
        did: String,
        /// Lifecycle action label.
        action: &'static str,
        /// Revocation state before the attempted action.
        from_revoked: bool,
    },
    /// Chain adapter submission failed while preparing or submitting payload.
    ChainAdapterSubmitFailed {
        /// Submission stage that failed.
        context: &'static str,
        /// Human-readable failure reason.
        reason: String,
    },
    /// Filesystem I/O failed while persisting chain adapter state.
    PersistenceIo(String),
    /// Persisted chain adapter payload was invalid.
    PersistenceInvalidPayload(String),
}

impl DidRegistryError {
    /// Returns stable reason code for telemetry/policy contract lanes.
    pub fn reason_code(&self) -> &'static str {
        match self {
            Self::AlreadyRegistered(_) => "did_registry_already_registered",
            Self::ConflictingFinalityUpdate { .. } => "did_registry_finality_conflict",
            Self::ConflictingSubmissionIdempotencyKey { .. } => {
                "did_registry_submission_key_conflict"
            }
            Self::NotFound(_) => "did_registry_not_found",
            Self::StaleFinalityUpdate { .. } => "did_registry_finality_stale",
            Self::UnknownSubmissionIdempotencyKey { .. } => "did_registry_submission_key_unknown",
            Self::Revoked(_) => "did_registry_revoked",
            Self::DocumentDidMismatch { .. } => "did_registry_document_did_mismatch",
            Self::InvalidMutationNonce { .. } => "did_lifecycle_mutation_nonce_invalid",
            Self::ReplayedMutationNonce { .. } => "did_lifecycle_mutation_nonce_replay",
            Self::UnauthorizedMutationActor { .. } => "did_lifecycle_mutation_unauthorized_actor",
            Self::InvalidLifecycleMutationTransition { .. } => {
                "did_lifecycle_mutation_invalid_transition"
            }
            Self::ChainAdapterSubmitFailed { .. } => "did_chain_adapter_submit_failed",
            Self::PersistenceIo(_) => "did_registry_persistence_io",
            Self::PersistenceInvalidPayload(_) => "did_registry_persistence_invalid_payload",
        }
    }
}
