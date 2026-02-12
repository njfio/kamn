//! Agent-driven runtime upgrade proposal, review, governance, and activation workflow contracts.

use crate::{
    AgentDid, GovernanceProposalDraft, GovernanceProposalRecord, GovernanceProposalStatus,
    GovernanceVoteChoice, GovernanceVoteRecord, GovernanceWorkflow, GovernanceWorkflowError,
    UpgradeOrchestrationError, VersionUpgradeAuditView, VersionUpgradeOrchestrator,
};
use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

/// Configuration parameters used to initialize an agent-driven upgrade workflow.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AgentUpgradeWorkflowConfig {
    /// Runtime version currently active at workflow initialization.
    pub current_version: String,
    /// DID allowlist for agents permitted to submit upgrade proposals.
    pub allowed_agent_proposers: Vec<String>,
    /// DID allowlist for validators permitted to review and vote.
    pub allowed_validator_voters: Vec<String>,
    /// Minimum number of distinct human reviews required before governance submission.
    pub required_human_reviews: usize,
    /// Governance quorum required to approve the proposal.
    pub required_validator_quorum: usize,
    /// Minimum delay, in seconds, between governance approval and activation.
    pub min_activation_delay_secs: u64,
}

/// Draft payload submitted by an authorized agent proposer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AgentUpgradeProposalDraft {
    /// Unique upgrade proposal identifier.
    pub proposal_id: String,
    /// Candidate runtime version to activate if the workflow succeeds.
    pub target_version: String,
    /// DID of the proposing agent.
    pub agent_did: String,
    /// Human-readable rationale for the upgrade.
    pub rationale: String,
    /// Proposal creation timestamp in Unix seconds.
    pub created_at_unix: u64,
    /// Governance voting deadline timestamp in Unix seconds.
    pub voting_deadline_unix: u64,
}

/// Lifecycle state machine for agent-driven upgrade proposals.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AgentUpgradeProposalState {
    /// Proposal exists and is collecting required human reviews.
    PendingHumanReview,
    /// Proposal has been promoted and is currently in governance voting.
    GovernanceVoting,
    /// Governance has approved the proposal and activation delay may apply.
    GovernanceApproved,
    /// Upgrade has been executed and marked active.
    Activated,
}

/// Canonical state snapshot for a proposal tracked by this workflow.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AgentUpgradeProposalRecord {
    /// Unique proposal identifier.
    pub proposal_id: String,
    /// Candidate runtime version requested by the proposal.
    pub target_version: String,
    /// DID of the original proposing agent.
    pub agent_did: String,
    /// Human-readable upgrade rationale.
    pub rationale: String,
    /// Proposal creation timestamp in Unix seconds.
    pub created_at_unix: u64,
    /// Governance voting deadline timestamp in Unix seconds.
    pub voting_deadline_unix: u64,
    /// Set of reviewers that approved this proposal for governance submission.
    pub human_reviewers: BTreeSet<String>,
    /// Current proposal lifecycle state.
    pub state: AgentUpgradeProposalState,
    /// Current governance status mirrored from the governance workflow.
    pub governance_status: GovernanceProposalStatus,
    /// Timestamp when governance first reached approved status.
    pub governance_approved_at_unix: Option<u64>,
    /// Timestamp when upgrade activation completed.
    pub activated_at_unix: Option<u64>,
    /// Governance operation hash used during final execution.
    pub operation_hash: Option<String>,
}

/// Audit event kinds emitted by the agent-driven upgrade workflow.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AgentUpgradeAuditEventKind {
    /// An agent submitted a proposal draft.
    AgentProposed,
    /// A human reviewer approved the proposal for governance promotion.
    HumanReviewApproved,
    /// The proposal was submitted into governance voting.
    GovernanceSubmitted,
    /// Governance reached quorum approval for the proposal.
    GovernanceApproved,
    /// Governance execution completed for the proposal.
    GovernanceExecuted,
    /// The runtime upgrade was activated.
    UpgradeActivated,
}

