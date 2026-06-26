use std::error::Error;
use std::fmt::{Display, Formatter};

#[derive(Debug, Clone, PartialEq, Eq)]
/// Approver quorum error.
pub enum ApproverQuorumError {
    /// Invalid required approvals.
    /// Invalid required approvals variant for this public contract enum.
    InvalidRequiredApprovals {
        /// Required carried by this enum variant.
        required: usize,
    },
    /// Invalid action id.
    InvalidActionId,
    /// Invalid payload digest.
    InvalidPayloadDigest,
    /// Invalid approver did.
    InvalidApproverDid {
        /// Str carried by this public contract model.
        field: &'static str,
        /// Str carried by this public contract model.
        reason_code: &'static str,
        /// String carried by this public contract model.
        detail: String,
    },
    /// Invalid attestation id.
    InvalidAttestationId,
    /// Duplicate approver attestation.
    /// Duplicate approver attestation variant for this public contract enum.
    DuplicateApproverAttestation {
        /// Approver did carried by this enum variant.
        approver_did: String,
    },
    /// Payload digest mismatch.
    /// Payload digest mismatch variant for this public contract enum.
    PayloadDigestMismatch {
        /// Expected carried by this enum variant.
        expected: String,
        /// Found carried by this enum variant.
        found: String,
    },
    /// Insufficient approvals.
    /// Insufficient approvals variant for this public contract enum.
    InsufficientApprovals {
        /// Required carried by this enum variant.
        required: usize,
        /// Received carried by this enum variant.
        received: usize,
    },
}

impl Display for ApproverQuorumError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidRequiredApprovals { required } => {
                write!(f, "invalid approver quorum requirement: {required}")
            }
            Self::InvalidActionId => write!(f, "approver quorum action id cannot be empty"),
            Self::InvalidPayloadDigest => {
                write!(f, "approver quorum payload digest cannot be empty")
            }
            Self::InvalidApproverDid {
                field,
                reason_code,
                detail,
            } => write!(f, "invalid did field {field}: {reason_code} ({detail})"),
            Self::InvalidAttestationId => write!(f, "approver attestation id cannot be empty"),
            Self::DuplicateApproverAttestation { approver_did } => write!(
                f,
                "duplicate approver attestation replay detected for {approver_did}"
            ),
            Self::PayloadDigestMismatch { expected, found } => write!(
                f,
                "approver payload digest mismatch: expected {expected}, found {found}"
            ),
            Self::InsufficientApprovals { required, received } => write!(
                f,
                "approver quorum insufficient approvals: required {required}, received {received}"
            ),
        }
    }
}

impl Error for ApproverQuorumError {}
