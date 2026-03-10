use super::channel_errors::ChannelSnapshotStoreError;
use super::channel_types::{ChannelMetadata, ChannelRecordSnapshot, ChannelSnapshot, ChannelType};

fn channel_type_code(channel_type: ChannelType) -> &'static str {
    match channel_type {
        ChannelType::Direct => "0",
        ChannelType::Group => "1",
        ChannelType::Broadcast => "2",
        ChannelType::Task => "3",
        ChannelType::Marketplace => "4",
        ChannelType::Governance => "5",
    }
}

fn parse_channel_type_code(raw: &str) -> Option<ChannelType> {
    match raw {
        "0" => Some(ChannelType::Direct),
        "1" => Some(ChannelType::Group),
        "2" => Some(ChannelType::Broadcast),
        "3" => Some(ChannelType::Task),
        "4" => Some(ChannelType::Marketplace),
        "5" => Some(ChannelType::Governance),
        _ => None,
    }
}

fn metadata_snapshot_value(metadata: &ChannelMetadata) -> &str {
    match metadata {
        ChannelMetadata::Direct | ChannelMetadata::Group => "",
        ChannelMetadata::Broadcast { topic } => topic,
        ChannelMetadata::Task { task_id } => task_id,
        ChannelMetadata::Marketplace { market_scope } => market_scope,
        ChannelMetadata::Governance { proposal_scope } => proposal_scope,
    }
}

fn parse_metadata_snapshot_value(
    channel_type: ChannelType,
    value: &str,
) -> Result<ChannelMetadata, ChannelSnapshotStoreError> {
    match channel_type {
        ChannelType::Direct => {
            if !value.is_empty() {
                return Err(ChannelSnapshotStoreError::InvalidPayload(
                    "direct channel metadata payload must be empty".to_owned(),
                ));
            }
            Ok(ChannelMetadata::Direct)
        }
        ChannelType::Group => {
            if !value.is_empty() {
                return Err(ChannelSnapshotStoreError::InvalidPayload(
                    "group channel metadata payload must be empty".to_owned(),
                ));
            }
            Ok(ChannelMetadata::Group)
        }
        ChannelType::Broadcast => Ok(ChannelMetadata::Broadcast {
            topic: value.to_owned(),
        }),
        ChannelType::Task => Ok(ChannelMetadata::Task {
            task_id: value.to_owned(),
        }),
        ChannelType::Marketplace => Ok(ChannelMetadata::Marketplace {
            market_scope: value.to_owned(),
        }),
        ChannelType::Governance => Ok(ChannelMetadata::Governance {
            proposal_scope: value.to_owned(),
        }),
    }
}

fn ensure_snapshot_token(value: &str, field: &str) -> Result<(), ChannelSnapshotStoreError> {
    if value.contains('|') || value.contains('\n') || value.contains('\r') || value.contains(',') {
        return Err(ChannelSnapshotStoreError::InvalidPayload(format!(
            "{field} contains unsupported delimiter characters"
        )));
    }
    Ok(())
}

pub(super) fn serialize_channel_snapshot(
    snapshot: &ChannelSnapshot,
) -> Result<String, ChannelSnapshotStoreError> {
    let mut payload = format!("schema|{}\n", snapshot.schema_version);
    for record in &snapshot.records {
        ensure_snapshot_token(&record.channel_id, "channel_id")?;
        let metadata_value = metadata_snapshot_value(&record.metadata);
        ensure_snapshot_token(metadata_value, "metadata")?;
        for member in &record.members {
            ensure_snapshot_token(member, "member")?;
        }
        for admin in &record.admins {
            ensure_snapshot_token(admin, "admin")?;
        }
        payload.push_str(&format!(
            "record|{}|{}|{}|{}|{}\n",
            record.channel_id,
            channel_type_code(record.channel_type),
            metadata_value,
            record.members.join(","),
            record.admins.join(",")
        ));
    }
    Ok(payload)
}

pub(super) fn parse_channel_snapshot_payload(
    payload: &str,
) -> Result<ChannelSnapshot, ChannelSnapshotStoreError> {
    let mut lines = payload.lines().filter(|line| !line.trim().is_empty());
    let Some(schema_line) = lines.next() else {
        return Err(ChannelSnapshotStoreError::InvalidPayload(
            "missing schema line".to_owned(),
        ));
    };

    let mut schema_parts = schema_line.split('|');
    let Some(schema_prefix) = schema_parts.next() else {
        return Err(ChannelSnapshotStoreError::InvalidPayload(
            schema_line.to_owned(),
        ));
    };
    let Some(schema_version_raw) = schema_parts.next() else {
        return Err(ChannelSnapshotStoreError::InvalidPayload(
            schema_line.to_owned(),
        ));
    };
    if schema_prefix != "schema" || schema_parts.next().is_some() {
        return Err(ChannelSnapshotStoreError::InvalidPayload(
            schema_line.to_owned(),
        ));
    }
    let schema_version = schema_version_raw
        .parse::<u16>()
        .map_err(|_| ChannelSnapshotStoreError::InvalidPayload(schema_line.to_owned()))?;

    let mut records = Vec::new();
    for line in lines {
        let mut parts = line.split('|');
        let Some(prefix) = parts.next() else {
            return Err(ChannelSnapshotStoreError::InvalidPayload(line.to_owned()));
        };
        if prefix != "record" {
            return Err(ChannelSnapshotStoreError::InvalidPayload(line.to_owned()));
        }
        let Some(channel_id) = parts.next() else {
            return Err(ChannelSnapshotStoreError::InvalidPayload(line.to_owned()));
        };
        let Some(type_code) = parts.next() else {
            return Err(ChannelSnapshotStoreError::InvalidPayload(line.to_owned()));
        };
        let Some(metadata_raw) = parts.next() else {
            return Err(ChannelSnapshotStoreError::InvalidPayload(line.to_owned()));
        };
        let Some(members_raw) = parts.next() else {
            return Err(ChannelSnapshotStoreError::InvalidPayload(line.to_owned()));
        };
        let Some(admins_raw) = parts.next() else {
            return Err(ChannelSnapshotStoreError::InvalidPayload(line.to_owned()));
        };
        if parts.next().is_some() {
            return Err(ChannelSnapshotStoreError::InvalidPayload(line.to_owned()));
        }

        let channel_type = parse_channel_type_code(type_code)
            .ok_or_else(|| ChannelSnapshotStoreError::InvalidPayload(line.to_owned()))?;
        let metadata = parse_metadata_snapshot_value(channel_type, metadata_raw)?;
        let members = if members_raw.is_empty() {
            Vec::new()
        } else {
            members_raw
                .split(',')
                .map(|value| value.to_owned())
                .collect::<Vec<_>>()
        };
        let admins = if admins_raw.is_empty() {
            Vec::new()
        } else {
            admins_raw
                .split(',')
                .map(|value| value.to_owned())
                .collect::<Vec<_>>()
        };

        records.push(ChannelRecordSnapshot {
            channel_id: channel_id.to_owned(),
            channel_type,
            metadata,
            members,
            admins,
        });
    }

    Ok(ChannelSnapshot {
        schema_version,
        records,
    })
}