/// Immutable audit event record emitted by the workflow.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AgentUpgradeAuditEvent {
    /// Proposal identifier associated with this event.
    pub proposal_id: String,
    /// DID or system actor responsible for the event.
    pub actor_did: String,
    /// Event timestamp in Unix seconds.
    pub event_at_unix: u64,
    /// Event classification.
    pub kind: AgentUpgradeAuditEventKind,
    /// Optional operator-facing annotation for this event.
    pub note: Option<String>,
}

/// In-memory workflow coordinating proposal review, governance, and activation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AgentDrivenUpgradeWorkflow {
    allowed_agent_proposers: BTreeSet<String>,
    allowed_validator_voters: BTreeSet<String>,
    required_human_reviews: usize,
    required_validator_quorum: usize,
    min_activation_delay_secs: u64,
    governance: GovernanceWorkflow,
    orchestrator: VersionUpgradeOrchestrator,
    proposals: BTreeMap<String, AgentUpgradeProposalRecord>,
    events: Vec<AgentUpgradeAuditEvent>,
}

impl AgentDrivenUpgradeWorkflow {
    /// Construct a workflow instance after validating config invariants and DID allowlists.
    pub fn new(config: AgentUpgradeWorkflowConfig) -> Result<Self, AgentUpgradeWorkflowError> {
        if config.required_human_reviews == 0 {
            return Err(AgentUpgradeWorkflowError::InvalidRequiredHumanReviews(0));
        }
        if config.required_validator_quorum == 0 {
            return Err(AgentUpgradeWorkflowError::InvalidRequiredValidatorQuorum(0));
        }
        if config.min_activation_delay_secs == 0 {
            return Err(AgentUpgradeWorkflowError::InvalidMinActivationDelaySecs(0));
        }
        if config.allowed_agent_proposers.is_empty() {
            return Err(AgentUpgradeWorkflowError::MissingAllowedAgentProposers);
        }
        if config.allowed_validator_voters.is_empty() {
            return Err(AgentUpgradeWorkflowError::MissingAllowedValidatorVoters);
        }

        let mut allowed_agent_proposers = BTreeSet::new();
        for proposer in config.allowed_agent_proposers {
            validate_did(&proposer)?;
            allowed_agent_proposers.insert(proposer);
        }
        let mut allowed_validator_voters = BTreeSet::new();
        for validator in config.allowed_validator_voters {
            validate_did(&validator)?;
            allowed_validator_voters.insert(validator);
        }

        let orchestrator =
            VersionUpgradeOrchestrator::new(&config.current_version).map_err(Self::map_upgrade)?;

        Ok(Self {
            allowed_agent_proposers,
            allowed_validator_voters,
            required_human_reviews: config.required_human_reviews,
            required_validator_quorum: config.required_validator_quorum,
            min_activation_delay_secs: config.min_activation_delay_secs,
            governance: GovernanceWorkflow::new(),
            orchestrator,
            proposals: BTreeMap::new(),
            events: Vec::new(),
        })
    }

    /// Register a new proposal from an authorized agent and seed upgrade orchestration state.
    pub fn submit_agent_proposal(
        &mut self,
        draft: AgentUpgradeProposalDraft,
    ) -> Result<(), AgentUpgradeWorkflowError> {
        require_non_empty("proposal_id", &draft.proposal_id)?;
        require_non_empty("rationale", &draft.rationale)?;
        validate_did(&draft.agent_did)?;
        validate_timestamp("created_at_unix", draft.created_at_unix)?;
        if draft.voting_deadline_unix <= draft.created_at_unix {
            return Err(AgentUpgradeWorkflowError::InvalidDeadline {
                created_at_unix: draft.created_at_unix,
                voting_deadline_unix: draft.voting_deadline_unix,
            });
        }
        if !self.allowed_agent_proposers.contains(&draft.agent_did) {
            return Err(AgentUpgradeWorkflowError::UnauthorizedAgentProposer(
                draft.agent_did,
            ));
        }
        if self.proposals.contains_key(&draft.proposal_id) {
            return Err(AgentUpgradeWorkflowError::ProposalAlreadyExists(
                draft.proposal_id,
            ));
        }

        self.orchestrator
            .propose_upgrade(
                &draft.proposal_id,
                &draft.target_version,
                &draft.agent_did,
                self.required_validator_quorum,
                draft.created_at_unix,
            )
            .map_err(Self::map_upgrade)?;

        self.proposals.insert(
            draft.proposal_id.clone(),
            AgentUpgradeProposalRecord {
                proposal_id: draft.proposal_id.clone(),
                target_version: draft.target_version,
                agent_did: draft.agent_did.clone(),
                rationale: draft.rationale,
                created_at_unix: draft.created_at_unix,
                voting_deadline_unix: draft.voting_deadline_unix,
                human_reviewers: BTreeSet::new(),
                state: AgentUpgradeProposalState::PendingHumanReview,
                governance_status: GovernanceProposalStatus::Voting,
                governance_approved_at_unix: None,
                activated_at_unix: None,
                operation_hash: None,
            },
        );
        self.events.push(AgentUpgradeAuditEvent {
            proposal_id: draft.proposal_id,
            actor_did: draft.agent_did,
            event_at_unix: draft.created_at_unix,
            kind: AgentUpgradeAuditEventKind::AgentProposed,
            note: Some("agent proposal submitted".to_owned()),
        });
        Ok(())
    }

