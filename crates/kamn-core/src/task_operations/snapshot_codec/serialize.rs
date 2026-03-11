use super::*;

pub fn ensure_snapshot_token(
    value: &str,
    field: &str,
    allow_comma: bool,
) -> Result<(), TaskOperationSnapshotStoreError> {
    let has_comma = !allow_comma && value.contains(',');
    if value.contains('|') || value.contains('\n') || value.contains('\r') || has_comma {
        return Err(TaskOperationSnapshotStoreError::InvalidPayload(format!(
            "{field} contains unsupported delimiter characters"
        )));
    }
    Ok(())
}

pub fn serialize_task_operation_snapshot(
    snapshot: &TaskOperationSnapshot,
) -> Result<String, TaskOperationSnapshotStoreError> {
    let mut payload = format!("schema|{}\n", snapshot.schema_version);
    for task in &snapshot.tasks {
        validate_snapshot_record(task)?;
        payload.push_str(serialize_task_line(task).as_str());
    }
    Ok(payload)
}

pub fn task_state_code(state: TaskState) -> &'static str {
    match state {
        TaskState::Submitted => "0",
        TaskState::Accepted => "1",
        TaskState::Delegated => "2",
        TaskState::InProgress => "3",
        TaskState::InputRequired => "4",
        TaskState::Blocked => "5",
        TaskState::Completed => "6",
        TaskState::Failed => "7",
        TaskState::Cancelled => "8",
    }
}

pub fn task_notice_code(notice: TaskOperationNoticeKind) -> &'static str {
    match notice {
        TaskOperationNoticeKind::Submitted => "0",
        TaskOperationNoticeKind::Accepted => "1",
        TaskOperationNoticeKind::Delegated => "2",
        TaskOperationNoticeKind::Started => "3",
        TaskOperationNoticeKind::InputRequired => "4",
        TaskOperationNoticeKind::Blocked => "5",
        TaskOperationNoticeKind::Completed => "6",
        TaskOperationNoticeKind::Failed => "7",
        TaskOperationNoticeKind::Cancelled => "8",
    }
}

fn validate_snapshot_record(
    task: &TaskOperationRecordSnapshot,
) -> Result<(), TaskOperationSnapshotStoreError> {
    ensure_snapshot_token(&task.task_id, "task_id", false)?;
    ensure_snapshot_token(&task.requester, "requester", false)?;
    if let Some(assignee) = &task.assignee {
        ensure_snapshot_token(assignee, "assignee", false)?;
    }
    ensure_snapshot_token(&task.description, "description", true)?;
    for dependency in &task.dependencies {
        ensure_snapshot_token(dependency, "dependency", false)?;
    }
    Ok(())
}

fn serialize_task_line(task: &TaskOperationRecordSnapshot) -> String {
    let assignee = task.assignee.clone().unwrap_or_default();
    let history = task
        .lifecycle_history
        .iter()
        .map(|state| task_state_code(*state))
        .collect::<Vec<_>>()
        .join(",");
    let dependencies = task.dependencies.join(",");
    let notices = task
        .notices
        .iter()
        .map(|notice| task_notice_code(*notice))
        .collect::<Vec<_>>()
        .join(",");
    format!(
        "task|{}|{}|{}|{}|{}|{}|{}\n",
        task.task_id, task.requester, assignee, task.description, history, dependencies, notices
    )
}
