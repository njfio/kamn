mod execution;
mod validation;

use crate::agent_upgrade_workflow::{
    AgentDrivenUpgradeWorkflow, AgentUpgradeProposalRecord, AgentUpgradeWorkflowError,
};
use execution::{
    apply_activation_to_orchestrator, execute_governance_action, push_execution_events,
    record_activation,
};
use validation::{
    current_governance_status, validate_activation_window, validate_execution_request,
};

impl AgentDrivenUpgradeWorkflow {
    /// Execute governance-approved proposal and activate the runtime upgrade once delay passes.
    pub fn finalize_upgrade(
        &mut self,
        proposal_id: &str,
        executed_by: &str,
        executed_at_unix: u64,
        operation_hash: &str,
    ) -> Result<(), AgentUpgradeWorkflowError> {
        let mut proposal = prepare_activation(
            self,
            proposal_id,
            executed_by,
            executed_at_unix,
            operation_hash,
        )?;
        complete_activation(
            self,
            &mut proposal,
            proposal_id,
            executed_by,
            executed_at_unix,
            operation_hash,
        )
    }
}

fn complete_activation(
    workflow: &mut AgentDrivenUpgradeWorkflow,
    proposal: &mut AgentUpgradeProposalRecord,
    proposal_id: &str,
    executed_by: &str,
    executed_at_unix: u64,
    operation_hash: &str,
) -> Result<(), AgentUpgradeWorkflowError> {
    execute_activation(
        workflow,
        proposal_id,
        executed_by,
        executed_at_unix,
        operation_hash,
    )?;
    apply_activation_to_orchestrator(workflow, proposal_id, executed_by, executed_at_unix)?;
    record_activation(proposal, executed_at_unix, operation_hash);
    workflow
        .proposals
        .insert(proposal_id.to_owned(), proposal.clone());
    push_execution_events(workflow, proposal_id, executed_by, executed_at_unix);
    Ok(())
}

fn load_proposal_for_activation(
    workflow: &AgentDrivenUpgradeWorkflow,
    proposal_id: &str,
) -> Result<AgentUpgradeProposalRecord, AgentUpgradeWorkflowError> {
    workflow
        .proposals
        .get(proposal_id)
        .cloned()
        .ok_or_else(|| AgentUpgradeWorkflowError::ProposalNotFound(proposal_id.to_owned()))
}

fn prepare_activation(
    workflow: &mut AgentDrivenUpgradeWorkflow,
    proposal_id: &str,
    executed_by: &str,
    executed_at_unix: u64,
    operation_hash: &str,
) -> Result<AgentUpgradeProposalRecord, AgentUpgradeWorkflowError> {
    validate_execution_request(executed_by, executed_at_unix, operation_hash)?;
    let proposal = load_proposal_for_activation(workflow, proposal_id)?;
    let governance_status = current_governance_status(workflow, proposal_id, executed_at_unix)?;
    validate_activation_window(
        proposal_id,
        executed_at_unix,
        governance_status,
        proposal.governance_approved_at_unix,
        workflow.min_activation_delay_secs,
    )?;
    Ok(proposal)
}

fn execute_activation(
    workflow: &mut AgentDrivenUpgradeWorkflow,
    proposal_id: &str,
    executed_by: &str,
    executed_at_unix: u64,
    operation_hash: &str,
) -> Result<(), AgentUpgradeWorkflowError> {
    execute_governance_action(
        workflow,
        proposal_id,
        executed_by,
        executed_at_unix,
        operation_hash,
    )
}
