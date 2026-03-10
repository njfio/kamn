use super::metadata::{channel_type_code, metadata_snapshot_value};
use super::support::ensure_snapshot_token;
use super::*;

pub(crate) fn serialize_channel_snapshot(
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
