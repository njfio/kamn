use super::{ApproverAttestation, ApproverQuorumError};

#[derive(Debug, Clone, PartialEq, Eq)]
/// Approver quorum input.
pub struct ApproverQuorumInput {
    action_id: String,
    payload_digest: String,
    attestations: Vec<ApproverAttestation>,
}

impl ApproverQuorumInput {
    /// Handles new.
    pub fn new(
        action_id: &str,
        payload_digest: &str,
        attestations: Vec<ApproverAttestation>,
    ) -> Result<Self, ApproverQuorumError> {
        if action_id.trim().is_empty() {
            return Err(ApproverQuorumError::InvalidActionId);
        }
        if payload_digest.trim().is_empty() {
            return Err(ApproverQuorumError::InvalidPayloadDigest);
        }
        Ok(Self {
            action_id: action_id.to_owned(),
            payload_digest: payload_digest.to_owned(),
            attestations,
        })
    }

    /// Handles action id.
    pub fn action_id(&self) -> &str {
        &self.action_id
    }

    /// Handles payload digest.
    pub fn payload_digest(&self) -> &str {
        &self.payload_digest
    }

    /// Handles attestations.
    pub fn attestations(&self) -> &[ApproverAttestation] {
        &self.attestations
    }
}
