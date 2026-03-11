use crate::{AgentDid, DidDocument};

#[derive(Debug, Clone, PartialEq, Eq)]
/// Lifecycle mutation action applied to a DID record.
pub enum DidLifecycleMutationAction {
    /// Rotate to a new active DID document.
    Rotate {
        /// Replacement DID document.
        document: DidDocument,
    },
    /// Revoke the DID record.
    Revoke,
    /// Recover a revoked DID with replacement document.
    Recover {
        /// Recovery DID document.
        document: DidDocument,
    },
}

impl DidLifecycleMutationAction {
    pub(crate) fn label(&self) -> &'static str {
        match self {
            Self::Rotate { .. } => "rotate",
            Self::Revoke => "revoke",
            Self::Recover { .. } => "recover",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// Lifecycle mutation request envelope.
pub struct DidLifecycleMutationRequest {
    /// Target DID for mutation.
    pub did: AgentDid,
    /// Actor DID authorized to perform mutation.
    pub actor_did: String,
    /// Strictly increasing mutation nonce.
    pub nonce: u64,
    /// Requested lifecycle action.
    pub action: DidLifecycleMutationAction,
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// Evidence produced by successful lifecycle mutation.
pub struct DidLifecycleMutationEvidence {
    /// Target DID identifier.
    pub did: String,
    /// Actor DID that executed mutation.
    pub actor_did: String,
    /// Mutation nonce accepted by registry.
    pub nonce: u64,
    /// Action label executed by registry.
    pub action: &'static str,
    /// Revocation state before mutation.
    pub from_revoked: bool,
    /// Revocation state after mutation.
    pub to_revoked: bool,
    /// Stable reason code for policy lanes.
    pub reason_code: &'static str,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// Retry classification for register submissions.
pub enum DidSubmissionRetryClass {
    /// First submission for DID/key pair.
    NewSubmission,
    /// Submission in-flight and retry is allowed.
    RetryableInFlight,
    /// Submission already finalized and should not retry.
    FinalizedNoRetry,
    /// Idempotency key conflicts with existing submission.
    ConflictNoRetry,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// Finality status for DID registration submission.
pub enum DidSubmissionFinalityStatus {
    /// Submission finalized successfully.
    Confirmed,
    /// Submission finalized as rejected.
    Rejected,
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// Finality record tracked per DID submission.
pub struct DidSubmissionFinalityRecord {
    /// Idempotency key associated with submission.
    pub idempotency_key: String,
    /// Monotonic finality sequence number.
    pub sequence: u64,
    /// Finality status.
    pub status: DidSubmissionFinalityStatus,
    /// Provider receipt payload for finality event.
    pub receipt: String,
}
