use crate::GovernanceProposalStatus;

use crate::agent_upgrade_workflow::{
    AgentDrivenUpgradeWorkflow, AgentUpgradeWorkflowError,
    support::{
        AGENT_UPGRADE_WORKFLOW_INVALID_EXECUTED_BY_DID_REASON_CODE, require_non_empty,
        validate_did, validate_timestamp,
    },
};

pub(super) fn validate_execution_request(
    executed_by: &str,
    executed_at_unix: u64,
    operation_hash: &str,
) -> Result<(), AgentUpgradeWorkflowError> {
    validate_did(
        executed_by,
        "executed_by",
        AGENT_UPGRADE_WORKFLOW_INVALID_EXECUTED_BY_DID_REASON_CODE,
    )?;
    validate_timestamp("executed_at_unix", executed_at_unix)?;
    require_non_empty("operation_hash", operation_hash)
}

pub(super) fn current_governance_status(
    workflow: &mut AgentDrivenUpgradeWorkflow,
    proposal_id: &str,
    executed_at_unix: u64,
) -> Result<GovernanceProposalStatus, AgentUpgradeWorkflowError> {
    workflow
        .governance
        .evaluate(proposal_id, executed_at_unix)
        .map_err(AgentUpgradeWorkflowError::GovernanceWorkflow)
}

pub(super) fn validate_activation_window(
    proposal_id: &str,
    executed_at_unix: u64,
    governance_status: GovernanceProposalStatus,
    governance_approved_at_unix: Option<u64>,
    min_activation_delay_secs: u64,
) -> Result<(), AgentUpgradeWorkflowError> {
    require_approved_status(proposal_id, governance_status)?;
    let approved_at = require_approval_timestamp(proposal_id, governance_approved_at_unix)?;
    ensure_delay_elapsed(
        proposal_id,
        executed_at_unix,
        approved_at,
        min_activation_delay_secs,
    )
}

pub(super) fn require_approved_status(
    proposal_id: &str,
    governance_status: GovernanceProposalStatus,
) -> Result<(), AgentUpgradeWorkflowError> {
    if governance_status != GovernanceProposalStatus::Approved {
        return Err(AgentUpgradeWorkflowError::GovernanceStatusNotApproved {
            proposal_id: proposal_id.to_owned(),
            status: governance_status,
        });
    }
    Ok(())
}

pub(super) fn require_approval_timestamp(
    proposal_id: &str,
    governance_approved_at_unix: Option<u64>,
) -> Result<u64, AgentUpgradeWorkflowError> {
    governance_approved_at_unix.ok_or_else(|| {
        AgentUpgradeWorkflowError::MissingGovernanceApprovalTimestamp(proposal_id.to_owned())
    })
}

pub(super) fn ensure_delay_elapsed(
    proposal_id: &str,
    executed_at_unix: u64,
    governance_approved_at_unix: u64,
    min_activation_delay_secs: u64,
) -> Result<(), AgentUpgradeWorkflowError> {
    let earliest_activation_unix =
        governance_approved_at_unix.saturating_add(min_activation_delay_secs);
    if executed_at_unix < earliest_activation_unix {
        return Err(AgentUpgradeWorkflowError::ActivationDelayNotElapsed {
            proposal_id: proposal_id.to_owned(),
            earliest_activation_unix,
            attempted_activation_unix: executed_at_unix,
        });
    }
    Ok(())
}
