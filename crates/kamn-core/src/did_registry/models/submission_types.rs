use super::{DidLifecycleMutationEvidence, DidSubmissionRetryClass};
use crate::{AgentDid, DidDocument};

#[derive(Debug, Clone, PartialEq, Eq)]
/// Chain adapter request for DID registration submission.
pub struct DidChainSubmissionRequest {
    /// DID being submitted.
    pub did: AgentDid,
    /// Deterministic idempotency key.
    pub idempotency_key: String,
    /// DID document payload for registration.
    pub document: DidDocument,
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// Chain adapter receipt for a submission attempt.
pub struct DidChainSubmissionReceipt {
    /// Provider name that handled submission.
    pub provider: String,
    /// Provider transaction identifier.
    pub transaction_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// Submission outcome returned by chain adapter.
pub enum DidChainSubmissionOutcome {
    /// New submission accepted by provider.
    Submitted(DidChainSubmissionReceipt),
    /// Duplicate idempotency key acknowledged with existing receipt.
    Duplicate(DidChainSubmissionReceipt),
    /// Submission rejected by provider policy.
    Rejected {
        /// Provider-supplied rejection reason.
        reason: String,
    },
    /// Registry determined no provider call was needed.
    FinalizedNoOp,
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// Result envelope for registry + chain adapter registration flow.
pub struct DidChainSubmissionResult {
    /// DID processed by submission flow.
    pub did: AgentDid,
    /// Idempotency key used for this flow.
    pub idempotency_key: String,
    /// Retry classification returned by registry.
    pub retry_class: DidSubmissionRetryClass,
    /// Provider/registry submission outcome.
    pub outcome: DidChainSubmissionOutcome,
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// Chain adapter request for DID lifecycle mutation submission.
pub struct DidLifecycleChainSubmissionRequest {
    /// Target DID for lifecycle submission.
    pub did: AgentDid,
    /// Actor DID that initiated the lifecycle mutation.
    pub actor_did: String,
    /// Monotonic mutation nonce.
    pub nonce: u64,
    /// Lifecycle action label associated with submission.
    pub action: &'static str,
    /// Deterministic idempotency key for lifecycle submission.
    pub idempotency_key: String,
    /// Deterministic lifecycle payload hash marker.
    pub payload_hash: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// Result envelope for registry + chain adapter lifecycle mutation flow.
pub struct DidLifecycleChainSubmissionResult {
    /// DID processed by submission flow.
    pub did: AgentDid,
    /// Mutation nonce processed by submission flow.
    pub nonce: u64,
    /// Idempotency key used for this flow.
    pub idempotency_key: String,
    /// Retry classification returned by registry.
    pub retry_class: DidSubmissionRetryClass,
    /// Provider/registry submission outcome.
    pub outcome: DidChainSubmissionOutcome,
    /// Lifecycle evidence emitted by mutation state machine.
    pub evidence: DidLifecycleMutationEvidence,
}
