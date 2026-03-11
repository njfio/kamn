use super::*;

pub(crate) fn parse_channel_snapshot_payload(
    payload: &str,
) -> Result<ChannelSnapshot, ChannelSnapshotStoreError> {
    let mut lines = payload.lines().filter(|line| !line.trim().is_empty());
    let schema_line = lines.next().ok_or_else(|| {
        ChannelSnapshotStoreError::InvalidPayload("missing schema line".to_owned())
    })?;
    let schema_version = parse_schema_line(schema_line)?;
    let records = lines
        .map(parse_record_line)
        .collect::<Result<Vec<_>, _>>()?;
    Ok(ChannelSnapshot {
        schema_version,
        records,
    })
}

fn parse_schema_line(schema_line: &str) -> Result<u16, ChannelSnapshotStoreError> {
    let mut schema_parts = schema_line.split('|');
    let schema_prefix = require_part(schema_parts.next(), schema_line)?;
    let schema_version_raw = require_part(schema_parts.next(), schema_line)?;
    if schema_prefix != "schema" || schema_parts.next().is_some() {
        return invalid_payload(schema_line);
    }
    schema_version_raw
        .parse::<u16>()
        .map_err(|_| ChannelSnapshotStoreError::InvalidPayload(schema_line.to_owned()))
}

fn parse_record_line(line: &str) -> Result<ChannelRecordSnapshot, ChannelSnapshotStoreError> {
    let mut parts = line.split('|');
    ensure_record_prefix(require_part(parts.next(), line)?, line)?;
    let channel_id = require_part(parts.next(), line)?;
    let type_code = require_part(parts.next(), line)?;
    let metadata_raw = require_part(parts.next(), line)?;
    let members_raw = require_part(parts.next(), line)?;
    let admins_raw = require_part(parts.next(), line)?;
    if parts.next().is_some() {
        return invalid_payload(line);
    }
    Ok(ChannelRecordSnapshot {
        channel_id: channel_id.to_owned(),
        channel_type: parse_record_channel_type(type_code, line)?,
        metadata: parse_record_metadata(type_code, metadata_raw, line)?,
        members: split_snapshot_values(members_raw),
        admins: split_snapshot_values(admins_raw),
    })
}

fn require_part<'a>(
    part: Option<&'a str>,
    line: &str,
) -> Result<&'a str, ChannelSnapshotStoreError> {
    part.ok_or_else(|| ChannelSnapshotStoreError::InvalidPayload(line.to_owned()))
}

fn ensure_record_prefix(prefix: &str, line: &str) -> Result<(), ChannelSnapshotStoreError> {
    if prefix != "record" {
        return invalid_payload(line);
    }
    Ok(())
}

fn parse_record_channel_type(
    type_code: &str,
    line: &str,
) -> Result<ChannelType, ChannelSnapshotStoreError> {
    parse_channel_type_code(type_code)
        .ok_or_else(|| ChannelSnapshotStoreError::InvalidPayload(line.to_owned()))
}

fn parse_record_metadata(
    type_code: &str,
    metadata_raw: &str,
    line: &str,
) -> Result<ChannelMetadata, ChannelSnapshotStoreError> {
    let channel_type = parse_record_channel_type(type_code, line)?;
    parse_metadata_snapshot_value(channel_type, metadata_raw)
}

fn split_snapshot_values(raw: &str) -> Vec<String> {
    if raw.is_empty() {
        return Vec::new();
    }
    raw.split(',').map(|value| value.to_owned()).collect()
}

fn invalid_payload<T>(line: &str) -> Result<T, ChannelSnapshotStoreError> {
    Err(ChannelSnapshotStoreError::InvalidPayload(line.to_owned()))
}
