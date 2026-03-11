use super::*;

pub(crate) fn map_sqlite_store_error(error: SqliteStoreBackendError) -> ChannelSnapshotStoreError {
    match error {
        SqliteStoreBackendError::SchemaVersionMissing => ChannelSnapshotStoreError::InvalidPayload(
            "channel snapshot sqlite schema missing".to_owned(),
        ),
        SqliteStoreBackendError::SchemaVersionInvalid(value) => {
            ChannelSnapshotStoreError::InvalidPayload(format!(
                "channel snapshot sqlite schema invalid: {value}"
            ))
        }
        SqliteStoreBackendError::SchemaVersionMismatch { expected, found } => {
            ChannelSnapshotStoreError::InvalidPayload(format!(
                "channel snapshot sqlite schema mismatch: expected {expected}, found {found}"
            ))
        }
        SqliteStoreBackendError::InvalidPath => ChannelSnapshotStoreError::InvalidPayload(
            "snapshot file path cannot be empty".to_owned(),
        ),
        other => ChannelSnapshotStoreError::Io(other.to_string()),
    }
}
