mod errors;
mod validation;
mod vote_apply;

pub use errors::AgentUpgradeWorkflowError;
pub(crate) use validation::{
    AGENT_UPGRADE_WORKFLOW_INVALID_ALLOWED_PROPOSER_DID_REASON_CODE,
    AGENT_UPGRADE_WORKFLOW_INVALID_ALLOWED_VALIDATOR_DID_REASON_CODE,
    AGENT_UPGRADE_WORKFLOW_INVALID_EXECUTED_BY_DID_REASON_CODE,
    AGENT_UPGRADE_WORKFLOW_INVALID_PROPOSAL_AGENT_DID_REASON_CODE,
    AGENT_UPGRADE_WORKFLOW_INVALID_REVIEWER_DID_REASON_CODE,
    AGENT_UPGRADE_WORKFLOW_INVALID_VALIDATOR_DID_REASON_CODE, require_non_empty, validate_did,
    validate_timestamp,
};
pub(crate) use vote_apply::apply_yes_votes_as_upgrade_approvals;
