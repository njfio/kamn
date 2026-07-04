use super::*;

pub fn parse_task_operation_snapshot_payload(
    payload: &str,
) -> Result<TaskOperationSnapshot, TaskOperationSnapshotStoreError> {
    let mut lines = payload.lines().filter(|line| !line.trim().is_empty());
    let schema_version = parse_schema_line(lines.next())?;
    let tasks = lines
        .map(parse_task_line)
        .collect::<Result<Vec<_>, TaskOperationSnapshotStoreError>>()?;
    Ok(TaskOperationSnapshot {
        schema_version,
        tasks,
    })
}

pub fn parse_task_state_code(raw: &str) -> Option<TaskState> {
    match raw {
        "0" => Some(TaskState::Submitted),
        "1" => Some(TaskState::Accepted),
        "2" => Some(TaskState::Delegated),
        "3" => Some(TaskState::InProgress),
        "4" => Some(TaskState::InputRequired),
        "5" => Some(TaskState::Blocked),
        "6" => Some(TaskState::Completed),
        "7" => Some(TaskState::Failed),
        "8" => Some(TaskState::Cancelled),
        _ => None,
    }
}

pub fn parse_task_notice_code(raw: &str) -> Option<TaskOperationNoticeKind> {
    match raw {
        "0" => Some(TaskOperationNoticeKind::Submitted),
        "1" => Some(TaskOperationNoticeKind::Accepted),
        "2" => Some(TaskOperationNoticeKind::Delegated),
        "3" => Some(TaskOperationNoticeKind::Started),
        "4" => Some(TaskOperationNoticeKind::InputRequired),
        "5" => Some(TaskOperationNoticeKind::Blocked),
        "6" => Some(TaskOperationNoticeKind::Completed),
        "7" => Some(TaskOperationNoticeKind::Failed),
        "8" => Some(TaskOperationNoticeKind::Cancelled),
        _ => None,
    }
}

fn parse_schema_line(schema_line: Option<&str>) -> Result<u16, TaskOperationSnapshotStoreError> {
    let Some(schema_line) = schema_line else {
        return Err(TaskOperationSnapshotStoreError::InvalidPayload(
            "missing schema line".to_owned(),
        ));
    };
    let mut schema_parts = schema_line.split('|');
    let Some(schema_prefix) = schema_parts.next() else {
        return invalid_payload(schema_line);
    };
    let Some(schema_version_raw) = schema_parts.next() else {
        return invalid_payload(schema_line);
    };
    if schema_prefix != "schema" || schema_parts.next().is_some() {
        return invalid_payload(schema_line);
    }
    schema_version_raw
        .parse::<u16>()
        .map_err(|_| TaskOperationSnapshotStoreError::InvalidPayload(schema_line.to_owned()))
}

fn parse_task_line(
    line: &str,
) -> Result<TaskOperationRecordSnapshot, TaskOperationSnapshotStoreError> {
    let fields = split_task_fields(line)?;
    Ok(TaskOperationRecordSnapshot {
        task_id: fields[0].to_owned(),
        requester: fields[1].to_owned(),
        assignee: optional_field(fields[2]),
        description: fields[3].to_owned(),
        lifecycle_history: parse_lifecycle_history(fields[4], line)?,
        dependencies: parse_string_list(fields[5]),
        notices: parse_notice_history(fields[6], line)?,
    })
}

fn split_task_fields(line: &str) -> Result<[&str; 7], TaskOperationSnapshotStoreError> {
    let mut parts = line.split('|');
    if parts.next() != Some("task") {
        return invalid_payload(line);
    }
    let mut fields = [""; 7];
    for field in &mut fields {
        let Some(value) = parts.next() else {
            return invalid_payload(line);
        };
        *field = value;
    }
    if parts.next().is_some() {
        return invalid_payload(line);
    }
    Ok(fields)
}

fn parse_lifecycle_history(
    raw: &str,
    line: &str,
) -> Result<Vec<TaskState>, TaskOperationSnapshotStoreError> {
    if raw.is_empty() {
        return Ok(Vec::new());
    }
    raw.split(',')
        .map(|value| {
            parse_task_state_code(value)
                .ok_or_else(|| TaskOperationSnapshotStoreError::InvalidPayload(line.to_owned()))
        })
        .collect()
}

fn parse_notice_history(
    raw: &str,
    line: &str,
) -> Result<Vec<TaskOperationNoticeKind>, TaskOperationSnapshotStoreError> {
    if raw.is_empty() {
        return Ok(Vec::new());
    }
    raw.split(',')
        .map(|value| {
            parse_task_notice_code(value)
                .ok_or_else(|| TaskOperationSnapshotStoreError::InvalidPayload(line.to_owned()))
        })
        .collect()
}

fn parse_string_list(raw: &str) -> Vec<String> {
    if raw.is_empty() {
        return Vec::new();
    }
    raw.split(',').map(|value| value.to_owned()).collect()
}

fn optional_field(raw: &str) -> Option<String> {
    if raw.is_empty() {
        None
    } else {
        Some(raw.to_owned())
    }
}

fn invalid_payload<T>(line: &str) -> Result<T, TaskOperationSnapshotStoreError> {
    Err(TaskOperationSnapshotStoreError::InvalidPayload(
        line.to_owned(),
    ))
}
