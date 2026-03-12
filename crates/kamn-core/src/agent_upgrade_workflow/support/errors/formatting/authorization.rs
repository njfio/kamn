use std::fmt;

use crate::agent_upgrade_workflow::support::errors::AgentUpgradeWorkflowError;

pub(super) fn format_authorization_error(
    error: &AgentUpgradeWorkflowError,
    f: &mut fmt::Formatter<'_>,
) -> Option<fmt::Result> {
    match error {
        AgentUpgradeWorkflowError::UnauthorizedAgentProposer(agent_did) => {
            Some(write!(f, "unauthorized agent proposer: {agent_did}"))
        }
        AgentUpgradeWorkflowError::UnauthorizedHumanReviewer(reviewer_did) => {
            Some(write!(f, "unauthorized human reviewer: {reviewer_did}"))
        }
        AgentUpgradeWorkflowError::UnauthorizedValidatorVoter(validator_did) => {
            Some(write!(f, "unauthorized validator voter: {validator_did}"))
        }
        AgentUpgradeWorkflowError::ProposalAlreadyExists(proposal_id) => {
            Some(write!(f, "proposal already exists: {proposal_id}"))
        }
        AgentUpgradeWorkflowError::ProposalNotFound(proposal_id) => {
            Some(write!(f, "proposal not found: {proposal_id}"))
        }
        _ => None,
    }
}
