use super::metadata::{parse_channel_type_code, parse_metadata_snapshot_value};
use super::support::{invalid_payload, split_snapshot_list};
use super::*;

pub(crate) fn parse_channel_snapshot_payload(
    payload: &str,
) -> Result<ChannelSnapshot, ChannelSnapshotStoreError> {
    let mut lines = payload.lines().filter(|line| !line.trim().is_empty());
    let schema_line = lines.next().ok_or_else(missing_schema_line)?;
    let schema_version = parse_schema_line(schema_line)?;

    let mut records = Vec::new();
    for line in lines {
        records.push(parse_record_line(line)?);
    }

    Ok(ChannelSnapshot {
        schema_version,
        records,
    })
}

fn missing_schema_line() -> ChannelSnapshotStoreError {
    ChannelSnapshotStoreError::InvalidPayload("missing schema line".to_owned())
}

fn parse_schema_line(schema_line: &str) -> Result<u16, ChannelSnapshotStoreError> {
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
        .map_err(|_| ChannelSnapshotStoreError::InvalidPayload(schema_line.to_owned()))
}

fn parse_record_line(line: &str) -> Result<ChannelRecordSnapshot, ChannelSnapshotStoreError> {
    let mut parts = line.split('|');
    match (
        parts.next(),
        parts.next(),
        parts.next(),
        parts.next(),
        parts.next(),
        parts.next(),
        parts.next(),
    ) {
        (
            Some("record"),
            Some(channel_id),
            Some(type_code),
            Some(metadata_raw),
            Some(members_raw),
            Some(admins_raw),
            None,
        ) => build_snapshot_record(
            line,
            channel_id,
            type_code,
            metadata_raw,
            members_raw,
            admins_raw,
        ),
        _ => invalid_payload(line),
    }
}

fn build_snapshot_record(
    line: &str,
    channel_id: &str,
    type_code: &str,
    metadata_raw: &str,
    members_raw: &str,
    admins_raw: &str,
) -> Result<ChannelRecordSnapshot, ChannelSnapshotStoreError> {
    let channel_type = parse_channel_type_code(type_code)
        .ok_or_else(|| ChannelSnapshotStoreError::InvalidPayload(line.to_owned()))?;
    Ok(ChannelRecordSnapshot {
        channel_id: channel_id.to_owned(),
        channel_type,
        metadata: parse_metadata_snapshot_value(channel_type, metadata_raw)?,
        members: split_snapshot_list(members_raw),
        admins: split_snapshot_list(admins_raw),
    })
}
