use crate::message_lifecycle::{
    MessageLifecycleSnapshot, MessageLifecycleSnapshotStoreError, MessageRecordSnapshot,
    MessageStatus,
};

pub(crate) fn serialize_message_lifecycle_snapshot(
    snapshot: &MessageLifecycleSnapshot,
) -> Result<String, MessageLifecycleSnapshotStoreError> {
    let mut payload = format!("schema|{}\n", snapshot.schema_version);
    for record in &snapshot.records {
        payload.push_str(&serialize_message_lifecycle_record(record)?);
    }
    Ok(payload)
}

fn serialize_message_lifecycle_record(
    record: &MessageRecordSnapshot,
) -> Result<String, MessageLifecycleSnapshotStoreError> {
    validate_record_tokens(record)?;
    Ok(format!(
        "record|{}|{}|{}|{}|{}|{}|{}\n",
        record.message_id,
        record.sender,
        record.recipients.join(","),
        record.created,
        record.expires,
        message_status_code(record.status),
        serialize_status_history(&record.history)
    ))
}

fn validate_record_tokens(
    record: &MessageRecordSnapshot,
) -> Result<(), MessageLifecycleSnapshotStoreError> {
    ensure_snapshot_token(&record.message_id, "message_id", false)?;
    ensure_snapshot_token(&record.sender, "sender", false)?;
    ensure_snapshot_token(&record.created, "created", false)?;
    ensure_snapshot_token(&record.expires, "expires", false)?;
    for recipient in &record.recipients {
        ensure_snapshot_token(recipient, "recipient", false)?;
    }
    Ok(())
}

fn serialize_status_history(history: &[MessageStatus]) -> String {
    history
        .iter()
        .map(|status| message_status_code(*status))
        .collect::<Vec<_>>()
        .join(",")
}

fn ensure_snapshot_token(
    value: &str,
    field: &str,
    allow_comma: bool,
) -> Result<(), MessageLifecycleSnapshotStoreError> {
    let has_comma = !allow_comma && value.contains(',');
    if value.contains('|') || value.contains('\n') || value.contains('\r') || has_comma {
        return Err(MessageLifecycleSnapshotStoreError::InvalidPayload(format!(
            "{field} contains unsupported delimiter characters"
        )));
    }
    Ok(())
}

fn message_status_code(status: MessageStatus) -> &'static str {
    match status {
        MessageStatus::Created => "0",
        MessageStatus::Signed => "1",
        MessageStatus::Broadcast => "2",
        MessageStatus::Included => "3",
        MessageStatus::Delivered => "4",
        MessageStatus::Validated => "5",
        MessageStatus::Rejected => "6",
        MessageStatus::Expired => "7",
    }
}
