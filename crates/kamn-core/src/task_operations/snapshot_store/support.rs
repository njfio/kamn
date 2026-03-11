use super::*;

pub fn map_sqlite_store_error(error: SqliteStoreBackendError) -> TaskOperationSnapshotStoreError {
    match error {
        SqliteStoreBackendError::SchemaVersionMissing => {
            TaskOperationSnapshotStoreError::InvalidPayload(
                "task operation sqlite schema missing".to_owned(),
            )
        }
        SqliteStoreBackendError::SchemaVersionInvalid(value) => {
            TaskOperationSnapshotStoreError::InvalidPayload(format!(
                "task operation sqlite schema invalid: {value}"
            ))
        }
        SqliteStoreBackendError::SchemaVersionMismatch { expected, found } => {
            TaskOperationSnapshotStoreError::InvalidPayload(format!(
                "task operation sqlite schema mismatch: expected {expected}, found {found}"
            ))
        }
        SqliteStoreBackendError::InvalidPath => TaskOperationSnapshotStoreError::InvalidPayload(
            "snapshot file path cannot be empty".to_owned(),
        ),
        other => TaskOperationSnapshotStoreError::Io(other.to_string()),
    }
}