    /// Record a distinct human-review approval for a pending proposal.
    pub fn approve_human_review(
        &mut self,
        proposal_id: &str,
        reviewer_did: &str,
        reviewed_at_unix: u64,
    ) -> Result<(), AgentUpgradeWorkflowError> {
        validate_did(reviewer_did)?;
        validate_timestamp("reviewed_at_unix", reviewed_at_unix)?;
        if !self.allowed_validator_voters.contains(reviewer_did) {
            return Err(AgentUpgradeWorkflowError::UnauthorizedHumanReviewer(
                reviewer_did.to_owned(),
            ));
        }
        let proposal = self
            .proposals
            .get_mut(proposal_id)
            .ok_or_else(|| AgentUpgradeWorkflowError::ProposalNotFound(proposal_id.to_owned()))?;

        if !proposal.human_reviewers.insert(reviewer_did.to_owned()) {
            return Err(AgentUpgradeWorkflowError::DuplicateHumanReview {
                proposal_id: proposal_id.to_owned(),
                reviewer_did: reviewer_did.to_owned(),
            });
        }
        self.events.push(AgentUpgradeAuditEvent {
            proposal_id: proposal_id.to_owned(),
            actor_did: reviewer_did.to_owned(),
            event_at_unix: reviewed_at_unix,
            kind: AgentUpgradeAuditEventKind::HumanReviewApproved,
            note: Some("human review approval registered".to_owned()),
        });
        Ok(())
    }

    /// Promote a reviewed proposal into governance voting.
    pub fn submit_to_governance(
        &mut self,
        proposal_id: &str,
        submitted_at_unix: u64,
    ) -> Result<(), AgentUpgradeWorkflowError> {
        validate_timestamp("submitted_at_unix", submitted_at_unix)?;
        let mut proposal =
            self.proposals.get(proposal_id).cloned().ok_or_else(|| {
                AgentUpgradeWorkflowError::ProposalNotFound(proposal_id.to_owned())
            })?;

        if proposal.state != AgentUpgradeProposalState::PendingHumanReview {
            return Err(AgentUpgradeWorkflowError::GovernanceSubmissionNotAllowed {
                proposal_id: proposal_id.to_owned(),
                state: proposal.state,
            });
        }
        let provided_reviews = proposal.human_reviewers.len();
        if provided_reviews < self.required_human_reviews {
            return Err(AgentUpgradeWorkflowError::InsufficientHumanReviews {
                required: self.required_human_reviews,
                provided: provided_reviews,
            });
        }
        if proposal.voting_deadline_unix <= submitted_at_unix {
            return Err(AgentUpgradeWorkflowError::InvalidDeadline {
                created_at_unix: submitted_at_unix,
                voting_deadline_unix: proposal.voting_deadline_unix,
            });
        }

        self.governance
            .submit_proposal(GovernanceProposalDraft {
                proposal_id: proposal.proposal_id.clone(),
                title: format!("Agent-driven upgrade to {}", proposal.target_version),
                description: proposal.rationale.clone(),
                proposer_did: proposal.agent_did.clone(),
                created_at_unix: submitted_at_unix,
                voting_deadline_unix: proposal.voting_deadline_unix,
                quorum_threshold: self.required_validator_quorum,
                parameter_change: None,
            })
            .map_err(Self::map_governance)?;

        proposal.state = AgentUpgradeProposalState::GovernanceVoting;
        proposal.governance_status = GovernanceProposalStatus::Voting;
        proposal.governance_approved_at_unix = None;
        self.proposals.insert(proposal_id.to_owned(), proposal);
        self.events.push(AgentUpgradeAuditEvent {
            proposal_id: proposal_id.to_owned(),
            actor_did: "workflow".to_owned(),
            event_at_unix: submitted_at_unix,
            kind: AgentUpgradeAuditEventKind::GovernanceSubmitted,
            note: Some("proposal promoted to governance workflow".to_owned()),
        });
        Ok(())
    }

