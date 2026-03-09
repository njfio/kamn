use super::error::GovernanceWorkflowError;
use super::models::{GovernanceExecutionRecord, GovernanceProposalRecord, GovernanceVoteRecord};
use super::state::GovernanceWorkflow;

impl GovernanceWorkflow {
    /// Construct an empty governance workflow engine.
    pub fn new() -> Self {
        Self::default()
    }

    /// Return a proposal record snapshot by identifier.
    pub fn proposal(&self, proposal_id: &str) -> Option<GovernanceProposalRecord> {
        self.proposals
            .get(proposal_id)
            .map(|state| state.record.clone())
    }

    /// Return vote history for a proposal in deterministic voter order.
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

    /// Return all execution records emitted by this workflow.
    pub fn execution_history(&self) -> Vec<GovernanceExecutionRecord> {
        self.execution_history.clone()
    }
}
