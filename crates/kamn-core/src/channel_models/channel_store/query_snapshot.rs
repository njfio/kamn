use super::validation::validate_snapshot_record;
use super::*;

impl ChannelStore {
    /// Look up the channel type for a channel identifier.
    pub fn channel_type(&self, channel_id: &str) -> Result<ChannelType, ChannelModelError> {
        self.channels
            .get(channel_id)
            .map(|record| record.channel_type)
            .ok_or_else(|| ChannelModelError::NotFound(channel_id.to_owned()))
    }

    /// Return all channel members for a channel identifier.
    pub fn members(&self, channel_id: &str) -> Result<Vec<String>, ChannelModelError> {
        let record = self
            .channels
            .get(channel_id)
            .ok_or_else(|| ChannelModelError::NotFound(channel_id.to_owned()))?;
        Ok(record.members.iter().cloned().collect())
    }

    /// Return all channel admins for a channel identifier.
    pub fn admins(&self, channel_id: &str) -> Result<Vec<String>, ChannelModelError> {
        let record = self
            .channels
            .get(channel_id)
            .ok_or_else(|| ChannelModelError::NotFound(channel_id.to_owned()))?;
        Ok(record.admins.iter().cloned().collect())
    }

    /// Return channel IDs where the given DID is currently a member.
    pub fn channels_for_member(&self, member: &str) -> Vec<String> {
        self.channels_by_member
            .get(member)
            .map(|values| values.iter().cloned().collect())
            .unwrap_or_default()
    }

    /// Export all channels into a deterministic snapshot payload.
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

    /// Restore channel state from a validated snapshot payload.
    pub fn restore_snapshot(
        &mut self,
        snapshot: ChannelSnapshot,
    ) -> Result<(), ChannelSnapshotError> {
        if snapshot.schema_version != CHANNEL_SNAPSHOT_SCHEMA_VERSION {
            return Err(ChannelSnapshotError::SnapshotVersionMismatch {
                expected: CHANNEL_SNAPSHOT_SCHEMA_VERSION,
                found: snapshot.schema_version,
            });
        }

        let mut channels: BTreeMap<String, ChannelRecord> = BTreeMap::new();
        let mut channels_by_member: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
        for record_snapshot in snapshot.records {
            if channels.contains_key(&record_snapshot.channel_id) {
                return Err(ChannelSnapshotError::DuplicateChannelId(
                    record_snapshot.channel_id,
                ));
            }

            let record =
                validate_snapshot_record(&record_snapshot).map_err(ChannelSnapshotError::Model)?;

            let channel_id = record_snapshot.channel_id;
            for member in &record.members {
                channels_by_member
                    .entry(member.clone())
                    .or_insert_with(BTreeSet::new)
                    .insert(channel_id.clone());
            }
            channels.insert(channel_id, record);
        }

        self.channels = channels;
        self.channels_by_member = channels_by_member;
        Ok(())
    }

    /// Check whether a DID is currently a member of the given channel.
    pub fn is_member(&self, channel_id: &str, member: &str) -> Result<bool, ChannelModelError> {
        let record = self
            .channels
            .get(channel_id)
            .ok_or_else(|| ChannelModelError::NotFound(channel_id.to_owned()))?;
        Ok(record.members.contains(member))
    }

    /// Return metadata associated with the given channel.
    pub fn metadata(&self, channel_id: &str) -> Result<ChannelMetadata, ChannelModelError> {
        let record = self
            .channels
            .get(channel_id)
            .ok_or_else(|| ChannelModelError::NotFound(channel_id.to_owned()))?;
        Ok(record.metadata.clone())
    }
}
