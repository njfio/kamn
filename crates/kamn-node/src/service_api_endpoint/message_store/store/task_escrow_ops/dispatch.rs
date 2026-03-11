use super::super::super::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct DispatchableTaskPayload {
    pub(super) creator_did: String,
    pub(super) task_type: String,
    pub(super) description: String,
}

pub(super) fn parse_dispatchable_task_payload(
    payload: &str,
) -> Result<Option<DispatchableTaskPayload>, String> {
    let root = match serde_json::from_str::<serde_json::Value>(payload) {
        Ok(root) => root,
        Err(_) => return Ok(None),
    };
    let Some(object) = root.as_object() else {
        return Ok(None);
    };
    let has_dispatch_keys = object.contains_key("creator") || object.contains_key("task_type");
    if !has_dispatch_keys {
        return Ok(None);
    }
    Ok(Some(DispatchableTaskPayload {
        creator_did: required_dispatch_field(object, "creator")?,
        task_type: required_dispatch_field(object, "task_type")?,
        description: required_dispatch_field(object, "description")?,
    }))
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

fn required_dispatch_field(
    object: &serde_json::Map<String, serde_json::Value>,
    field: &str,
) -> Result<String, String> {
    let Some(value) = object.get(field).and_then(serde_json::Value::as_str) else {
        return Err(format!(
            "task dispatch payload field `{field}` must be string"
        ));
    };
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Err(format!(
            "task dispatch payload field `{field}` must not be empty"
        ));
    }
    Ok(trimmed.to_owned())
}
