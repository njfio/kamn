use super::did::parse_agent_did;
use super::error::GovernanceWorkflowError;
use super::models::{
    GovernanceExecutionRecord, GovernanceProposalDraft, GovernanceProposalRecord,
    GovernanceProposalStatus, GovernanceVoteChoice, GovernanceVoteRecord,
};
use super::parameter_policy::{require_non_empty, validate_parameter_change};
use super::state::{reevaluate_status, GovernanceProposalState, GovernanceWorkflow};
use std::collections::BTreeMap;

const GOVERNANCE_WORKFLOW_INVALID_PROPOSER_DID_REASON_CODE: &str =
    "governance_workflow_invalid_proposer_did";
const GOVERNANCE_WORKFLOW_INVALID_VOTER_DID_REASON_CODE: &str =
    "governance_workflow_invalid_voter_did";
const GOVERNANCE_WORKFLOW_INVALID_EXECUTOR_DID_REASON_CODE: &str =
    "governance_workflow_invalid_executor_did";

impl GovernanceWorkflow {
    /// Submit a proposal draft into the governance workflow.
    pub fn submit_proposal(
        &mut self,
        draft: GovernanceProposalDraft,
    ) -> Result<(), GovernanceWorkflowError> {
        require_non_empty("proposal_id", &draft.proposal_id)?;
        require_non_empty("title", &draft.title)?;
        require_non_empty("description", &draft.description)?;
        let proposer_did = parse_agent_did(
            draft.proposer_did.as_str(),
            "proposer_did",
            GOVERNANCE_WORKFLOW_INVALID_PROPOSER_DID_REASON_CODE,
        )?;
        if draft.created_at_unix == 0 {
            return Err(GovernanceWorkflowError::InvalidTimestamp("created_at_unix"));
        }
        if draft.voting_deadline_unix <= draft.created_at_unix {
            return Err(GovernanceWorkflowError::InvalidDeadline {
                created_at_unix: draft.created_at_unix,
                voting_deadline_unix: draft.voting_deadline_unix,
            });
        }
        if draft.quorum_threshold == 0 {
            return Err(GovernanceWorkflowError::InvalidQuorum(0));
        }
        if let Some(parameter_change) = &draft.parameter_change {
            validate_parameter_change(parameter_change)?;
        }
        if self.proposals.contains_key(&draft.proposal_id) {
            return Err(GovernanceWorkflowError::DuplicateProposal(
                draft.proposal_id.clone(),
            ));
        }
        self.proposals.insert(
            draft.proposal_id.clone(),
            GovernanceProposalState {
                record: GovernanceProposalRecord {
                    proposal_id: draft.proposal_id,
                    title: draft.title,
                    description: draft.description,
                    proposer_did: proposer_did.as_str().to_owned(),
                    created_at_unix: draft.created_at_unix,
                    voting_deadline_unix: draft.voting_deadline_unix,
                    quorum_threshold: draft.quorum_threshold,
                    parameter_change: draft.parameter_change,
                    status: GovernanceProposalStatus::Voting,
                    yes_votes: 0,
                    no_votes: 0,
                    abstain_votes: 0,
                    executed_at_unix: None,
                },
                votes: BTreeMap::new(),
            },
        );
        Ok(())
    }

