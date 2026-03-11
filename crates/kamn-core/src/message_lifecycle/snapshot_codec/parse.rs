use crate::message_lifecycle::{
    MessageLifecycleSnapshot, MessageLifecycleSnapshotStoreError, MessageRecordSnapshot,
    MessageStatus,
};

const MESSAGE_LIFECYCLE_SNAPSHOT_JOURNAL_CORRUPT_TAIL_PREFIX: &str =
    "message_lifecycle_snapshot_journal_corrupt_tail";

struct ParsedMessageLifecycleRecordFields<'a> {
    message_id: &'a str,
    sender: &'a str,
    recipients_raw: &'a str,
    created: &'a str,
    expires: &'a str,
    status_raw: &'a str,
    history_raw: &'a str,
}

pub(crate) fn parse_message_lifecycle_snapshot_payload(
    payload: &str,
) -> Result<MessageLifecycleSnapshot, MessageLifecycleSnapshotStoreError> {
    let mut lines = payload.lines().filter(|line| !line.trim().is_empty());
    let schema_line = lines
        .next()
        .ok_or_else(|| invalid_message_lifecycle_snapshot_payload("missing schema line"))?;
    let schema_version = parse_message_lifecycle_snapshot_schema(schema_line)?;
    let records = lines
        .map(parse_message_lifecycle_snapshot_record)
        .collect::<Result<Vec<_>, _>>()?;
    Ok(MessageLifecycleSnapshot {
        schema_version,
        records,
    })
}

pub(crate) fn message_lifecycle_snapshot_journal_corrupt_tail(
    index: usize,
) -> MessageLifecycleSnapshotStoreError {
    MessageLifecycleSnapshotStoreError::InvalidPayload(format!(
        "{MESSAGE_LIFECYCLE_SNAPSHOT_JOURNAL_CORRUPT_TAIL_PREFIX}:{index}"
    ))
}

pub(crate) fn invalid_message_lifecycle_snapshot_payload(
    value: &str,
) -> MessageLifecycleSnapshotStoreError {
    MessageLifecycleSnapshotStoreError::InvalidPayload(value.to_owned())
}

fn parse_message_lifecycle_snapshot_schema(
    schema_line: &str,
) -> Result<u16, MessageLifecycleSnapshotStoreError> {
    match schema_line.split('|').collect::<Vec<_>>().as_slice() {
        ["schema", schema_version_raw] => schema_version_raw
            .parse::<u16>()
            .map_err(|_| invalid_message_lifecycle_snapshot_payload(schema_line)),
        _ => Err(invalid_message_lifecycle_snapshot_payload(schema_line)),
    }
}

fn parse_message_lifecycle_snapshot_record(
    line: &str,
) -> Result<MessageRecordSnapshot, MessageLifecycleSnapshotStoreError> {
    let fields = parse_message_lifecycle_snapshot_record_fields(line)?;
    Ok(MessageRecordSnapshot {
        message_id: fields.message_id.to_owned(),
        sender: fields.sender.to_owned(),
        recipients: parse_message_lifecycle_snapshot_recipients(fields.recipients_raw),
        created: fields.created.to_owned(),
        expires: fields.expires.to_owned(),
        status: parse_message_lifecycle_snapshot_status(fields.status_raw, line)?,
        history: parse_message_lifecycle_snapshot_status_history(fields.history_raw, line)?,
    })
}

fn parse_message_lifecycle_snapshot_record_fields(
    line: &str,
) -> Result<ParsedMessageLifecycleRecordFields<'_>, MessageLifecycleSnapshotStoreError> {
    match line.split('|').collect::<Vec<_>>().as_slice() {
        ["record", message_id, sender, recipients_raw, created, expires, status_raw, history_raw] => {
            Ok(ParsedMessageLifecycleRecordFields {
                message_id,
                sender,
                recipients_raw,
                created,
                expires,
                status_raw,
                history_raw,
            })
        }
        _ => Err(invalid_message_lifecycle_snapshot_payload(line)),
    }
}

fn parse_message_lifecycle_snapshot_recipients(raw: &str) -> Vec<String> {
    if raw.is_empty() {
        return Vec::new();
    }
    raw.split(',').map(|value| value.to_owned()).collect()
}

fn parse_message_lifecycle_snapshot_status(
    raw: &str,
    line: &str,
) -> Result<MessageStatus, MessageLifecycleSnapshotStoreError> {
    parse_message_status_code(raw).ok_or_else(|| invalid_message_lifecycle_snapshot_payload(line))
}

fn parse_message_lifecycle_snapshot_status_history(
    raw: &str,
    line: &str,
) -> Result<Vec<MessageStatus>, MessageLifecycleSnapshotStoreError> {
    if raw.is_empty() {
        return Ok(Vec::new());
    }
    raw.split(',')
        .map(|value| parse_message_lifecycle_snapshot_status(value, line))
        .collect()
}

fn parse_message_status_code(raw: &str) -> Option<MessageStatus> {
    match raw {
        "0" => Some(MessageStatus::Created),
        "1" => Some(MessageStatus::Signed),
        "2" => Some(MessageStatus::Broadcast),
        "3" => Some(MessageStatus::Included),
        "4" => Some(MessageStatus::Delivered),
        "5" => Some(MessageStatus::Validated),
        "6" => Some(MessageStatus::Rejected),
        "7" => Some(MessageStatus::Expired),
        _ => None,
    }
}
