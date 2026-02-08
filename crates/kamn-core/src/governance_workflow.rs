use crate::AgentDid;
use std::collections::BTreeMap;
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GovernanceProposalDraft {
    pub proposal_id: String,
    pub title: String,
    pub description: String,
    pub proposer_did: String,
    pub created_at_unix: u64,
    pub voting_deadline_unix: u64,
    pub quorum_threshold: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GovernanceVoteChoice {
    Yes,
    No,
    Abstain,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GovernanceProposalStatus {
    Voting,
    Approved,
    Rejected,
    Executed,
    Expired,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GovernanceVoteRecord {
    pub proposal_id: String,
    pub voter_did: String,
    pub choice: GovernanceVoteChoice,
    pub cast_at_unix: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GovernanceExecutionRecord {
    pub proposal_id: String,
    pub executed_by: String,
    pub executed_at_unix: u64,
    pub operation_hash: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GovernanceProposalRecord {
    pub proposal_id: String,
    pub title: String,
    pub description: String,
    pub proposer_did: String,
    pub created_at_unix: u64,
    pub voting_deadline_unix: u64,
    pub quorum_threshold: usize,
    pub status: GovernanceProposalStatus,
    pub yes_votes: usize,
    pub no_votes: usize,
    pub abstain_votes: usize,
    pub executed_at_unix: Option<u64>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct GovernanceWorkflow {
    proposals: BTreeMap<String, GovernanceProposalState>,
    execution_history: Vec<GovernanceExecutionRecord>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct GovernanceProposalState {
    record: GovernanceProposalRecord,
    votes: BTreeMap<String, GovernanceVoteRecord>,
}

impl GovernanceWorkflow {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn submit_proposal(
        &mut self,
        draft: GovernanceProposalDraft,
    ) -> Result<(), GovernanceWorkflowError> {
        require_non_empty("proposal_id", &draft.proposal_id)?;
        require_non_empty("title", &draft.title)?;
        require_non_empty("description", &draft.description)?;
        validate_did(&draft.proposer_did)?;
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
                    proposer_did: draft.proposer_did,
                    created_at_unix: draft.created_at_unix,
                    voting_deadline_unix: draft.voting_deadline_unix,
                    quorum_threshold: draft.quorum_threshold,
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
        validate_did(voter_did)?;
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
        if state.votes.contains_key(voter_did) {
            return Err(GovernanceWorkflowError::DuplicateVote {
                proposal_id: proposal_id.to_owned(),
                voter_did: voter_did.to_owned(),
            });
        }

        let vote = GovernanceVoteRecord {
            proposal_id: proposal_id.to_owned(),
            voter_did: voter_did.to_owned(),
            choice,
            cast_at_unix,
        };
        state.votes.insert(voter_did.to_owned(), vote);
        match choice {
            GovernanceVoteChoice::Yes => state.record.yes_votes += 1,
            GovernanceVoteChoice::No => state.record.no_votes += 1,
            GovernanceVoteChoice::Abstain => state.record.abstain_votes += 1,
        }
        reevaluate_status(&mut state.record, cast_at_unix);
        Ok(())
    }

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
        validate_did(executed_by)?;
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
            executed_by: executed_by.to_owned(),
            executed_at_unix,
            operation_hash: operation_hash.to_owned(),
        };
        self.execution_history.push(record.clone());
        Ok(record)
    }

    pub fn proposal(&self, proposal_id: &str) -> Option<GovernanceProposalRecord> {
        self.proposals
            .get(proposal_id)
            .map(|state| state.record.clone())
    }

    pub fn vote_history(
        &self,
        proposal_id: &str,
    ) -> Result<Vec<GovernanceVoteRecord>, GovernanceWorkflowError> {
        let state = self
            .proposals
            .get(proposal_id)
            .ok_or_else(|| GovernanceWorkflowError::ProposalNotFound(proposal_id.to_owned()))?;
        Ok(state.votes.values().cloned().collect())
    }

    pub fn execution_history(&self) -> Vec<GovernanceExecutionRecord> {
        self.execution_history.clone()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GovernanceWorkflowError {
    EmptyField(&'static str),
    InvalidDid(String),
    InvalidTimestamp(&'static str),
    InvalidDeadline {
        created_at_unix: u64,
        voting_deadline_unix: u64,
    },
    InvalidQuorum(usize),
    DuplicateProposal(String),
    ProposalNotFound(String),
    DuplicateVote {
        proposal_id: String,
        voter_did: String,
    },
    ProposalClosed {
        proposal_id: String,
        status: GovernanceProposalStatus,
    },
    ProposalNotApproved {
        proposal_id: String,
        status: GovernanceProposalStatus,
    },
    AlreadyExecuted(String),
}

impl fmt::Display for GovernanceWorkflowError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyField(field) => write!(f, "field must not be empty: {field}"),
            Self::InvalidDid(value) => write!(f, "invalid did: {value}"),
            Self::InvalidTimestamp(field) => write!(f, "timestamp must be > 0: {field}"),
            Self::InvalidDeadline {
                created_at_unix,
                voting_deadline_unix,
            } => write!(
                f,
                "invalid voting deadline: created_at_unix={created_at_unix}, voting_deadline_unix={voting_deadline_unix}"
            ),
            Self::InvalidQuorum(value) => write!(f, "invalid quorum threshold: {value}"),
            Self::DuplicateProposal(proposal_id) => {
                write!(f, "duplicate governance proposal id: {proposal_id}")
            }
            Self::ProposalNotFound(proposal_id) => {
                write!(f, "governance proposal not found: {proposal_id}")
            }
            Self::DuplicateVote {
                proposal_id,
                voter_did,
            } => write!(
                f,
                "duplicate governance vote: proposal={proposal_id}, voter={voter_did}"
            ),
            Self::ProposalClosed {
                proposal_id,
                status,
            } => write!(
                f,
                "proposal is closed for voting: proposal={proposal_id}, status={status:?}"
            ),
            Self::ProposalNotApproved {
                proposal_id,
                status,
            } => write!(
                f,
                "proposal is not approved for execution: proposal={proposal_id}, status={status:?}"
            ),
            Self::AlreadyExecuted(proposal_id) => {
                write!(f, "proposal already executed: {proposal_id}")
            }
        }
    }
}

impl std::error::Error for GovernanceWorkflowError {}

fn require_non_empty(field: &'static str, value: &str) -> Result<(), GovernanceWorkflowError> {
    if value.trim().is_empty() {
        return Err(GovernanceWorkflowError::EmptyField(field));
    }
    Ok(())
}

fn validate_did(value: &str) -> Result<(), GovernanceWorkflowError> {
    AgentDid::parse(value)
        .map_err(|error| GovernanceWorkflowError::InvalidDid(error.to_string()))?;
    Ok(())
}

fn reevaluate_status(record: &mut GovernanceProposalRecord, now_unix: u64) {
    if record.status != GovernanceProposalStatus::Voting {
        return;
    }
    if record.yes_votes >= record.quorum_threshold {
        record.status = GovernanceProposalStatus::Approved;
        return;
    }
    if record.no_votes >= record.quorum_threshold {
        record.status = GovernanceProposalStatus::Rejected;
        return;
    }
    if now_unix > record.voting_deadline_unix {
        record.status = GovernanceProposalStatus::Expired;
    }
}

#[cfg(test)]
mod tests {
    use super::{
        GovernanceProposalDraft, GovernanceProposalStatus, GovernanceVoteChoice,
        GovernanceWorkflow, GovernanceWorkflowError,
    };

    #[test]
    fn submit_rejects_invalid_deadline() {
        let mut workflow = GovernanceWorkflow::new();
        assert_eq!(
            workflow.submit_proposal(GovernanceProposalDraft {
                proposal_id: "gov-deadline".to_owned(),
                title: "Invalid deadline".to_owned(),
                description: "Should fail".to_owned(),
                proposer_did: "kamn:did:agent:validator-1".to_owned(),
                created_at_unix: 100,
                voting_deadline_unix: 99,
                quorum_threshold: 1,
            }),
            Err(GovernanceWorkflowError::InvalidDeadline {
                created_at_unix: 100,
                voting_deadline_unix: 99
            })
        );
    }

    #[test]
    fn no_quorum_transitions_to_rejected() {
        let mut workflow = GovernanceWorkflow::new();
        workflow
            .submit_proposal(GovernanceProposalDraft {
                proposal_id: "gov-reject".to_owned(),
                title: "Reject path".to_owned(),
                description: "No votes should reject".to_owned(),
                proposer_did: "kamn:did:agent:validator-1".to_owned(),
                created_at_unix: 100,
                voting_deadline_unix: 200,
                quorum_threshold: 2,
            })
            .expect("proposal should submit");
        workflow
            .cast_vote(
                "gov-reject",
                "kamn:did:agent:validator-2",
                GovernanceVoteChoice::No,
                110,
            )
            .expect("first no vote should pass");
        workflow
            .cast_vote(
                "gov-reject",
                "kamn:did:agent:validator-3",
                GovernanceVoteChoice::No,
                111,
            )
            .expect("second no vote should pass");

        assert_eq!(
            workflow
                .evaluate("gov-reject", 112)
                .expect("evaluation should succeed"),
            GovernanceProposalStatus::Rejected
        );
    }
}