    /// Cast a governance vote from an allowlisted validator and update mirrored proposal state.
    pub fn cast_validator_vote(
        &mut self,
        proposal_id: &str,
        validator_did: &str,
        choice: GovernanceVoteChoice,
        cast_at_unix: u64,
    ) -> Result<(), AgentUpgradeWorkflowError> {
        validate_did(validator_did)?;
        if !self.allowed_validator_voters.contains(validator_did) {
            return Err(AgentUpgradeWorkflowError::UnauthorizedValidatorVoter(
                validator_did.to_owned(),
            ));
        }
        self.governance
            .cast_vote(proposal_id, validator_did, choice, cast_at_unix)
            .map_err(Self::map_governance)?;
        let status = self
            .governance
            .evaluate(proposal_id, cast_at_unix)
            .map_err(Self::map_governance)?;
        if let Some(record) = self.proposals.get_mut(proposal_id) {
            record.governance_status = status;
            if status == GovernanceProposalStatus::Approved {
                record.state = AgentUpgradeProposalState::GovernanceApproved;
                if record.governance_approved_at_unix.is_none() {
                    record.governance_approved_at_unix = Some(cast_at_unix);
                }
                self.events.push(AgentUpgradeAuditEvent {
                    proposal_id: proposal_id.to_owned(),
                    actor_did: validator_did.to_owned(),
                    event_at_unix: cast_at_unix,
                    kind: AgentUpgradeAuditEventKind::GovernanceApproved,
                    note: Some("governance quorum reached".to_owned()),
                });
            }
        }
        Ok(())
    }

    /// Execute governance-approved proposal and activate the runtime upgrade once delay passes.
    pub fn finalize_upgrade(
        &mut self,
        proposal_id: &str,
        executed_by: &str,
        executed_at_unix: u64,
        operation_hash: &str,
    ) -> Result<(), AgentUpgradeWorkflowError> {
        validate_did(executed_by)?;
        validate_timestamp("executed_at_unix", executed_at_unix)?;
        require_non_empty("operation_hash", operation_hash)?;

        let mut proposal =
            self.proposals.get(proposal_id).cloned().ok_or_else(|| {
                AgentUpgradeWorkflowError::ProposalNotFound(proposal_id.to_owned())
            })?;
        let governance_status = self
            .governance
            .evaluate(proposal_id, executed_at_unix)
            .map_err(Self::map_governance)?;
        if governance_status != GovernanceProposalStatus::Approved {
            return Err(AgentUpgradeWorkflowError::GovernanceStatusNotApproved {
                proposal_id: proposal_id.to_owned(),
                status: governance_status,
            });
        }
        let governance_approved_at_unix =
            proposal.governance_approved_at_unix.ok_or_else(|| {
                AgentUpgradeWorkflowError::MissingGovernanceApprovalTimestamp(
                    proposal_id.to_owned(),
                )
            })?;
        let earliest_activation_unix =
            governance_approved_at_unix.saturating_add(self.min_activation_delay_secs);
        if executed_at_unix < earliest_activation_unix {
            return Err(AgentUpgradeWorkflowError::ActivationDelayNotElapsed {
                proposal_id: proposal_id.to_owned(),
                earliest_activation_unix,
                attempted_activation_unix: executed_at_unix,
            });
        }

        self.governance
            .execute(proposal_id, executed_by, executed_at_unix, operation_hash)
            .map_err(Self::map_governance)?;
        let votes = self
            .governance
            .vote_history(proposal_id)
            .map_err(Self::map_governance)?;
        apply_yes_votes_as_upgrade_approvals(&mut self.orchestrator, proposal_id, votes)
            .map_err(Self::map_upgrade)?;
        self.orchestrator
            .mark_governance_status(
                proposal_id,
                GovernanceProposalStatus::Approved,
                executed_at_unix,
            )
            .map_err(Self::map_upgrade)?;
        self.orchestrator
            .activate_upgrade(proposal_id, executed_by, executed_at_unix)
            .map_err(Self::map_upgrade)?;

        proposal.governance_status = GovernanceProposalStatus::Executed;
        proposal.state = AgentUpgradeProposalState::Activated;
        proposal.activated_at_unix = Some(executed_at_unix);
        proposal.operation_hash = Some(operation_hash.to_owned());
        self.proposals.insert(proposal_id.to_owned(), proposal);

        self.events.push(AgentUpgradeAuditEvent {
            proposal_id: proposal_id.to_owned(),
            actor_did: executed_by.to_owned(),
            event_at_unix: executed_at_unix,
            kind: AgentUpgradeAuditEventKind::GovernanceExecuted,
            note: Some("governance execution completed".to_owned()),
        });
        self.events.push(AgentUpgradeAuditEvent {
            proposal_id: proposal_id.to_owned(),
            actor_did: executed_by.to_owned(),
            event_at_unix: executed_at_unix,
            kind: AgentUpgradeAuditEventKind::UpgradeActivated,
            note: Some("version upgrade activated".to_owned()),
        });
        Ok(())
    }

