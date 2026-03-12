mod formatting;

use crate::{GovernanceProposalStatus, GovernanceWorkflowError, UpgradeOrchestrationError};
use std::fmt;

use crate::agent_upgrade_workflow::AgentUpgradeProposalState;

/// Errors emitted by the agent-driven upgrade workflow lifecycle.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AgentUpgradeWorkflowError {
    /// Required string field was empty or whitespace.
    EmptyField(&'static str),
    /// DID failed canonical parse/validation.
    InvalidDid {
        /// Input field carrying invalid DID.
        field: &'static str,
        /// Stable deterministic reason marker.
        reason_code: &'static str,
        /// Canonical parser detail.
        detail: String,
    },
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
        formatting::format_error(self, f)
    }
}

impl std::error::Error for AgentUpgradeWorkflowError {}
