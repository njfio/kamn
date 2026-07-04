use super::super::did_validation::parse_approver_did;
use super::ApproverQuorumError;

#[derive(Debug, Clone, PartialEq, Eq)]
/// Approver attestation.
pub struct ApproverAttestation {
    approver_did: String,
    payload_digest: String,
    attestation_id: String,
}

impl ApproverAttestation {
    /// Handles new.
    pub fn new(
        approver_did: &str,
        payload_digest: &str,
        attestation_id: &str,
    ) -> Result<Self, ApproverQuorumError> {
        parse_approver_did(approver_did, "approver_did")?;
        validate_attestation_fields(payload_digest, attestation_id)?;
        Ok(Self {
            approver_did: approver_did.to_owned(),
            payload_digest: payload_digest.to_owned(),
            attestation_id: attestation_id.to_owned(),
        })
    }

    /// Handles approver did.
    pub fn approver_did(&self) -> &str {
        &self.approver_did
    }

    /// Handles payload digest.
    pub fn payload_digest(&self) -> &str {
        &self.payload_digest
    }
}

fn validate_attestation_fields(
    payload_digest: &str,
    attestation_id: &str,
) -> Result<(), ApproverQuorumError> {
    if payload_digest.trim().is_empty() {
        return Err(ApproverQuorumError::InvalidPayloadDigest);
    }
    if attestation_id.trim().is_empty() {
        return Err(ApproverQuorumError::InvalidAttestationId);
    }
    Ok(())
}