    /// Return proposal snapshot by identifier if present.
    pub fn proposal(&self, proposal_id: &str) -> Option<AgentUpgradeProposalRecord> {
        self.proposals.get(proposal_id).cloned()
    }

    /// Return mirrored governance proposal record by identifier if present.
    pub fn governance_record(&self, proposal_id: &str) -> Option<GovernanceProposalRecord> {
        self.governance.proposal(proposal_id)
    }

    /// Return upgrade-orchestrator audit view for all tracked operations.
    pub fn upgrade_audit_view(&self) -> VersionUpgradeAuditView {
        self.orchestrator.audit_view()
    }

    /// Return emitted agent workflow audit events in insertion order.
    pub fn agent_audit_log(&self) -> Vec<AgentUpgradeAuditEvent> {
        self.events.clone()
    }

    fn map_governance(error: GovernanceWorkflowError) -> AgentUpgradeWorkflowError {
        AgentUpgradeWorkflowError::GovernanceWorkflow(error)
    }

    fn map_upgrade(error: UpgradeOrchestrationError) -> AgentUpgradeWorkflowError {
        AgentUpgradeWorkflowError::UpgradeOrchestration(error)
    }
}

fn apply_yes_votes_as_upgrade_approvals(
    orchestrator: &mut VersionUpgradeOrchestrator,
    proposal_id: &str,
    votes: Vec<GovernanceVoteRecord>,
) -> Result<(), UpgradeOrchestrationError> {
    for vote in votes {
        if vote.choice == GovernanceVoteChoice::Yes {
            orchestrator.approve_upgrade(proposal_id, &vote.voter_did, vote.cast_at_unix)?;
        }
    }
    Ok(())
}

