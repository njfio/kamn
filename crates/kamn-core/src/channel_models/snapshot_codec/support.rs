use super::*;

pub(super) fn ensure_snapshot_token(
    value: &str,
    field: &str,
) -> Result<(), ChannelSnapshotStoreError> {
    if value.contains('|') || value.contains('\n') || value.contains('\r') || value.contains(',') {
        return Err(ChannelSnapshotStoreError::InvalidPayload(format!(
            "{field} contains unsupported delimiter characters"
        )));
    }
    Ok(())
}

pub(super) fn split_snapshot_list(raw: &str) -> Vec<String> {
    if raw.is_empty() {
        return Vec::new();
    }
    raw.split(',').map(|value| value.to_owned()).collect()
}

pub(super) fn invalid_payload<T>(line: &str) -> Result<T, ChannelSnapshotStoreError> {
    Err(ChannelSnapshotStoreError::InvalidPayload(line.to_owned()))
}
