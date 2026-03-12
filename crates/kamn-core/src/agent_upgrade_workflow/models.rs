use super::audit::AgentUpgradeAuditEvent;
use crate::{GovernanceProposalStatus, GovernanceWorkflow, VersionUpgradeOrchestrator};
use std::collections::{BTreeMap, BTreeSet};

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

/// In-memory workflow coordinating proposal review, governance, and activation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AgentDrivenUpgradeWorkflow {
    pub(super) allowed_agent_proposers: BTreeSet<String>,
    pub(super) allowed_validator_voters: BTreeSet<String>,
    pub(super) required_human_reviews: usize,
    pub(super) required_validator_quorum: usize,
    pub(super) min_activation_delay_secs: u64,
    pub(super) governance: GovernanceWorkflow,
    pub(super) orchestrator: VersionUpgradeOrchestrator,
    pub(super) proposals: BTreeMap<String, AgentUpgradeProposalRecord>,
    pub(super) events: Vec<AgentUpgradeAuditEvent>,
}
