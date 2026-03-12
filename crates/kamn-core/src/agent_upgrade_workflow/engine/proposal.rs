use crate::agent_upgrade_workflow::{
    AgentDrivenUpgradeWorkflow, AgentUpgradeAuditEvent, AgentUpgradeAuditEventKind,
    AgentUpgradeProposalDraft, AgentUpgradeProposalRecord, AgentUpgradeProposalState,
    AgentUpgradeWorkflowError,
    support::{
        AGENT_UPGRADE_WORKFLOW_INVALID_PROPOSAL_AGENT_DID_REASON_CODE,
        AGENT_UPGRADE_WORKFLOW_INVALID_REVIEWER_DID_REASON_CODE, require_non_empty, validate_did,
        validate_timestamp,
    },
};

impl AgentDrivenUpgradeWorkflow {
    /// Register a new proposal from an authorized agent and seed upgrade orchestration state.
    pub fn submit_agent_proposal(
        &mut self,
        draft: AgentUpgradeProposalDraft,
    ) -> Result<(), AgentUpgradeWorkflowError> {
        validate_proposal_draft(self, &draft)?;
        self.orchestrator
            .propose_upgrade(
                &draft.proposal_id,
                &draft.target_version,
                &draft.agent_did,
                self.required_validator_quorum,
                draft.created_at_unix,
            )
            .map_err(AgentUpgradeWorkflowError::UpgradeOrchestration)?;
        self.proposals
            .insert(draft.proposal_id.clone(), proposal_record(&draft));
        self.events.push(proposal_submission_event(&draft));
        Ok(())
    }

    /// Record a distinct human-review approval for a pending proposal.
    pub fn approve_human_review(
        &mut self,
        proposal_id: &str,
        reviewer_did: &str,
        reviewed_at_unix: u64,
    ) -> Result<(), AgentUpgradeWorkflowError> {
        authorize_human_reviewer(self, reviewer_did, reviewed_at_unix)?;
        record_human_review(self, proposal_id, reviewer_did)?;
        self.events.push(human_review_event(
            proposal_id,
            reviewer_did,
            reviewed_at_unix,
        ));
        Ok(())
    }
}

fn authorize_human_reviewer(
    workflow: &AgentDrivenUpgradeWorkflow,
    reviewer_did: &str,
    reviewed_at_unix: u64,
) -> Result<(), AgentUpgradeWorkflowError> {
    validate_did(
        reviewer_did,
        "reviewer_did",
        AGENT_UPGRADE_WORKFLOW_INVALID_REVIEWER_DID_REASON_CODE,
    )?;
    validate_timestamp("reviewed_at_unix", reviewed_at_unix)?;
    if !workflow.allowed_validator_voters.contains(reviewer_did) {
        return Err(AgentUpgradeWorkflowError::UnauthorizedHumanReviewer(
            reviewer_did.to_owned(),
        ));
    }
    Ok(())
}

fn record_human_review(
    workflow: &mut AgentDrivenUpgradeWorkflow,
    proposal_id: &str,
    reviewer_did: &str,
) -> Result<(), AgentUpgradeWorkflowError> {
    let proposal = workflow
        .proposals
        .get_mut(proposal_id)
        .ok_or_else(|| AgentUpgradeWorkflowError::ProposalNotFound(proposal_id.to_owned()))?;
    if !proposal.human_reviewers.insert(reviewer_did.to_owned()) {
        return Err(AgentUpgradeWorkflowError::DuplicateHumanReview {
            proposal_id: proposal_id.to_owned(),
            reviewer_did: reviewer_did.to_owned(),
        });
    }
    Ok(())
}

fn proposal_submission_event(draft: &AgentUpgradeProposalDraft) -> AgentUpgradeAuditEvent {
    AgentUpgradeAuditEvent {
        proposal_id: draft.proposal_id.clone(),
        actor_did: draft.agent_did.clone(),
        event_at_unix: draft.created_at_unix,
        kind: AgentUpgradeAuditEventKind::AgentProposed,
        note: Some("agent proposal submitted".to_owned()),
    }
}

fn human_review_event(
    proposal_id: &str,
    reviewer_did: &str,
    reviewed_at_unix: u64,
) -> AgentUpgradeAuditEvent {
    AgentUpgradeAuditEvent {
        proposal_id: proposal_id.to_owned(),
        actor_did: reviewer_did.to_owned(),
        event_at_unix: reviewed_at_unix,
        kind: AgentUpgradeAuditEventKind::HumanReviewApproved,
        note: Some("human review approval registered".to_owned()),
    }
}

fn validate_proposal_draft(
    workflow: &AgentDrivenUpgradeWorkflow,
    draft: &AgentUpgradeProposalDraft,
) -> Result<(), AgentUpgradeWorkflowError> {
    validate_draft_fields(draft)?;
    validate_draft_deadline(draft)?;
    validate_proposer_authorization(workflow, draft)?;
    ensure_unique_proposal(workflow, draft)
}

fn validate_draft_fields(
    draft: &AgentUpgradeProposalDraft,
) -> Result<(), AgentUpgradeWorkflowError> {
    require_non_empty("proposal_id", &draft.proposal_id)?;
    require_non_empty("rationale", &draft.rationale)?;
    validate_did(
        &draft.agent_did,
        "proposal.agent_did",
        AGENT_UPGRADE_WORKFLOW_INVALID_PROPOSAL_AGENT_DID_REASON_CODE,
    )?;
    validate_timestamp("created_at_unix", draft.created_at_unix)
}

fn validate_draft_deadline(
    draft: &AgentUpgradeProposalDraft,
) -> Result<(), AgentUpgradeWorkflowError> {
    if draft.voting_deadline_unix <= draft.created_at_unix {
        return Err(AgentUpgradeWorkflowError::InvalidDeadline {
            created_at_unix: draft.created_at_unix,
            voting_deadline_unix: draft.voting_deadline_unix,
        });
    }
    Ok(())
}

fn validate_proposer_authorization(
    workflow: &AgentDrivenUpgradeWorkflow,
    draft: &AgentUpgradeProposalDraft,
) -> Result<(), AgentUpgradeWorkflowError> {
    if !workflow.allowed_agent_proposers.contains(&draft.agent_did) {
        return Err(AgentUpgradeWorkflowError::UnauthorizedAgentProposer(
            draft.agent_did.clone(),
        ));
    }
    Ok(())
}

fn ensure_unique_proposal(
    workflow: &AgentDrivenUpgradeWorkflow,
    draft: &AgentUpgradeProposalDraft,
) -> Result<(), AgentUpgradeWorkflowError> {
    if workflow.proposals.contains_key(&draft.proposal_id) {
        return Err(AgentUpgradeWorkflowError::ProposalAlreadyExists(
            draft.proposal_id.clone(),
        ));
    }
    Ok(())
}

fn proposal_record(draft: &AgentUpgradeProposalDraft) -> AgentUpgradeProposalRecord {
    AgentUpgradeProposalRecord {
        proposal_id: draft.proposal_id.clone(),
        target_version: draft.target_version.clone(),
        agent_did: draft.agent_did.clone(),
        rationale: draft.rationale.clone(),
        created_at_unix: draft.created_at_unix,
        voting_deadline_unix: draft.voting_deadline_unix,
        human_reviewers: Default::default(),
        state: AgentUpgradeProposalState::PendingHumanReview,
        governance_status: crate::GovernanceProposalStatus::Voting,
        governance_approved_at_unix: None,
        activated_at_unix: None,
        operation_hash: None,
    }
}
