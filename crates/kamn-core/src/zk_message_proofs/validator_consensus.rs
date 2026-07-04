mod error_display;
mod evaluator;
mod status;

use crate::AgentDid;
use std::collections::BTreeSet;

pub use evaluator::ValidatorProofConsensusEvaluator;

/// Per-validator verdict on one processor proof artifact.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ValidatorProofVerdict {
    /// Valid variant for this public contract enum.
    Valid,
    /// Invalid variant for this public contract enum.
    Invalid,
    /// Replay variant for this public contract enum.
    Replay,
}

/// One validator attestation participating in proof consensus.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidatorProofAttestation {
    /// Attestation id carried by this public contract model.
    pub attestation_id: String,
    /// Validator did carried by this public contract model.
    pub validator_did: String,
    /// Message id carried by this public contract model.
    pub message_id: String,
    /// Artifact id carried by this public contract model.
    pub artifact_id: String,
    /// Verdict carried by this public contract model.
    pub verdict: ValidatorProofVerdict,
}

impl ValidatorProofAttestation {
    /// Creates a new value for this public contract type.
    pub fn new(
        attestation_id: &str,
        validator_did: &str,
        message_id: &str,
        artifact_id: &str,
        verdict: ValidatorProofVerdict,
    ) -> Result<Self, ValidatorProofConsensusError> {
        require_non_empty_consensus_field("attestation_id", attestation_id)?;
        require_non_empty_consensus_field("validator_did", validator_did)?;
        require_non_empty_consensus_field("message_id", message_id)?;
        require_non_empty_consensus_field("artifact_id", artifact_id)?;
        AgentDid::parse(validator_did).map_err(|error| {
            ValidatorProofConsensusError::InvalidValidatorDid(error.to_string())
        })?;
        Ok(Self {
            attestation_id: attestation_id.to_owned(),
            validator_did: validator_did.to_owned(),
            message_id: message_id.to_owned(),
            artifact_id: artifact_id.to_owned(),
            verdict,
        })
    }
}

/// Consensus input for a single message and artifact.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidatorProofConsensusInput {
    /// Message id carried by this public contract model.
    pub message_id: String,
    /// Artifact id carried by this public contract model.
    pub artifact_id: String,
    /// Attestations carried by this public contract model.
    pub attestations: Vec<ValidatorProofAttestation>,
}

impl ValidatorProofConsensusInput {
    /// Creates a new value for this public contract type.
    pub fn new(
        message_id: &str,
        artifact_id: &str,
        attestations: Vec<ValidatorProofAttestation>,
    ) -> Result<Self, ValidatorProofConsensusError> {
        require_non_empty_consensus_field("message_id", message_id)?;
        require_non_empty_consensus_field("artifact_id", artifact_id)?;
        if attestations.is_empty() {
            return Err(ValidatorProofConsensusError::EmptyAttestations);
        }
        Ok(Self {
            message_id: message_id.to_owned(),
            artifact_id: artifact_id.to_owned(),
            attestations,
        })
    }
}

/// Aggregate consensus status after tallying validator attestations.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ValidatorProofConsensusStatus {
    /// Consensus valid variant for this public contract enum.
    ConsensusValid,
    /// Consensus invalid variant for this public contract enum.
    ConsensusInvalid,
    /// Consensus replay variant for this public contract enum.
    ConsensusReplay,
    /// Validator mismatch variant for this public contract enum.
    ValidatorMismatch,
}

/// Final consensus decision emitted by the validator evaluator.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidatorProofConsensusDecision {
    /// Message id carried by this public contract model.
    pub message_id: String,
    /// Artifact id carried by this public contract model.
    pub artifact_id: String,
    /// Required quorum carried by this public contract model.
    pub required_quorum: usize,
    /// Validator count carried by this public contract model.
    pub validator_count: usize,
    /// Validator dids carried by this public contract model.
    pub validator_dids: Vec<String>,
    /// Valid attestation count carried by this public contract model.
    pub valid_attestation_count: usize,
    /// Invalid attestation count carried by this public contract model.
    pub invalid_attestation_count: usize,
    /// Replay attestation count carried by this public contract model.
    pub replay_attestation_count: usize,
    /// Status carried by this public contract model.
    pub status: ValidatorProofConsensusStatus,
}

/// Validation and quorum errors returned while building consensus.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ValidatorProofConsensusError {
    /// Invalid required quorum variant for this public contract enum.
    InvalidRequiredQuorum(usize),
    /// Invalid field variant for this public contract enum.
    InvalidField {
        /// Field carried by this enum variant.
        field: &'static str,
    },
    /// Empty attestations variant for this public contract enum.
    EmptyAttestations,
    /// Invalid validator did variant for this public contract enum.
    InvalidValidatorDid(String),
    /// Attestation message mismatch variant for this public contract enum.
    AttestationMessageMismatch {
        /// Expected carried by this enum variant.
        expected: String,
        /// Found carried by this enum variant.
        found: String,
    },
    /// Attestation artifact mismatch variant for this public contract enum.
    AttestationArtifactMismatch {
        /// Expected carried by this enum variant.
        expected: String,
        /// Found carried by this enum variant.
        found: String,
    },
    /// Duplicate validator variant for this public contract enum.
    DuplicateValidator(String),
    /// Duplicate attestation id variant for this public contract enum.
    DuplicateAttestationId(String),
    /// Attestation replay variant for this public contract enum.
    AttestationReplay(String),
    /// Insufficient attestations variant for this public contract enum.
    InsufficientAttestations {
        /// Required carried by this enum variant.
        required: usize,
        /// Received carried by this enum variant.
        received: usize,
    },
}

pub(super) fn require_non_empty_consensus_field(
    field: &'static str,
    value: &str,
) -> Result<(), ValidatorProofConsensusError> {
    if value.trim().is_empty() {
        return Err(ValidatorProofConsensusError::InvalidField { field });
    }
    Ok(())
}
