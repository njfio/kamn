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
