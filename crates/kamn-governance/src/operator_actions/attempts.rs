use super::model::{OperatorActionAuditRecord, OperatorActionOutcome};
use crate::operator_binding::OperatorBindingAction;

pub fn binding_attempt(
    agent_did: &str,
    operator_did: &str,
    requested_at_unix: u64,
    action: OperatorBindingAction,
) -> (String, String, OperatorBindingAction, String, u64) {
    (
        agent_did.to_owned(),
        operator_did.to_owned(),
        action,
        "binding".to_owned(),
        requested_at_unix,
    )
}

pub fn allowed_record(
    agent_did: &str,
    operator_did: &str,
    action: OperatorBindingAction,
    target: &str,
    value: Option<String>,
    requested_at_unix: u64,
) -> OperatorActionAuditRecord {
    build_record(
        agent_did,
        operator_did,
        action,
        target,
        value,
        requested_at_unix,
        OperatorActionOutcome::Allowed,
    )
}

pub fn denied_record(
    agent_did: &str,
    operator_did: &str,
    action: OperatorBindingAction,
    target: &str,
    value: Option<String>,
    requested_at_unix: u64,
) -> OperatorActionAuditRecord {
    build_record(
        agent_did,
        operator_did,
        action,
        target,
        value,
        requested_at_unix,
        OperatorActionOutcome::Denied,
    )
}

fn build_record(
    agent_did: &str,
    operator_did: &str,
    action: OperatorBindingAction,
    target: &str,
    value: Option<String>,
    requested_at_unix: u64,
    outcome: OperatorActionOutcome,
) -> OperatorActionAuditRecord {
    OperatorActionAuditRecord {
        agent_did: agent_did.to_owned(),
        operator_did: operator_did.to_owned(),
        action,
        target: target.to_owned(),
        value,
        requested_at_unix,
        outcome,
    }
}
