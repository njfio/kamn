use std::fmt;

use crate::agent_upgrade_workflow::support::errors::AgentUpgradeWorkflowError;

pub(super) fn format_config_error(
    error: &AgentUpgradeWorkflowError,
    f: &mut fmt::Formatter<'_>,
) -> Option<fmt::Result> {
    match error {
        AgentUpgradeWorkflowError::InvalidRequiredHumanReviews(value) => {
            Some(write!(f, "invalid required human reviews: {value}"))
        }
        AgentUpgradeWorkflowError::InvalidRequiredValidatorQuorum(value) => {
            Some(write!(f, "invalid required validator quorum: {value}"))
        }
        AgentUpgradeWorkflowError::InvalidMinActivationDelaySecs(value) => Some(write!(
            f,
            "invalid minimum activation delay seconds: {value}"
        )),
        AgentUpgradeWorkflowError::MissingAllowedAgentProposers => {
            Some(write!(f, "allowed agent proposer set must not be empty"))
        }
        AgentUpgradeWorkflowError::MissingAllowedValidatorVoters => {
            Some(write!(f, "allowed validator voter set must not be empty"))
        }
        _ => None,
    }
}
