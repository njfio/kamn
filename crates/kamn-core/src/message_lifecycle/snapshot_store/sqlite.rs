use super::*;

/// Sqlite-backed snapshot store for lifecycle state recovery.
#[derive(Debug)]
pub struct SqliteMessageLifecycleSnapshotStore {
    backend: SqliteStoreBackend,
}

impl SqliteMessageLifecycleSnapshotStore {
    /// Creates a sqlite-backed snapshot store at `path`.
    pub fn new(path: PathBuf) -> Result<Self, MessageLifecycleSnapshotStoreError> {
        let backend = SqliteStoreBackend::open(path.as_path()).map_err(map_sqlite_store_error)?;
        Ok(Self { backend })
    }
}

impl MessageLifecycleSnapshotStore for SqliteMessageLifecycleSnapshotStore {
    fn write(
        &mut self,
        snapshot: MessageLifecycleSnapshot,
    ) -> Result<(), MessageLifecycleSnapshotStoreError> {
        let mut verifier = MessageLifecycleStore::new();
        verifier
            .restore_snapshot(snapshot.clone())
            .map_err(MessageLifecycleSnapshotStoreError::Snapshot)?;
        let payload = codec::serialize_message_lifecycle_snapshot(&snapshot)?;
        self.backend
            .put(
                "message_lifecycle_snapshot_store",
                "latest",
                payload.as_bytes(),
            )
            .map_err(map_sqlite_store_error)?;
        Ok(())
    }

    fn read_latest(
        &self,
    ) -> Result<Option<MessageLifecycleSnapshot>, MessageLifecycleSnapshotStoreError> {
        let Some(payload_bytes) = self
            .backend
            .get("message_lifecycle_snapshot_store", "latest")
            .map_err(map_sqlite_store_error)?
        else {
            return Ok(None);
        };
        let payload = String::from_utf8(payload_bytes).map_err(|_| {
            MessageLifecycleSnapshotStoreError::InvalidPayload(
                "message lifecycle sqlite payload is not utf-8".to_owned(),
            )
        })?;
        if payload.trim().is_empty() {
            return Ok(None);
        }
        let snapshot = codec::parse_message_lifecycle_snapshot_payload(&payload)?;
        let mut verifier = MessageLifecycleStore::new();
        verifier
            .restore_snapshot(snapshot.clone())
            .map_err(MessageLifecycleSnapshotStoreError::Snapshot)?;
        Ok(Some(snapshot))
    }
}

fn map_sqlite_store_error(error: SqliteStoreBackendError) -> MessageLifecycleSnapshotStoreError {
    match error {
        SqliteStoreBackendError::SchemaVersionMissing => {
            MessageLifecycleSnapshotStoreError::InvalidPayload(
                "message lifecycle sqlite schema missing".to_owned(),
            )
        }
        SqliteStoreBackendError::SchemaVersionInvalid(value) => {
            MessageLifecycleSnapshotStoreError::InvalidPayload(format!(
                "message lifecycle sqlite schema invalid: {value}"
            ))
        }
        SqliteStoreBackendError::SchemaVersionMismatch { expected, found } => {
            MessageLifecycleSnapshotStoreError::InvalidPayload(format!(
                "message lifecycle sqlite schema mismatch: expected {expected}, found {found}"
            ))
        }
        SqliteStoreBackendError::InvalidPath => MessageLifecycleSnapshotStoreError::InvalidPayload(
            "snapshot file path cannot be empty".to_owned(),
        ),
        other => MessageLifecycleSnapshotStoreError::Io(other.to_string()),
    }
}
