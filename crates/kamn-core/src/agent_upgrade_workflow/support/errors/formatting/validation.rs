use std::fmt;

use crate::agent_upgrade_workflow::support::errors::AgentUpgradeWorkflowError;

pub(super) fn format_validation_error(
    error: &AgentUpgradeWorkflowError,
    f: &mut fmt::Formatter<'_>,
) -> Option<fmt::Result> {
    match error {
        AgentUpgradeWorkflowError::EmptyField(field) => Some(format_empty_field(f, field)),
        AgentUpgradeWorkflowError::InvalidDid {
            field,
            reason_code,
            detail,
        } => Some(format_invalid_did(f, field, reason_code, detail)),
        AgentUpgradeWorkflowError::InvalidTimestamp(field) => {
            Some(format_invalid_timestamp(f, field))
        }
        AgentUpgradeWorkflowError::InvalidDeadline {
            created_at_unix,
            voting_deadline_unix,
        } => Some(format_invalid_deadline(
            f,
            *created_at_unix,
            *voting_deadline_unix,
        )),
        _ => None,
    }
}

fn format_empty_field(f: &mut fmt::Formatter<'_>, field: &str) -> fmt::Result {
    write!(f, "field must not be empty: {field}")
}

fn format_invalid_did(
    f: &mut fmt::Formatter<'_>,
    field: &str,
    reason_code: &str,
    detail: &str,
) -> fmt::Result {
    write!(f, "invalid did field {field}: {reason_code} ({detail})")
}

fn format_invalid_timestamp(f: &mut fmt::Formatter<'_>, field: &str) -> fmt::Result {
    write!(f, "timestamp must be > 0: {field}")
}

fn format_invalid_deadline(
    f: &mut fmt::Formatter<'_>,
    created_at_unix: u64,
    voting_deadline_unix: u64,
) -> fmt::Result {
    write!(
        f,
        "invalid voting deadline: created_at_unix={created_at_unix}, voting_deadline_unix={voting_deadline_unix}"
    )
}