    /// Cast a vote for a proposal while it remains in the voting window.
    pub fn cast_vote(
        &mut self,
        proposal_id: &str,
        voter_did: &str,
        choice: GovernanceVoteChoice,
        cast_at_unix: u64,
    ) -> Result<(), GovernanceWorkflowError> {
        let state = self
            .proposals
            .get_mut(proposal_id)
            .ok_or_else(|| GovernanceWorkflowError::ProposalNotFound(proposal_id.to_owned()))?;
        if cast_at_unix == 0 {
            return Err(GovernanceWorkflowError::InvalidTimestamp("cast_at_unix"));
        }
        let voter_did = parse_agent_did(
            voter_did,
            "voter_did",
            GOVERNANCE_WORKFLOW_INVALID_VOTER_DID_REASON_CODE,
        )?;
        if state.record.status != GovernanceProposalStatus::Voting {
            return Err(GovernanceWorkflowError::ProposalClosed {
                proposal_id: proposal_id.to_owned(),
                status: state.record.status,
            });
        }
        if cast_at_unix > state.record.voting_deadline_unix {
            state.record.status = GovernanceProposalStatus::Expired;
            return Err(GovernanceWorkflowError::ProposalClosed {
                proposal_id: proposal_id.to_owned(),
                status: state.record.status,
            });
        }
        if state.votes.contains_key(voter_did.as_str()) {
            return Err(GovernanceWorkflowError::DuplicateVote {
                proposal_id: proposal_id.to_owned(),
                voter_did: voter_did.as_str().to_owned(),
            });
        }
        state.votes.insert(
            voter_did.as_str().to_owned(),
            GovernanceVoteRecord {
                proposal_id: proposal_id.to_owned(),
                voter_did: voter_did.as_str().to_owned(),
                choice,
                cast_at_unix,
            },
        );
        match choice {
            GovernanceVoteChoice::Yes => state.record.yes_votes += 1,
            GovernanceVoteChoice::No => state.record.no_votes += 1,
            GovernanceVoteChoice::Abstain => state.record.abstain_votes += 1,
        }
        reevaluate_status(&mut state.record, cast_at_unix);
        Ok(())
    }

    /// Reevaluate proposal status using current tallies and deadline checks.
    pub fn evaluate(
        &mut self,
        proposal_id: &str,
        evaluated_at_unix: u64,
    ) -> Result<GovernanceProposalStatus, GovernanceWorkflowError> {
        if evaluated_at_unix == 0 {
            return Err(GovernanceWorkflowError::InvalidTimestamp(
                "evaluated_at_unix",
            ));
        }
        let state = self
            .proposals
            .get_mut(proposal_id)
            .ok_or_else(|| GovernanceWorkflowError::ProposalNotFound(proposal_id.to_owned()))?;
        reevaluate_status(&mut state.record, evaluated_at_unix);
        Ok(state.record.status)
    }

    /// Execute an approved proposal and emit an execution record.
    pub fn execute(
        &mut self,
        proposal_id: &str,
        executed_by: &str,
        executed_at_unix: u64,
        operation_hash: &str,
    ) -> Result<GovernanceExecutionRecord, GovernanceWorkflowError> {
        if executed_at_unix == 0 {
            return Err(GovernanceWorkflowError::InvalidTimestamp(
                "executed_at_unix",
            ));
        }
        let executed_by = parse_agent_did(
            executed_by,
            "executed_by",
            GOVERNANCE_WORKFLOW_INVALID_EXECUTOR_DID_REASON_CODE,
        )?;
        require_non_empty("operation_hash", operation_hash)?;
        let state = self
            .proposals
            .get_mut(proposal_id)
            .ok_or_else(|| GovernanceWorkflowError::ProposalNotFound(proposal_id.to_owned()))?;
        if state.record.status == GovernanceProposalStatus::Executed {
            return Err(GovernanceWorkflowError::AlreadyExecuted(
                proposal_id.to_owned(),
            ));
        }
        reevaluate_status(&mut state.record, executed_at_unix);
        if state.record.status != GovernanceProposalStatus::Approved {
            return Err(GovernanceWorkflowError::ProposalNotApproved {
                proposal_id: proposal_id.to_owned(),
                status: state.record.status,
            });
        }
        state.record.status = GovernanceProposalStatus::Executed;
        state.record.executed_at_unix = Some(executed_at_unix);
        let record = GovernanceExecutionRecord {
            proposal_id: proposal_id.to_owned(),
            executed_by: executed_by.as_str().to_owned(),
            executed_at_unix,
            operation_hash: operation_hash.to_owned(),
        };
        self.execution_history.push(record.clone());
        Ok(record)
    }
}
