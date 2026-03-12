mod authorization;
mod config;
mod state;
mod validation;
mod wrapped;

use std::fmt;

use crate::agent_upgrade_workflow::support::errors::AgentUpgradeWorkflowError;

pub(super) fn format_error(
    error: &AgentUpgradeWorkflowError,
    f: &mut fmt::Formatter<'_>,
) -> fmt::Result {
    validation::format_validation_error(error, f)
        .or_else(|| config::format_config_error(error, f))
        .or_else(|| authorization::format_authorization_error(error, f))
        .or_else(|| state::format_state_error(error, f))
        .or_else(|| wrapped::format_wrapped_error(error, f))
        .unwrap_or(Ok(()))
}
