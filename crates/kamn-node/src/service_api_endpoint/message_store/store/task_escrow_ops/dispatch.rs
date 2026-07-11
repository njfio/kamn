use super::super::super::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct DispatchableTaskPayload {
    pub(super) creator_did: String,
    pub(super) task_type: String,
    pub(super) description: String,
}

pub(super) fn dispatch_request_from_record(
    record: &ServiceApiPersistedTaskRecord,
) -> Result<Option<DispatchableTaskPayload>, String> {
    match (
        record.creator_did.as_deref(),
        record.task_type.as_deref(),
        record.description.as_deref(),
    ) {
        (Some(creator_did), Some(task_type), Some(description)) => {
            if creator_did.trim().is_empty()
                || task_type.trim().is_empty()
                || description.trim().is_empty()
            {
                return Err("task dispatch metadata must not contain empty fields".to_owned());
            }
            Ok(Some(DispatchableTaskPayload {
                creator_did: creator_did.to_owned(),
                task_type: task_type.to_owned(),
                description: description.to_owned(),
            }))
        }
        (None, None, None) => Ok(None),
        _ => Err("task dispatch metadata must be complete".to_owned()),
    }
}

pub(super) fn select_dispatch_assignee(
    agents: &BTreeMap<String, ServiceApiPersistedAgentRecord>,
    request: &DispatchableTaskPayload,
) -> Option<String> {
    agents
        .values()
        .filter(|record| record.registered)
        .filter(|record| record.did != request.creator_did)
        .filter(|record| {
            record
                .capabilities
                .iter()
                .any(|value| value == &request.task_type)
        })
        .map(|record| record.did.clone())
        .min()
}

pub(super) fn dispatch_prerequisite_missing_error(task_type: &str) -> String {
    format!("task dispatch prerequisites missing: no registered worker for task_type {task_type}")
}
