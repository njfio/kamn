use crate::{GovernanceProposalStatus, GovernanceVoteRecord};

use crate::agent_upgrade_workflow::{
    AgentDrivenUpgradeWorkflow, AgentUpgradeAuditEvent, AgentUpgradeAuditEventKind,
    AgentUpgradeProposalRecord, AgentUpgradeProposalState, AgentUpgradeWorkflowError,
    support::apply_yes_votes_as_upgrade_approvals,
};

pub(super) fn execute_governance_action(
    workflow: &mut AgentDrivenUpgradeWorkflow,
    proposal_id: &str,
    executed_by: &str,
    executed_at_unix: u64,
    operation_hash: &str,
) -> Result<(), AgentUpgradeWorkflowError> {
    workflow
        .governance
        .execute(proposal_id, executed_by, executed_at_unix, operation_hash)
        .map(|_| ())
        .map_err(AgentUpgradeWorkflowError::GovernanceWorkflow)
}

pub(super) fn apply_activation_to_orchestrator(
    workflow: &mut AgentDrivenUpgradeWorkflow,
    proposal_id: &str,
    executed_by: &str,
    executed_at_unix: u64,
) -> Result<(), AgentUpgradeWorkflowError> {
    let votes = governance_votes(workflow, proposal_id)?;
    apply_approval_votes(workflow, proposal_id, votes)?;
    mark_orchestrator_approved(workflow, proposal_id, executed_at_unix)?;
    activate_orchestrator_upgrade(workflow, proposal_id, executed_by, executed_at_unix)
}

fn governance_votes(
    workflow: &mut AgentDrivenUpgradeWorkflow,
    proposal_id: &str,
) -> Result<Vec<GovernanceVoteRecord>, AgentUpgradeWorkflowError> {
    workflow
        .governance
        .vote_history(proposal_id)
        .map_err(AgentUpgradeWorkflowError::GovernanceWorkflow)
}

fn apply_approval_votes(
    workflow: &mut AgentDrivenUpgradeWorkflow,
    proposal_id: &str,
    votes: Vec<GovernanceVoteRecord>,
) -> Result<(), AgentUpgradeWorkflowError> {
    apply_yes_votes_as_upgrade_approvals(&mut workflow.orchestrator, proposal_id, votes)
        .map_err(AgentUpgradeWorkflowError::UpgradeOrchestration)
}

fn mark_orchestrator_approved(
    workflow: &mut AgentDrivenUpgradeWorkflow,
    proposal_id: &str,
    executed_at_unix: u64,
) -> Result<(), AgentUpgradeWorkflowError> {
    workflow
        .orchestrator
        .mark_governance_status(
            proposal_id,
            GovernanceProposalStatus::Approved,
            executed_at_unix,
        )
        .map_err(AgentUpgradeWorkflowError::UpgradeOrchestration)
}

fn activate_orchestrator_upgrade(
    workflow: &mut AgentDrivenUpgradeWorkflow,
    proposal_id: &str,
    executed_by: &str,
    executed_at_unix: u64,
) -> Result<(), AgentUpgradeWorkflowError> {
    workflow
        .orchestrator
        .activate_upgrade(proposal_id, executed_by, executed_at_unix)
        .map_err(AgentUpgradeWorkflowError::UpgradeOrchestration)
}

pub(super) fn record_activation(
    proposal: &mut AgentUpgradeProposalRecord,
    executed_at_unix: u64,
    operation_hash: &str,
) {
    proposal.governance_status = GovernanceProposalStatus::Executed;
    proposal.state = AgentUpgradeProposalState::Activated;
    proposal.activated_at_unix = Some(executed_at_unix);
    proposal.operation_hash = Some(operation_hash.to_owned());
}

pub(super) fn push_execution_events(
    workflow: &mut AgentDrivenUpgradeWorkflow,
    proposal_id: &str,
    executed_by: &str,
    executed_at_unix: u64,
) {
    workflow.events.push(governance_executed_event(
        proposal_id,
        executed_by,
        executed_at_unix,
    ));
    workflow.events.push(upgrade_activated_event(
        proposal_id,
        executed_by,
        executed_at_unix,
    ));
}

fn governance_executed_event(
    proposal_id: &str,
    executed_by: &str,
    executed_at_unix: u64,
) -> AgentUpgradeAuditEvent {
    AgentUpgradeAuditEvent {
        proposal_id: proposal_id.to_owned(),
        actor_did: executed_by.to_owned(),
        event_at_unix: executed_at_unix,
        kind: AgentUpgradeAuditEventKind::GovernanceExecuted,
        note: Some("governance execution completed".to_owned()),
    }
}

fn upgrade_activated_event(
    proposal_id: &str,
    executed_by: &str,
    executed_at_unix: u64,
) -> AgentUpgradeAuditEvent {
    AgentUpgradeAuditEvent {
        proposal_id: proposal_id.to_owned(),
        actor_did: executed_by.to_owned(),
        event_at_unix: executed_at_unix,
        kind: AgentUpgradeAuditEventKind::UpgradeActivated,
        note: Some("version upgrade activated".to_owned()),
    }
}
