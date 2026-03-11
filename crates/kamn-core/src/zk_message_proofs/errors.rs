use crate::MessageEnvelopeError;
use std::fmt;

/// Errors emitted by proof-option evaluation, witness generation, and proof-admission flows.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ZkDesignError {
    InvalidPolicy(String),
    InvalidOption { option: String, reason: String },
    EmptyOptionSet,
    RankingInvariantViolated,
    InvalidPrivateField(String),
    MissingPrivateField(String),
    InvalidProofArtifact(String),
    ProofArtifactMessageMismatch { expected: String, found: String },
    ProofArtifactCommitmentMismatch { expected: String, found: String },
    ProofArtifactReplay(String),
    ProofVerificationFailed { artifact_id: String, reason: String },
    EnvelopeError(MessageEnvelopeError),
}

impl fmt::Display for ZkDesignError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidPolicy(message) => write!(f, "invalid policy: {message}"),
            Self::InvalidOption { option, reason } => {
                write!(f, "invalid option `{option}`: {reason}")
            }
            Self::EmptyOptionSet => write!(f, "at least one architecture option is required"),
            Self::RankingInvariantViolated => write!(
                f,
                "non-empty architecture option set did not produce a ranked recommendation"
            ),
            Self::InvalidPrivateField(message) => write!(f, "invalid private field: {message}"),
            Self::MissingPrivateField(field) => {
                write!(
                    f,
                    "private field `{field}` is missing from envelope body payload"
                )
            }
            Self::InvalidProofArtifact(message) => write!(f, "invalid proof artifact: {message}"),
            Self::ProofArtifactMessageMismatch { expected, found } => {
                write!(
                    f,
                    "proof artifact message mismatch: expected {expected}, found {found}"
                )
            }
            Self::ProofArtifactCommitmentMismatch { expected, found } => write!(
                f,
                "proof artifact commitment mismatch: expected {expected}, found {found}"
            ),
            Self::ProofArtifactReplay(artifact_id) => {
                write!(f, "proof artifact replay detected: {artifact_id}")
            }
            Self::ProofVerificationFailed {
                artifact_id,
                reason,
            } => {
                write!(
                    f,
                    "proof verification failed for artifact {artifact_id}: {reason}"
                )
            }
            Self::EnvelopeError(error) => write!(f, "invalid canonical envelope: {error}"),
        }
    }
}

impl std::error::Error for ZkDesignError {}

pub(crate) fn require_non_empty_artifact_field(
    field: &str,
    value: &str,
) -> Result<(), ZkDesignError> {
    if value.trim().is_empty() {
        return Err(ZkDesignError::InvalidProofArtifact(format!(
            "{field} must not be empty"
        )));
    }
    Ok(())
}
