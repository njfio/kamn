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
    pub parameter_change: Option<GovernanceParameterChangeDraft>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GovernanceParameterChangeDraft {
    pub key: String,
    pub proposed_value: u64,
    pub min_value: u64,
    pub max_value: u64,
    pub target_version: String,
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
    pub parameter_change: Option<GovernanceParameterChangeDraft>,
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct SemanticVersion {
    major: u64,
    minor: u64,
    patch: u64,
}

impl SemanticVersion {
    fn parse(value: &str) -> Option<Self> {
        let mut parts = value.split('.');
        let major = parts.next()?.parse().ok()?;
        let minor = parts.next()?.parse().ok()?;
        let patch = parts.next()?.parse().ok()?;
        if parts.next().is_some() {
            return None;
        }
        Some(Self {
            major,
            minor,
            patch,
        })
    }

    fn canonical_string(self) -> String {
        format!("{}.{}.{}", self.major, self.minor, self.patch)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct ParameterPolicySpec {
    key: &'static str,
    min_value: u64,
    max_value: u64,
    min_supported_version: SemanticVersion,
}

const PARAMETER_POLICY_CATALOG: [ParameterPolicySpec; 3] = [
    ParameterPolicySpec {
        key: "listener.quorum",
        min_value: 1,
        max_value: 7,
        min_supported_version: SemanticVersion {
            major: 1,
            minor: 0,
            patch: 0,
        },
    },
    ParameterPolicySpec {
        key: "approver.required_approvals",
        min_value: 1,
        max_value: 7,
        min_supported_version: SemanticVersion {
            major: 1,
            minor: 0,
            patch: 0,
        },
    },
    ParameterPolicySpec {
        key: "watchdog.delivery_ratio_bps",
        min_value: 9000,
        max_value: 9999,
        min_supported_version: SemanticVersion {
            major: 1,
            minor: 1,
            patch: 0,
        },
    },
];

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
                    proposer_did: draft.proposer_did,
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
    InvalidParameterTargetVersion(String),
    InvalidParameterRange {
        key: String,
        min_value: u64,
        max_value: u64,
    },
    UnknownParameterKey(String),
    ParameterRangeOutsidePolicy {
        key: String,
        min_value: u64,
        max_value: u64,
        policy_min_value: u64,
        policy_max_value: u64,
    },
    ParameterUnsupportedForVersion {
        key: String,
        target_version: String,
        min_supported_version: String,
    },
    ParameterOutOfBounds {
        key: String,
        proposed_value: u64,
        min_value: u64,
        max_value: u64,
    },
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
            Self::InvalidParameterTargetVersion(value) => {
                write!(f, "invalid parameter target version: {value}")
            }
            Self::InvalidParameterRange {
                key,
                min_value,
                max_value,
            } => write!(
                f,
                "invalid parameter range: key={key}, min_value={min_value}, max_value={max_value}"
            ),
            Self::UnknownParameterKey(key) => {
                write!(f, "unknown governance parameter key: {key}")
            }
            Self::ParameterRangeOutsidePolicy {
                key,
                min_value,
                max_value,
                policy_min_value,
                policy_max_value,
            } => write!(
                f,
                "parameter range outside policy: key={key}, min_value={min_value}, max_value={max_value}, policy_min_value={policy_min_value}, policy_max_value={policy_max_value}"
            ),
            Self::ParameterUnsupportedForVersion {
                key,
                target_version,
                min_supported_version,
            } => write!(
                f,
                "parameter key unsupported for target version: key={key}, target_version={target_version}, min_supported_version={min_supported_version}"
            ),
            Self::ParameterOutOfBounds {
                key,
                proposed_value,
                min_value,
                max_value,
            } => write!(
                f,
                "parameter value out of bounds: key={key}, proposed_value={proposed_value}, min_value={min_value}, max_value={max_value}"
            ),
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

fn validate_parameter_change(
    parameter_change: &GovernanceParameterChangeDraft,
) -> Result<(), GovernanceWorkflowError> {
    require_non_empty("parameter_change.key", &parameter_change.key)?;
    require_non_empty(
        "parameter_change.target_version",
        &parameter_change.target_version,
    )?;
    let target_version =
        SemanticVersion::parse(&parameter_change.target_version).ok_or_else(|| {
            GovernanceWorkflowError::InvalidParameterTargetVersion(
                parameter_change.target_version.clone(),
            )
        })?;
    if parameter_change.min_value > parameter_change.max_value {
        return Err(GovernanceWorkflowError::InvalidParameterRange {
            key: parameter_change.key.clone(),
            min_value: parameter_change.min_value,
            max_value: parameter_change.max_value,
        });
    }
    let policy = parameter_policy_for_key(&parameter_change.key).ok_or_else(|| {
        GovernanceWorkflowError::UnknownParameterKey(parameter_change.key.clone())
    })?;
    if target_version < policy.min_supported_version {
        return Err(GovernanceWorkflowError::ParameterUnsupportedForVersion {
            key: parameter_change.key.clone(),
            target_version: parameter_change.target_version.clone(),
            min_supported_version: policy.min_supported_version.canonical_string(),
        });
    }
    if parameter_change.min_value < policy.min_value
        || parameter_change.max_value > policy.max_value
    {
        return Err(GovernanceWorkflowError::ParameterRangeOutsidePolicy {
            key: parameter_change.key.clone(),
            min_value: parameter_change.min_value,
            max_value: parameter_change.max_value,
            policy_min_value: policy.min_value,
            policy_max_value: policy.max_value,
        });
    }
    if parameter_change.proposed_value < parameter_change.min_value
        || parameter_change.proposed_value > parameter_change.max_value
    {
        return Err(GovernanceWorkflowError::ParameterOutOfBounds {
            key: parameter_change.key.clone(),
            proposed_value: parameter_change.proposed_value,
            min_value: parameter_change.min_value,
            max_value: parameter_change.max_value,
        });
    }
    Ok(())
}

fn parameter_policy_for_key(key: &str) -> Option<&'static ParameterPolicySpec> {
    PARAMETER_POLICY_CATALOG
        .iter()
        .find(|policy| policy.key == key)
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
                parameter_change: None,
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
                parameter_change: None,
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
