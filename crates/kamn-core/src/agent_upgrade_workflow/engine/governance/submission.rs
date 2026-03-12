use crate::{GovernanceProposalDraft, GovernanceProposalStatus};

use crate::agent_upgrade_workflow::{
    AgentDrivenUpgradeWorkflow, AgentUpgradeAuditEvent, AgentUpgradeAuditEventKind,
    AgentUpgradeProposalRecord, AgentUpgradeProposalState, AgentUpgradeWorkflowError,
    support::validate_timestamp,
};

impl AgentDrivenUpgradeWorkflow {
    /// Promote a reviewed proposal into governance voting.
    pub fn submit_to_governance(
        &mut self,
        proposal_id: &str,
        submitted_at_unix: u64,
    ) -> Result<(), AgentUpgradeWorkflowError> {
        validate_timestamp("submitted_at_unix", submitted_at_unix)?;
        let mut proposal = load_proposal_for_submission(self, proposal_id)?;
        validate_governance_submission(self, &proposal, proposal_id, submitted_at_unix)?;
        submit_governance_draft(self, &proposal, submitted_at_unix)?;
        record_submission_state(&mut proposal);
        self.proposals.insert(proposal_id.to_owned(), proposal);
        self.events
            .push(governance_submission_event(proposal_id, submitted_at_unix));
        Ok(())
    }
}

fn load_proposal_for_submission(
    workflow: &AgentDrivenUpgradeWorkflow,
    proposal_id: &str,
) -> Result<AgentUpgradeProposalRecord, AgentUpgradeWorkflowError> {
    workflow
        .proposals
        .get(proposal_id)
        .cloned()
        .ok_or_else(|| AgentUpgradeWorkflowError::ProposalNotFound(proposal_id.to_owned()))
}

fn submit_governance_draft(
    workflow: &mut AgentDrivenUpgradeWorkflow,
    proposal: &AgentUpgradeProposalRecord,
    submitted_at_unix: u64,
) -> Result<(), AgentUpgradeWorkflowError> {
    workflow
        .governance
        .submit_proposal(governance_draft(
            proposal,
            submitted_at_unix,
            workflow.required_validator_quorum,
        ))
        .map_err(AgentUpgradeWorkflowError::GovernanceWorkflow)
}

fn record_submission_state(proposal: &mut AgentUpgradeProposalRecord) {
    proposal.state = AgentUpgradeProposalState::GovernanceVoting;
    proposal.governance_status = GovernanceProposalStatus::Voting;
    proposal.governance_approved_at_unix = None;
}

fn governance_submission_event(
    proposal_id: &str,
    submitted_at_unix: u64,
) -> AgentUpgradeAuditEvent {
    AgentUpgradeAuditEvent {
        proposal_id: proposal_id.to_owned(),
        actor_did: "workflow".to_owned(),
        event_at_unix: submitted_at_unix,
        kind: AgentUpgradeAuditEventKind::GovernanceSubmitted,
        note: Some("proposal promoted to governance workflow".to_owned()),
    }
}

fn validate_governance_submission(
    workflow: &AgentDrivenUpgradeWorkflow,
    proposal: &AgentUpgradeProposalRecord,
    proposal_id: &str,
    submitted_at_unix: u64,
) -> Result<(), AgentUpgradeWorkflowError> {
    validate_submission_state(proposal, proposal_id)?;
    validate_review_threshold(workflow, proposal)?;
    validate_submission_deadline(proposal, submitted_at_unix)
}

fn validate_submission_state(
    proposal: &AgentUpgradeProposalRecord,
    proposal_id: &str,
) -> Result<(), AgentUpgradeWorkflowError> {
    if proposal.state != AgentUpgradeProposalState::PendingHumanReview {
        return Err(AgentUpgradeWorkflowError::GovernanceSubmissionNotAllowed {
            proposal_id: proposal_id.to_owned(),
            state: proposal.state,
        });
    }
    Ok(())
}

fn validate_review_threshold(
    workflow: &AgentDrivenUpgradeWorkflow,
    proposal: &AgentUpgradeProposalRecord,
) -> Result<(), AgentUpgradeWorkflowError> {
    let provided_reviews = proposal.human_reviewers.len();
    if provided_reviews < workflow.required_human_reviews {
        return Err(AgentUpgradeWorkflowError::InsufficientHumanReviews {
            required: workflow.required_human_reviews,
            provided: provided_reviews,
        });
    }
    Ok(())
}

fn validate_submission_deadline(
    proposal: &AgentUpgradeProposalRecord,
    submitted_at_unix: u64,
) -> Result<(), AgentUpgradeWorkflowError> {
    if proposal.voting_deadline_unix <= submitted_at_unix {
        return Err(AgentUpgradeWorkflowError::InvalidDeadline {
            created_at_unix: submitted_at_unix,
            voting_deadline_unix: proposal.voting_deadline_unix,
        });
    }
    Ok(())
}

fn governance_draft(
    proposal: &AgentUpgradeProposalRecord,
    submitted_at_unix: u64,
    quorum: usize,
) -> GovernanceProposalDraft {
    GovernanceProposalDraft {
        proposal_id: proposal.proposal_id.clone(),
        title: format!("Agent-driven upgrade to {}", proposal.target_version),
        description: proposal.rationale.clone(),
        proposer_did: proposal.agent_did.clone(),
        created_at_unix: submitted_at_unix,
        voting_deadline_unix: proposal.voting_deadline_unix,
        quorum_threshold: quorum,
        parameter_change: None,
    }
}
