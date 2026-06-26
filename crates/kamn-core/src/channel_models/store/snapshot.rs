use super::*;
use crate::{ChannelModelError, ChannelSnapshotError};
use std::collections::{BTreeMap, BTreeSet};

impl ChannelStore {
    /// Exports the full channel state as a snapshot payload.
    pub fn export_snapshot(&self) -> ChannelSnapshot {
        let records = self
            .channels
            .iter()
            .map(|(channel_id, record)| ChannelRecordSnapshot {
                channel_id: channel_id.clone(),
                channel_type: record.channel_type,
                metadata: record.metadata.clone(),
                members: record.members.iter().cloned().collect(),
                admins: record.admins.iter().cloned().collect(),
            })
            .collect();
        ChannelSnapshot {
            schema_version: CHANNEL_SNAPSHOT_SCHEMA_VERSION,
            records,
        }
    }

    /// Restores the full channel state from a snapshot payload.
    pub fn restore_snapshot(
        &mut self,
        snapshot: ChannelSnapshot,
    ) -> Result<(), ChannelSnapshotError> {
        validate_snapshot_schema(snapshot.schema_version)?;
        let (channels, channels_by_member) = restore_snapshot_maps(snapshot.records)?;
        self.channels = channels;
        self.channels_by_member = channels_by_member;
        Ok(())
    }
}

fn validate_snapshot_schema(schema_version: u16) -> Result<(), ChannelSnapshotError> {
    if schema_version != CHANNEL_SNAPSHOT_SCHEMA_VERSION {
        return Err(ChannelSnapshotError::SnapshotVersionMismatch {
            expected: CHANNEL_SNAPSHOT_SCHEMA_VERSION,
            found: schema_version,
        });
    }
    Ok(())
}

type RestoredChannelMaps = (
    BTreeMap<String, ChannelRecord>,
    BTreeMap<String, BTreeSet<String>>,
);

fn restore_snapshot_maps(
    records: Vec<ChannelRecordSnapshot>,
) -> Result<RestoredChannelMaps, ChannelSnapshotError> {
    let mut channels = BTreeMap::new();
    let mut channels_by_member = BTreeMap::new();
    for record_snapshot in records {
        let channel_id = record_snapshot.channel_id.clone();
        if channels.contains_key(&channel_id) {
            return Err(ChannelSnapshotError::DuplicateChannelId(channel_id));
        }
        let record =
            validate_snapshot_record(&record_snapshot).map_err(ChannelSnapshotError::Model)?;
        index_snapshot_members(&mut channels_by_member, &channel_id, &record.members);
        channels.insert(channel_id, record);
    }
    Ok((channels, channels_by_member))
}

pub(crate) fn validate_snapshot_record(
    record: &ChannelRecordSnapshot,
) -> Result<ChannelRecord, ChannelModelError> {
    validate_channel_id(&record.channel_id)?;
    validate_snapshot_metadata(record)?;
    validate_snapshot_member_lists(record)?;
    let members = collect_snapshot_members(record)?;
    let admins = collect_snapshot_admins(record, &members)?;
    validate_channel_shape(record, &members, &admins)?;
    Ok(ChannelRecord {
        channel_type: record.channel_type,
        metadata: record.metadata.clone(),
        members,
        admins,
    })
}

fn validate_snapshot_metadata(record: &ChannelRecordSnapshot) -> Result<(), ChannelModelError> {
    if !metadata_matches_channel_type(record.channel_type, &record.metadata) {
        return Err(ChannelModelError::InvalidMetadata(
            "channel type and metadata variant mismatch".to_owned(),
        ));
    }
    validate_metadata(&record.metadata)
}

fn validate_snapshot_member_lists(record: &ChannelRecordSnapshot) -> Result<(), ChannelModelError> {
    if record.members.is_empty() {
        return Err(ChannelModelError::EmptyMembers);
    }
    if record.admins.is_empty() {
        return Err(ChannelModelError::EmptyAdmins);
    }
    Ok(())
}

fn collect_snapshot_members(
    record: &ChannelRecordSnapshot,
) -> Result<BTreeSet<String>, ChannelModelError> {
    let mut members = BTreeSet::new();
    for member in &record.members {
        validate_did(member)?;
        if !members.insert(member.clone()) {
            return Err(ChannelModelError::InvalidMetadata(
                "duplicate member DID in snapshot".to_owned(),
            ));
        }
    }
    Ok(members)
}

fn collect_snapshot_admins(
    record: &ChannelRecordSnapshot,
    members: &BTreeSet<String>,
) -> Result<BTreeSet<String>, ChannelModelError> {
    let mut admins = BTreeSet::new();
    for admin in &record.admins {
        validate_did(admin)?;
        if !members.contains(admin) {
            return Err(ChannelModelError::AdminNotMember(admin.clone()));
        }
        if !admins.insert(admin.clone()) {
            return Err(ChannelModelError::InvalidMetadata(
                "duplicate admin DID in snapshot".to_owned(),
            ));
        }
    }
    Ok(admins)
}

fn validate_channel_shape(
    record: &ChannelRecordSnapshot,
    members: &BTreeSet<String>,
    admins: &BTreeSet<String>,
) -> Result<(), ChannelModelError> {
    if record.channel_type == ChannelType::Direct {
        return validate_direct_snapshot_shape(members, admins);
    }
    enforce_specialized_member_requirements(record.channel_type, members.len())
}

fn validate_direct_snapshot_shape(
    members: &BTreeSet<String>,
    admins: &BTreeSet<String>,
) -> Result<(), ChannelModelError> {
    if members.len() != 2 || admins != members {
        return Err(ChannelModelError::InvalidDirectParticipants);
    }
    Ok(())
}

fn index_snapshot_members(
    channels_by_member: &mut BTreeMap<String, BTreeSet<String>>,
    channel_id: &str,
    members: &BTreeSet<String>,
) {
    for member in members {
        channels_by_member
            .entry(member.clone())
            .or_default()
            .insert(channel_id.to_owned());
    }
}
