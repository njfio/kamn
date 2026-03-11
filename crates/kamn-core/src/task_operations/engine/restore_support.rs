use super::*;

pub(super) fn validate_schema_version(found: u16) -> Result<(), TaskOperationError> {
    if found != TASK_OPERATION_SNAPSHOT_SCHEMA_VERSION {
        return Err(TaskOperationError::SnapshotVersionMismatch {
            expected: TASK_OPERATION_SNAPSHOT_SCHEMA_VERSION,
            found,
        });
    }
    Ok(())
}

pub(super) type RestoredMaps = (
    BTreeMap<String, TaskOperationRecord>,
    BTreeMap<String, Vec<TaskOperationNoticeKind>>,
    BTreeMap<String, BTreeSet<String>>,
);

pub(super) fn restored_task_state(
    restored_tasks: &BTreeMap<String, TaskOperationRecord>,
    task_id: &str,
) -> Result<TaskState, TaskOperationError> {
    restored_tasks
        .get(task_id)
        .map(|task| task.lifecycle.state())
        .ok_or_else(|| TaskOperationError::NotFound(task_id.to_owned()))
}
