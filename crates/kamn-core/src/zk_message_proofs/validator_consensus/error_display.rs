use super::ValidatorProofConsensusError;
use std::fmt;

impl fmt::Display for ValidatorProofConsensusError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidRequiredQuorum(required_quorum) => write!(
                f,
                "validator proof required quorum must be greater than zero, found {required_quorum}"
            ),
            Self::InvalidField { field } => write!(f, "{field} must not be empty"),
            Self::EmptyAttestations => {
                write!(f, "validator proof attestations must not be empty")
            }
            Self::InvalidValidatorDid(value) => {
                write!(f, "validator proof attestation DID is invalid: {value}")
            }
            Self::AttestationMessageMismatch { expected, found } => write!(
                f,
                "validator attestation message mismatch: expected {expected}, found {found}"
            ),
            Self::AttestationArtifactMismatch { expected, found } => write!(
                f,
                "validator attestation artifact mismatch: expected {expected}, found {found}"
            ),
            Self::DuplicateValidator(validator_did) => write!(
                f,
                "validator proof attestation duplicate validator: {validator_did}"
            ),
            Self::DuplicateAttestationId(attestation_id) => write!(
                f,
                "validator proof attestation id duplicated in input: {attestation_id}"
            ),
            Self::AttestationReplay(attestation_id) => write!(
                f,
                "validator proof attestation replay detected: {attestation_id}"
            ),
            Self::InsufficientAttestations { required, received } => write!(
                f,
                "validator proof quorum insufficient attestations: required {required}, received {received}"
            ),
        }
    }
}

impl std::error::Error for ValidatorProofConsensusError {}
