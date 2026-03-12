use crate::AgentDid;

use super::AgentUpgradeWorkflowError;

pub(crate) const AGENT_UPGRADE_WORKFLOW_INVALID_ALLOWED_PROPOSER_DID_REASON_CODE: &str =
    "agent_upgrade_workflow_invalid_allowed_proposer_did";
pub(crate) const AGENT_UPGRADE_WORKFLOW_INVALID_ALLOWED_VALIDATOR_DID_REASON_CODE: &str =
    "agent_upgrade_workflow_invalid_allowed_validator_did";
pub(crate) const AGENT_UPGRADE_WORKFLOW_INVALID_PROPOSAL_AGENT_DID_REASON_CODE: &str =
    "agent_upgrade_workflow_invalid_proposal_agent_did";
pub(crate) const AGENT_UPGRADE_WORKFLOW_INVALID_REVIEWER_DID_REASON_CODE: &str =
    "agent_upgrade_workflow_invalid_reviewer_did";
pub(crate) const AGENT_UPGRADE_WORKFLOW_INVALID_VALIDATOR_DID_REASON_CODE: &str =
    "agent_upgrade_workflow_invalid_validator_did";
pub(crate) const AGENT_UPGRADE_WORKFLOW_INVALID_EXECUTED_BY_DID_REASON_CODE: &str =
    "agent_upgrade_workflow_invalid_executed_by_did";

pub(crate) fn require_non_empty(
    field: &'static str,
    value: &str,
) -> Result<(), AgentUpgradeWorkflowError> {
    if value.trim().is_empty() {
        return Err(AgentUpgradeWorkflowError::EmptyField(field));
    }
    Ok(())
}

pub(crate) fn validate_timestamp(
    field: &'static str,
    value: u64,
) -> Result<(), AgentUpgradeWorkflowError> {
    if value == 0 {
        return Err(AgentUpgradeWorkflowError::InvalidTimestamp(field));
    }
    Ok(())
}

pub(crate) fn validate_did(
    value: &str,
    field: &'static str,
    reason_code: &'static str,
) -> Result<AgentDid, AgentUpgradeWorkflowError> {
    AgentDid::parse(value).map_err(|error| AgentUpgradeWorkflowError::InvalidDid {
        field,
        reason_code,
        detail: error.to_string(),
    })
}
