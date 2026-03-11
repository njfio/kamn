use crate::MessageEnvelopeError;
use std::fmt;

/// Errors emitted by proof-option evaluation, witness generation, and proof-admission flows.
/// Design-time validation errors for zero-knowledge message proof planning and admission.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ZkDesignError {
    /// The selected policy is semantically invalid.
    InvalidPolicy(String),
    /// One option field failed validation.
    InvalidOption { option: String, reason: String },
    /// No options were provided for evaluation.
    EmptyOptionSet,
    /// Ranking produced an impossible or unstable result.
    RankingInvariantViolated,
    /// A declared private field is invalid for the source message.
    InvalidPrivateField(String),
    /// A declared private field is missing from the source message.
    MissingPrivateField(String),
    /// A proof artifact payload is malformed.
    InvalidProofArtifact(String),
    /// The proof artifact references a different message than expected.
    ProofArtifactMessageMismatch { expected: String, found: String },
    /// The proof artifact commitment does not match the expected commitment.
    ProofArtifactCommitmentMismatch { expected: String, found: String },
    /// The same proof artifact was replayed.
    ProofArtifactReplay(String),
    /// Proof verification failed for the supplied artifact.
    ProofVerificationFailed { artifact_id: String, reason: String },
    /// Canonical envelope parsing failed before proof evaluation.
    EnvelopeError(MessageEnvelopeError),
}

impl fmt::Display for ZkDesignError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.render_message())
    }
}

impl std::error::Error for ZkDesignError {}

impl ZkDesignError {
    fn render_message(&self) -> String {
        match self {
            Self::InvalidPolicy(message) => format!("invalid policy: {message}"),
            Self::InvalidOption { option, reason } => invalid_option_message(option, reason),
            Self::EmptyOptionSet => "at least one architecture option is required".to_owned(),
            Self::RankingInvariantViolated => ranking_invariant_message(),
            Self::InvalidPrivateField(message) => format!("invalid private field: {message}"),
            Self::MissingPrivateField(field) => missing_private_field_message(field),
            Self::InvalidProofArtifact(message) => format!("invalid proof artifact: {message}"),
            Self::ProofArtifactMessageMismatch { expected, found } => {
                artifact_mismatch_message("message", expected, found)
            }
            Self::ProofArtifactCommitmentMismatch { expected, found } => {
                artifact_mismatch_message("commitment", expected, found)
            }
            Self::ProofArtifactReplay(artifact_id) => {
                format!("proof artifact replay detected: {artifact_id}")
            }
            Self::ProofVerificationFailed {
                artifact_id,
                reason,
            } => verification_failed_message(artifact_id, reason),
            Self::EnvelopeError(error) => format!("invalid canonical envelope: {error}"),
        }
    }
}

fn invalid_option_message(option: &str, reason: &str) -> String {
    format!("invalid option `{option}`: {reason}")
}

fn ranking_invariant_message() -> String {
    "non-empty architecture option set did not produce a ranked recommendation".to_owned()
}

fn missing_private_field_message(field: &str) -> String {
    format!("private field `{field}` is missing from envelope body payload")
}

fn artifact_mismatch_message(kind: &str, expected: &str, found: &str) -> String {
    format!("proof artifact {kind} mismatch: expected {expected}, found {found}")
}

fn verification_failed_message(artifact_id: &str, reason: &str) -> String {
    format!("proof verification failed for artifact {artifact_id}: {reason}")
}

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
