use super::*;

pub(crate) fn serialize_channel_snapshot(
    snapshot: &ChannelSnapshot,
) -> Result<String, ChannelSnapshotStoreError> {
    let mut payload = format!("schema|{}\n", snapshot.schema_version);
    for record in &snapshot.records {
        validate_record_tokens(record)?;
        payload.push_str(&serialized_record_line(record));
    }
    Ok(payload)
}

fn validate_record_tokens(record: &ChannelRecordSnapshot) -> Result<(), ChannelSnapshotStoreError> {
    ensure_snapshot_token(&record.channel_id, "channel_id")?;
    ensure_snapshot_token(metadata_snapshot_value(&record.metadata), "metadata")?;
    validate_snapshot_set(&record.members, "member")?;
    validate_snapshot_set(&record.admins, "admin")
}

fn validate_snapshot_set(values: &[String], field: &str) -> Result<(), ChannelSnapshotStoreError> {
    for value in values {
        ensure_snapshot_token(value, field)?;
    }
    Ok(())
}

fn serialized_record_line(record: &ChannelRecordSnapshot) -> String {
    format!(
        "record|{}|{}|{}|{}|{}\n",
        record.channel_id,
        channel_type_code(record.channel_type),
        metadata_snapshot_value(&record.metadata),
        record.members.join(","),
        record.admins.join(",")
    )
}
