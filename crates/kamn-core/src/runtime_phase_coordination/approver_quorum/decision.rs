#[derive(Debug, Clone, PartialEq, Eq)]
/// Approver quorum decision.
pub struct ApproverQuorumDecision {
    /// Action id.
    pub action_id: String,
    /// Required approvals.
    pub required_approvals: usize,
    /// Approved by.
    pub approved_by: Vec<String>,
    /// Authorized.
    pub authorized: bool,
}