/// Errors emitted by the agent-driven upgrade workflow lifecycle.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AgentUpgradeWorkflowError {
    /// Required string field was empty or whitespace.
    EmptyField(&'static str),
    /// DID failed canonical parse/validation.
    InvalidDid(String),
    /// Timestamp field was zero or otherwise invalid.
    InvalidTimestamp(&'static str),
    /// Voting deadline does not occur after the reference creation/submission timestamp.
    InvalidDeadline {
        /// Reference creation/submission timestamp in Unix seconds.
        created_at_unix: u64,
        /// Voting deadline timestamp in Unix seconds.
        voting_deadline_unix: u64,
    },
    /// Required human-review threshold must be positive.
    InvalidRequiredHumanReviews(usize),
    /// Required validator quorum must be positive.
    InvalidRequiredValidatorQuorum(usize),
    /// Activation delay must be positive.
    InvalidMinActivationDelaySecs(u64),
    /// Agent proposer allowlist is empty.
    MissingAllowedAgentProposers,
    /// Validator voter allowlist is empty.
    MissingAllowedValidatorVoters,
    /// Agent DID is not authorized to submit proposals.
    UnauthorizedAgentProposer(String),
    /// Reviewer DID is not authorized for human review approval.
    UnauthorizedHumanReviewer(String),
    /// Validator DID is not authorized to vote.
    UnauthorizedValidatorVoter(String),
    /// Proposal identifier already exists.
    ProposalAlreadyExists(String),
    /// Proposal identifier does not exist.
    ProposalNotFound(String),
    /// Reviewer submitted a duplicate human approval for the same proposal.
    DuplicateHumanReview {
        /// Proposal identifier.
        proposal_id: String,
        /// Reviewer DID that attempted duplicate approval.
        reviewer_did: String,
    },
    /// Proposal does not yet meet required human-review threshold.
    InsufficientHumanReviews {
        /// Required number of distinct human reviews.
        required: usize,
        /// Number of reviews currently recorded.
        provided: usize,
    },
    /// Proposal state does not permit submission into governance.
    GovernanceSubmissionNotAllowed {
        /// Proposal identifier.
        proposal_id: String,
        /// Current proposal state that blocks transition.
        state: AgentUpgradeProposalState,
    },
    /// Governance status is not approved at execution time.
    GovernanceStatusNotApproved {
        /// Proposal identifier.
        proposal_id: String,
        /// Governance status observed when execution was attempted.
        status: GovernanceProposalStatus,
    },
    /// Governance approval timestamp was never recorded for the proposal.
    MissingGovernanceApprovalTimestamp(String),
    /// Activation attempted before minimum post-approval delay elapsed.
    ActivationDelayNotElapsed {
        /// Proposal identifier.
        proposal_id: String,
        /// Earliest valid activation timestamp in Unix seconds.
        earliest_activation_unix: u64,
        /// Attempted activation timestamp in Unix seconds.
        attempted_activation_unix: u64,
    },
    /// Wrapped error propagated from governance workflow operations.
    GovernanceWorkflow(GovernanceWorkflowError),
    /// Wrapped error propagated from upgrade orchestrator operations.
    UpgradeOrchestration(UpgradeOrchestrationError),
}

impl fmt::Display for AgentUpgradeWorkflowError {
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
            Self::InvalidRequiredHumanReviews(value) => {
                write!(f, "invalid required human reviews: {value}")
            }
            Self::InvalidRequiredValidatorQuorum(value) => {
                write!(f, "invalid required validator quorum: {value}")
            }
            Self::InvalidMinActivationDelaySecs(value) => {
                write!(f, "invalid minimum activation delay seconds: {value}")
            }
            Self::MissingAllowedAgentProposers => {
                write!(f, "allowed agent proposer set must not be empty")
            }
            Self::MissingAllowedValidatorVoters => {
                write!(f, "allowed validator voter set must not be empty")
            }
            Self::UnauthorizedAgentProposer(agent_did) => {
                write!(f, "unauthorized agent proposer: {agent_did}")
            }
            Self::UnauthorizedHumanReviewer(reviewer_did) => {
                write!(f, "unauthorized human reviewer: {reviewer_did}")
            }
            Self::UnauthorizedValidatorVoter(validator_did) => {
                write!(f, "unauthorized validator voter: {validator_did}")
            }
            Self::ProposalAlreadyExists(proposal_id) => {
                write!(f, "proposal already exists: {proposal_id}")
            }
            Self::ProposalNotFound(proposal_id) => write!(f, "proposal not found: {proposal_id}"),
            Self::DuplicateHumanReview {
                proposal_id,
                reviewer_did,
            } => write!(
                f,
                "duplicate human review: proposal={proposal_id}, reviewer={reviewer_did}"
            ),
            Self::InsufficientHumanReviews { required, provided } => write!(
                f,
                "insufficient human reviews: required {required}, provided {provided}"
            ),
            Self::GovernanceSubmissionNotAllowed { proposal_id, state } => write!(
                f,
                "governance submission not allowed: proposal={proposal_id}, state={state:?}"
            ),
            Self::GovernanceStatusNotApproved {
                proposal_id,
                status,
            } => write!(
                f,
                "governance status is not approved: proposal={proposal_id}, status={status:?}"
            ),
            Self::MissingGovernanceApprovalTimestamp(proposal_id) => write!(
                f,
                "governance approval timestamp is missing for proposal: {proposal_id}"
            ),
            Self::ActivationDelayNotElapsed {
                proposal_id,
                earliest_activation_unix,
                attempted_activation_unix,
            } => write!(
                f,
                "activation delay not elapsed: proposal={proposal_id}, earliest_activation_unix={earliest_activation_unix}, attempted_activation_unix={attempted_activation_unix}"
            ),
            Self::GovernanceWorkflow(error) => write!(f, "{error}"),
            Self::UpgradeOrchestration(error) => write!(f, "{error}"),
        }
    }
}

