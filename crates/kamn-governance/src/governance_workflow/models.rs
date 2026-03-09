/// Draft proposal submitted into the governance workflow.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GovernanceProposalDraft {
    /// Unique proposal identifier.
    pub proposal_id: String,
    /// Human-readable proposal title.
    pub title: String,
    /// Detailed proposal description.
    pub description: String,
    /// DID of the proposer.
    pub proposer_did: String,
    /// Proposal creation timestamp in Unix seconds.
    pub created_at_unix: u64,
    /// Voting deadline timestamp in Unix seconds.
    pub voting_deadline_unix: u64,
    /// Number of matching votes required for terminal approval/rejection.
    pub quorum_threshold: usize,
    /// Optional parameter mutation requested by the proposal.
    pub parameter_change: Option<GovernanceParameterChangeDraft>,
}

/// Draft payload describing a governance-controlled parameter change.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GovernanceParameterChangeDraft {
    /// Catalog key for the parameter being changed.
    pub key: String,
    /// Candidate value to apply if proposal executes.
    pub proposed_value: u64,
    /// Lower bound asserted by proposal.
    pub min_value: u64,
    /// Upper bound asserted by proposal.
    pub max_value: u64,
    /// Target runtime version for compatibility checks.
    pub target_version: String,
}

/// Vote choices available to governance participants.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GovernanceVoteChoice {
    /// Vote in favor of proposal approval.
    Yes,
    /// Vote against proposal approval.
    No,
    /// Abstain while still recording participation.
    Abstain,
}

/// Lifecycle states for governance proposals.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GovernanceProposalStatus {
    /// Proposal is open for voting.
    Voting,
    /// Proposal met approval quorum and is eligible for execution.
    Approved,
    /// Proposal met rejection quorum.
    Rejected,
    /// Proposal has been executed.
    Executed,
    /// Proposal expired without approval/rejection quorum.
    Expired,
}

/// Immutable record of an individual governance vote.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GovernanceVoteRecord {
    /// Proposal identifier this vote belongs to.
    pub proposal_id: String,
    /// DID of the voter.
    pub voter_did: String,
    /// Vote choice cast by the voter.
    pub choice: GovernanceVoteChoice,
    /// Vote timestamp in Unix seconds.
    pub cast_at_unix: u64,
}

/// Immutable record of governance proposal execution.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GovernanceExecutionRecord {
    /// Executed proposal identifier.
    pub proposal_id: String,
    /// DID of executor.
    pub executed_by: String,
    /// Execution timestamp in Unix seconds.
    pub executed_at_unix: u64,
    /// Hash of operation payload executed by governance.
    pub operation_hash: String,
}

/// Canonical proposal state snapshot exposed by query APIs.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GovernanceProposalRecord {
    /// Unique proposal identifier.
    pub proposal_id: String,
    /// Human-readable proposal title.
    pub title: String,
    /// Detailed proposal description.
    pub description: String,
    /// DID of the proposer.
    pub proposer_did: String,
    /// Proposal creation timestamp in Unix seconds.
    pub created_at_unix: u64,
    /// Voting deadline timestamp in Unix seconds.
    pub voting_deadline_unix: u64,
    /// Number of matching votes required for terminal approval/rejection.
    pub quorum_threshold: usize,
    /// Optional parameter mutation requested by proposal.
    pub parameter_change: Option<GovernanceParameterChangeDraft>,
    /// Current proposal lifecycle status.
    pub status: GovernanceProposalStatus,
    /// Count of `Yes` votes.
    pub yes_votes: usize,
    /// Count of `No` votes.
    pub no_votes: usize,
    /// Count of `Abstain` votes.
    pub abstain_votes: usize,
    /// Execution timestamp when status is `Executed`.
    pub executed_at_unix: Option<u64>,
}
