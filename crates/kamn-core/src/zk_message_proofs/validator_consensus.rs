mod evaluator;

use crate::AgentDid;
use std::collections::BTreeSet;
use std::fmt;

pub use evaluator::ValidatorProofConsensusEvaluator;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ValidatorProofVerdict {
    Valid,
    Invalid,
    Replay,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidatorProofAttestation {
    pub attestation_id: String,
    pub validator_did: String,
    pub message_id: String,
    pub artifact_id: String,
    pub verdict: ValidatorProofVerdict,
}

impl ValidatorProofAttestation {
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidatorProofConsensusInput {
    pub message_id: String,
    pub artifact_id: String,
    pub attestations: Vec<ValidatorProofAttestation>,
}

impl ValidatorProofConsensusInput {
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ValidatorProofConsensusStatus {
    ConsensusValid,
    ConsensusInvalid,
    ConsensusReplay,
    ValidatorMismatch,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidatorProofConsensusDecision {
    pub message_id: String,
    pub artifact_id: String,
    pub required_quorum: usize,
    pub validator_count: usize,
    pub validator_dids: Vec<String>,
    pub valid_attestation_count: usize,
    pub invalid_attestation_count: usize,
    pub replay_attestation_count: usize,
    pub status: ValidatorProofConsensusStatus,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ValidatorProofConsensusError {
    InvalidRequiredQuorum(usize),
    InvalidField { field: &'static str },
    EmptyAttestations,
    InvalidValidatorDid(String),
    AttestationMessageMismatch { expected: String, found: String },
    AttestationArtifactMismatch { expected: String, found: String },
    DuplicateValidator(String),
    DuplicateAttestationId(String),
    AttestationReplay(String),
    InsufficientAttestations { required: usize, received: usize },
}

impl fmt::Display for ValidatorProofConsensusError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidRequiredQuorum(required_quorum) => write!(f, "validator proof required quorum must be greater than zero, found {required_quorum}"),
            Self::InvalidField { field } => write!(f, "{field} must not be empty"),
            Self::EmptyAttestations => write!(f, "validator proof attestations must not be empty"),
            Self::InvalidValidatorDid(value) => write!(f, "validator proof attestation DID is invalid: {value}"),
            Self::AttestationMessageMismatch { expected, found } => write!(f, "validator attestation message mismatch: expected {expected}, found {found}"),
            Self::AttestationArtifactMismatch { expected, found } => write!(f, "validator attestation artifact mismatch: expected {expected}, found {found}"),
            Self::DuplicateValidator(validator_did) => write!(f, "validator proof attestation duplicate validator: {validator_did}"),
            Self::DuplicateAttestationId(attestation_id) => write!(f, "validator proof attestation id duplicated in input: {attestation_id}"),
            Self::AttestationReplay(attestation_id) => write!(f, "validator proof attestation replay detected: {attestation_id}"),
            Self::InsufficientAttestations { required, received } => write!(f, "validator proof quorum insufficient attestations: required {required}, received {received}"),
        }
    }
}

impl std::error::Error for ValidatorProofConsensusError {}

pub(super) fn require_non_empty_consensus_field(
    field: &'static str,
    value: &str,
) -> Result<(), ValidatorProofConsensusError> {
    if value.trim().is_empty() {
        return Err(ValidatorProofConsensusError::InvalidField { field });
    }
    Ok(())
}

pub(super) fn distinct_bucket_count(valid: usize, invalid: usize, replay: usize) -> usize {
    [valid, invalid, replay]
        .into_iter()
        .filter(|count| *count > 0)
        .count()
}

pub(super) fn consensus_status(
    valid: usize,
    invalid: usize,
    replay: usize,
) -> ValidatorProofConsensusStatus {
    if distinct_bucket_count(valid, invalid, replay) > 1 {
        ValidatorProofConsensusStatus::ValidatorMismatch
    } else if valid > 0 {
        ValidatorProofConsensusStatus::ConsensusValid
    } else if invalid > 0 {
        ValidatorProofConsensusStatus::ConsensusInvalid
    } else {
        ValidatorProofConsensusStatus::ConsensusReplay
    }
}