impl std::error::Error for AgentUpgradeWorkflowError {}

fn require_non_empty(field: &'static str, value: &str) -> Result<(), AgentUpgradeWorkflowError> {
    if value.trim().is_empty() {
        return Err(AgentUpgradeWorkflowError::EmptyField(field));
    }
    Ok(())
}

fn validate_timestamp(field: &'static str, value: u64) -> Result<(), AgentUpgradeWorkflowError> {
    if value == 0 {
        return Err(AgentUpgradeWorkflowError::InvalidTimestamp(field));
    }
    Ok(())
}

fn validate_did(value: &str) -> Result<(), AgentUpgradeWorkflowError> {
    AgentDid::parse(value)
        .map_err(|error| AgentUpgradeWorkflowError::InvalidDid(error.to_string()))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{
        AgentDrivenUpgradeWorkflow, AgentUpgradeProposalDraft, AgentUpgradeProposalState,
        AgentUpgradeWorkflowConfig, AgentUpgradeWorkflowError,
    };
    use crate::GovernanceVoteChoice;

    #[test]
    fn constructor_requires_allowlisted_agent_set() {
        assert_eq!(
            AgentDrivenUpgradeWorkflow::new(AgentUpgradeWorkflowConfig {
                current_version: "v0.1.0".to_owned(),
                allowed_agent_proposers: Vec::new(),
                allowed_validator_voters: vec!["kamn:did:agent:validator-1".to_owned()],
                required_human_reviews: 1,
                required_validator_quorum: 2,
                min_activation_delay_secs: 60,
            }),
            Err(AgentUpgradeWorkflowError::MissingAllowedAgentProposers)
        );
    }

    #[test]
    fn constructor_requires_allowlisted_validator_set() {
        assert_eq!(
            AgentDrivenUpgradeWorkflow::new(AgentUpgradeWorkflowConfig {
                current_version: "v0.1.0".to_owned(),
                allowed_agent_proposers: vec!["kamn:did:agent:upgrade-bot".to_owned()],
                allowed_validator_voters: Vec::new(),
                required_human_reviews: 1,
                required_validator_quorum: 2,
                min_activation_delay_secs: 60,
            }),
            Err(AgentUpgradeWorkflowError::MissingAllowedValidatorVoters)
        );
    }

    #[test]
    fn constructor_rejects_zero_activation_delay() {
        assert_eq!(
            AgentDrivenUpgradeWorkflow::new(AgentUpgradeWorkflowConfig {
                current_version: "v0.1.0".to_owned(),
                allowed_agent_proposers: vec!["kamn:did:agent:upgrade-bot".to_owned()],
                allowed_validator_voters: vec!["kamn:did:agent:validator-1".to_owned()],
                required_human_reviews: 1,
                required_validator_quorum: 2,
                min_activation_delay_secs: 0,
            }),
            Err(AgentUpgradeWorkflowError::InvalidMinActivationDelaySecs(0))
        );
    }

    #[test]
    fn duplicate_human_review_is_rejected() {
        let mut workflow = AgentDrivenUpgradeWorkflow::new(AgentUpgradeWorkflowConfig {
            current_version: "v0.1.0".to_owned(),
            allowed_agent_proposers: vec!["kamn:did:agent:upgrade-bot".to_owned()],
            allowed_validator_voters: vec!["kamn:did:agent:validator-1".to_owned()],
            required_human_reviews: 1,
            required_validator_quorum: 2,
            min_activation_delay_secs: 60,
        })
        .expect("workflow should initialize");
        workflow
            .submit_agent_proposal(AgentUpgradeProposalDraft {
                proposal_id: "agent-upgrade-a".to_owned(),
                target_version: "v0.2.0".to_owned(),
                agent_did: "kamn:did:agent:upgrade-bot".to_owned(),
                rationale: "initial proposal".to_owned(),
                created_at_unix: 100,
                voting_deadline_unix: 200,
            })
            .expect("proposal should register");

        workflow
            .approve_human_review("agent-upgrade-a", "kamn:did:agent:validator-1", 110)
            .expect("first review should pass");
        assert_eq!(
            workflow.approve_human_review("agent-upgrade-a", "kamn:did:agent:validator-1", 111),
            Err(AgentUpgradeWorkflowError::DuplicateHumanReview {
                proposal_id: "agent-upgrade-a".to_owned(),
                reviewer_did: "kamn:did:agent:validator-1".to_owned(),
            })
        );

        let record = workflow
            .proposal("agent-upgrade-a")
            .expect("proposal should exist");
        assert_eq!(record.state, AgentUpgradeProposalState::PendingHumanReview);
    }

    #[test]
    fn cast_validator_vote_rejects_non_allowlisted_validator() {
        let mut workflow = AgentDrivenUpgradeWorkflow::new(AgentUpgradeWorkflowConfig {
            current_version: "v0.1.0".to_owned(),
            allowed_agent_proposers: vec!["kamn:did:agent:upgrade-bot".to_owned()],
            allowed_validator_voters: vec!["kamn:did:agent:validator-1".to_owned()],
            required_human_reviews: 1,
            required_validator_quorum: 1,
            min_activation_delay_secs: 60,
        })
        .expect("workflow should initialize");
        workflow
            .submit_agent_proposal(AgentUpgradeProposalDraft {
                proposal_id: "agent-upgrade-b".to_owned(),
                target_version: "v0.2.0".to_owned(),
                agent_did: "kamn:did:agent:upgrade-bot".to_owned(),
                rationale: "validator-vote-allowlist".to_owned(),
                created_at_unix: 100,
                voting_deadline_unix: 200,
            })
            .expect("proposal should register");
        workflow
            .approve_human_review("agent-upgrade-b", "kamn:did:agent:validator-1", 110)
            .expect("review should pass");
        workflow
            .submit_to_governance("agent-upgrade-b", 120)
            .expect("governance submission should pass");

        assert_eq!(
            workflow.cast_validator_vote(
                "agent-upgrade-b",
                "kamn:did:agent:validator-rogue",
                GovernanceVoteChoice::Yes,
                130,
            ),
            Err(AgentUpgradeWorkflowError::UnauthorizedValidatorVoter(
                "kamn:did:agent:validator-rogue".to_owned()
            ))
        );
    }

    #[test]
    fn approve_human_review_rejects_non_allowlisted_reviewer() {
        let mut workflow = AgentDrivenUpgradeWorkflow::new(AgentUpgradeWorkflowConfig {
            current_version: "v0.1.0".to_owned(),
            allowed_agent_proposers: vec!["kamn:did:agent:upgrade-bot".to_owned()],
            allowed_validator_voters: vec!["kamn:did:agent:validator-1".to_owned()],
            required_human_reviews: 1,
            required_validator_quorum: 1,
            min_activation_delay_secs: 60,
        })
        .expect("workflow should initialize");
        workflow
            .submit_agent_proposal(AgentUpgradeProposalDraft {
                proposal_id: "agent-upgrade-c".to_owned(),
                target_version: "v0.2.0".to_owned(),
                agent_did: "kamn:did:agent:upgrade-bot".to_owned(),
                rationale: "reviewer-allowlist".to_owned(),
                created_at_unix: 100,
                voting_deadline_unix: 200,
            })
            .expect("proposal should register");

        assert_eq!(
            workflow
                .approve_human_review("agent-upgrade-c", "kamn:did:agent:validator-rogue", 110,),
            Err(AgentUpgradeWorkflowError::UnauthorizedHumanReviewer(
                "kamn:did:agent:validator-rogue".to_owned()
            ))
        );
    }
}
