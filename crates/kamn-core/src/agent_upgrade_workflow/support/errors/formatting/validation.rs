use std::fmt;

use crate::agent_upgrade_workflow::support::errors::AgentUpgradeWorkflowError;

pub(super) fn format_validation_error(
    error: &AgentUpgradeWorkflowError,
    f: &mut fmt::Formatter<'_>,
) -> Option<fmt::Result> {
    match error {
        AgentUpgradeWorkflowError::EmptyField(field) => {
            Some(write!(f, "field must not be empty: {field}"))
        }
        AgentUpgradeWorkflowError::InvalidDid {
            field,
            reason_code,
            detail,
        } => Some(write!(
            f,
            "invalid did field {field}: {reason_code} ({detail})"
        )),
        AgentUpgradeWorkflowError::InvalidTimestamp(field) => {
            Some(write!(f, "timestamp must be > 0: {field}"))
        }
        AgentUpgradeWorkflowError::InvalidDeadline {
            created_at_unix,
            voting_deadline_unix,
        } => Some(write!(
            f,
            "invalid voting deadline: created_at_unix={created_at_unix}, voting_deadline_unix={voting_deadline_unix}"
        )),
        _ => None,
    }
}
