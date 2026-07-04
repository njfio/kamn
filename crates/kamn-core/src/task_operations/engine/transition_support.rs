use super::*;

pub(super) fn ensure_assignable(
    record: &TaskOperationRecord,
    actor: &str,
) -> Result<(), TaskOperationError> {
    if let Some(current) = record.assignee.as_deref() {
        if current != actor {
            return Err(TaskOperationError::UnauthorizedActor {
                actor: actor.to_owned(),
                required: "unassigned_or_current_assignee",
            });
        }
    }
    Ok(())
}

pub(super) fn ensure_requester_or_assignee(
    record: &TaskOperationRecord,
    actor: &str,
) -> Result<(), TaskOperationError> {
    let is_requester = record.requester == actor;
    let is_assignee = record.assignee.as_deref() == Some(actor);
    if !is_requester && !is_assignee {
        return Err(TaskOperationError::UnauthorizedActor {
            actor: actor.to_owned(),
            required: "requester_or_assignee",
        });
    }
    Ok(())
}

pub(super) fn ensure_reason_present(
    reason: &str,
    action: &'static str,
) -> Result<(), TaskOperationError> {
    if reason.trim().is_empty() {
        return Err(TaskOperationError::EmptyReason(action));
    }
    Ok(())
}

pub(super) fn apply_transition(
    record: &mut TaskOperationRecord,
    transition: TaskTransition,
) -> Result<(), TaskOperationError> {
    record
        .lifecycle
        .transition(transition)
        .map_err(lifecycle_error)
}

pub(super) fn ready_task_id(
    engine: &TaskOperationEngine,
    task_id: &str,
    record: &TaskOperationRecord,
) -> Option<String> {
    let state = record.lifecycle.state();
    if state != TaskState::Accepted && state != TaskState::Delegated {
        return None;
    }
    if engine
        .unsatisfied_dependency(task_id)
        .ok()
        .flatten()
        .is_some()
    {
        return None;
    }
    Some(task_id.to_owned())
}
