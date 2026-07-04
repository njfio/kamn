use std::collections::BTreeSet;

use super::{ApproverQuorumDecision, ApproverQuorumError, ApproverQuorumInput};
use crate::runtime::runtime_phase_coordination::did_validation::parse_approver_did;

#[derive(Debug, Clone, PartialEq, Eq)]
/// Approver quorum evaluator.
pub struct ApproverQuorumEvaluator {
    required_approvals: usize,
}

impl ApproverQuorumEvaluator {
    /// Handles new.
    pub fn new(required_approvals: usize) -> Result<Self, ApproverQuorumError> {
        if required_approvals == 0 {
            return Err(ApproverQuorumError::InvalidRequiredApprovals {
                required: required_approvals,
            });
        }
        Ok(Self { required_approvals })
    }

    /// Handles authorize.
    pub fn authorize(
        &self,
        input: ApproverQuorumInput,
    ) -> Result<ApproverQuorumDecision, ApproverQuorumError> {
        let approved_by = collect_approvals(&input)?;
        if approved_by.len() < self.required_approvals {
            return Err(ApproverQuorumError::InsufficientApprovals {
                required: self.required_approvals,
                received: approved_by.len(),
            });
        }
        Ok(ApproverQuorumDecision {
            action_id: input.action_id().to_owned(),
            required_approvals: self.required_approvals,
            approved_by,
            authorized: true,
        })
    }
}

/// Handles authorize daemon outbound action.
pub fn authorize_daemon_outbound_action(
    evaluator: &ApproverQuorumEvaluator,
    input: ApproverQuorumInput,
) -> Result<ApproverQuorumDecision, ApproverQuorumError> {
    evaluator.authorize(input)
}

fn collect_approvals(input: &ApproverQuorumInput) -> Result<Vec<String>, ApproverQuorumError> {
    let mut approved = BTreeSet::new();
    for attestation in input.attestations() {
        parse_approver_did(attestation.approver_did(), "attestations[].approver_did")?;
        validate_payload_digest(input, attestation.payload_digest())?;
        if !approved.insert(attestation.approver_did().to_owned()) {
            return Err(ApproverQuorumError::DuplicateApproverAttestation {
                approver_did: attestation.approver_did().to_owned(),
            });
        }
    }
    Ok(approved.into_iter().collect())
}

fn validate_payload_digest(
    input: &ApproverQuorumInput,
    attestation_digest: &str,
) -> Result<(), ApproverQuorumError> {
    if attestation_digest != input.payload_digest() {
        return Err(ApproverQuorumError::PayloadDigestMismatch {
            expected: input.payload_digest().to_owned(),
            found: attestation_digest.to_owned(),
        });
    }
    Ok(())
}
