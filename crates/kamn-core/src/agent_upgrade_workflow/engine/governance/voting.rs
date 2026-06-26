use crate::{GovernanceProposalStatus, GovernanceVoteChoice};

use crate::agent_upgrade_workflow::{
    support::{validate_did, AGENT_UPGRADE_WORKFLOW_INVALID_VALIDATOR_DID_REASON_CODE},
    AgentDrivenUpgradeWorkflow, AgentUpgradeAuditEvent, AgentUpgradeAuditEventKind,
    AgentUpgradeProposalRecord, AgentUpgradeProposalState, AgentUpgradeWorkflowError,
};

impl AgentDrivenUpgradeWorkflow {
    /// Cast a governance vote from an allowlisted validator and update mirrored proposal state.
    pub fn cast_validator_vote(
        &mut self,
        proposal_id: &str,
        validator_did: &str,
        choice: GovernanceVoteChoice,
        cast_at_unix: u64,
    ) -> Result<(), AgentUpgradeWorkflowError> {
        ensure_authorized_validator(self, validator_did)?;
        let status =
            cast_and_evaluate_vote(self, proposal_id, validator_did, choice, cast_at_unix)?;
        if let Some(record) = self.proposals.get_mut(proposal_id) {
            apply_vote_status(record, status, cast_at_unix);
        }
        if status == GovernanceProposalStatus::Approved {
            self.events.push(governance_approved_event(
                proposal_id,
                validator_did,
                cast_at_unix,
            ));
        }
        Ok(())
    }
}

fn ensure_authorized_validator(
    workflow: &AgentDrivenUpgradeWorkflow,
    validator_did: &str,
) -> Result<(), AgentUpgradeWorkflowError> {
    validate_did(
        validator_did,
        "validator_did",
        AGENT_UPGRADE_WORKFLOW_INVALID_VALIDATOR_DID_REASON_CODE,
    )?;
    if !workflow.allowed_validator_voters.contains(validator_did) {
        return Err(AgentUpgradeWorkflowError::UnauthorizedValidatorVoter(
            validator_did.to_owned(),
        ));
    }
    Ok(())
}

fn cast_and_evaluate_vote(
    workflow: &mut AgentDrivenUpgradeWorkflow,
    proposal_id: &str,
    validator_did: &str,
    choice: GovernanceVoteChoice,
    cast_at_unix: u64,
) -> Result<GovernanceProposalStatus, AgentUpgradeWorkflowError> {
    workflow
        .governance
        .cast_vote(proposal_id, validator_did, choice, cast_at_unix)
        .map_err(AgentUpgradeWorkflowError::GovernanceWorkflow)?;
    workflow
        .governance
        .evaluate(proposal_id, cast_at_unix)
        .map_err(AgentUpgradeWorkflowError::GovernanceWorkflow)
}

fn apply_vote_status(
    record: &mut AgentUpgradeProposalRecord,
    status: GovernanceProposalStatus,
    cast_at_unix: u64,
) {
    record.governance_status = status;
    if status == GovernanceProposalStatus::Approved {
        record.state = AgentUpgradeProposalState::GovernanceApproved;
        if record.governance_approved_at_unix.is_none() {
            record.governance_approved_at_unix = Some(cast_at_unix);
        }
    }
}

fn governance_approved_event(
    proposal_id: &str,
    validator_did: &str,
    cast_at_unix: u64,
) -> AgentUpgradeAuditEvent {
    AgentUpgradeAuditEvent {
        proposal_id: proposal_id.to_owned(),
        actor_did: validator_did.to_owned(),
        event_at_unix: cast_at_unix,
        kind: AgentUpgradeAuditEventKind::GovernanceApproved,
        note: Some("governance quorum reached".to_owned()),
    }
}
