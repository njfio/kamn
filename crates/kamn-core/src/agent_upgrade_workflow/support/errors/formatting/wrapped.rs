use std::fmt;

use crate::agent_upgrade_workflow::support::errors::AgentUpgradeWorkflowError;

pub(super) fn format_wrapped_error(
    error: &AgentUpgradeWorkflowError,
    f: &mut fmt::Formatter<'_>,
) -> Option<fmt::Result> {
    match error {
        AgentUpgradeWorkflowError::GovernanceWorkflow(error) => Some(write!(f, "{error}")),
        AgentUpgradeWorkflowError::UpgradeOrchestration(error) => Some(write!(f, "{error}")),
        _ => None,
    }
}
