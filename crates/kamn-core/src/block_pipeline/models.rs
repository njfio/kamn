use std::error::Error;
use std::fmt::{Display, Formatter};

use crate::runtime::{
    ApproverQuorumDecision, ApproverQuorumError, ListenerQuorumDecision, ListenerQuorumError,
};
use crate::smoke::{ProducedBlock, SmokeError};

/// Input contract for one consensus-validation and block-commit round.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlockConsensusRoundInput {
    /// Listener quorum event identifier.
    pub listener_event_id: String,
    /// Listener quorum event sequence.
    pub listener_event_sequence: u64,
    /// Approver outbound action identifier.
    pub outbound_action_id: String,
    /// Listener votes as `(listener_did, attestation_id)`.
    pub listener_votes: Vec<(String, String)>,
    /// Approver votes as `(approver_did, attestation_id, payload_digest_override)`.
    pub approver_votes: Vec<(String, String, Option<String>)>,
}

/// Commit report emitted when a consensus round commits a block.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlockPipelineCommitReport {
    /// Committed block payload.
    pub block: ProducedBlock,
    /// Listener quorum decision used for admission.
    pub listener_decision: ListenerQuorumDecision,
    /// Approver quorum decision used for authorization.
    pub approver_decision: ApproverQuorumDecision,
    /// Deterministic payload digest used by approver quorum validation.
    pub payload_digest: String,
}

/// Error variants for block pipeline validation and commit flows.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BlockPipelineError {
    /// Listener quorum validation failed.
    Listener(ListenerQuorumError),
    /// Approver quorum validation failed.
    Approver(ApproverQuorumError),
    /// Smoke network transport/guard operation failed.
    Smoke(SmokeError),
    /// Pipeline attempted a consensus round with an empty mempool.
    EmptyMempool,
    /// Approver payload digest override mismatched deterministic block digest.
    ConsensusPayloadDigestMismatch { expected: String, found: String },
    /// Transport feed returned an error while draining mempool candidates.
    TransportFeed(String),
    /// Canonical commit store returned an error while persisting/listing records.
    CommitStore(String),
    /// Fork-choice hook rejected canonical candidate block.
    ForkChoiceRejected { reason_code: String },
    /// Restart/replay lineage drift detected for canonical commit persistence.
    ReplayDrift { reason_code: String, detail: String },
}

impl From<ListenerQuorumError> for BlockPipelineError {
    fn from(value: ListenerQuorumError) -> Self {
        Self::Listener(value)
    }
}

impl From<ApproverQuorumError> for BlockPipelineError {
    fn from(value: ApproverQuorumError) -> Self {
        Self::Approver(value)
    }
}

impl From<SmokeError> for BlockPipelineError {
    fn from(value: SmokeError) -> Self {
        Self::Smoke(value)
    }
}

impl Display for BlockPipelineError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Listener(err) => write!(f, "listener quorum failed: {err}"),
            Self::Approver(err) => write!(f, "approver quorum failed: {err}"),
            Self::Smoke(err) => write!(f, "smoke network operation failed: {err}"),
            Self::EmptyMempool => write!(f, "processor mempool is empty"),
            Self::ConsensusPayloadDigestMismatch { expected, found } => {
                write!(
                    f,
                    "approver payload digest mismatch: expected {expected}, found {found}"
                )
            }
            Self::TransportFeed(detail) => write!(f, "transport feed failed: {detail}"),
            Self::CommitStore(detail) => write!(f, "commit store failed: {detail}"),
            Self::ForkChoiceRejected { reason_code } => {
                write!(f, "fork choice rejected canonical candidate: {reason_code}")
            }
            Self::ReplayDrift {
                reason_code,
                detail,
            } => {
                write!(f, "replay drift detected: {reason_code}: {detail}")
            }
        }
    }
}

impl Error for BlockPipelineError {}
