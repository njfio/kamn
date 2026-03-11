#![allow(missing_docs)]

use super::*;

#[derive(Debug)]
pub struct SqliteChannelSnapshotStore {
    backend: SqliteStoreBackend,
}

impl SqliteChannelSnapshotStore {
    pub fn new(path: PathBuf) -> Result<Self, ChannelSnapshotStoreError> {
        let backend = SqliteStoreBackend::open(path.as_path()).map_err(map_sqlite_store_error)?;
        Ok(Self { backend })
    }
}

impl ChannelSnapshotStore for SqliteChannelSnapshotStore {
    fn write(&mut self, snapshot: ChannelSnapshot) -> Result<(), ChannelSnapshotStoreError> {
        verify_snapshot(&snapshot)?;
        let payload = serialize_channel_snapshot(&snapshot)?;
        self.backend
            .put("channel_snapshot_store", "latest", payload.as_bytes())
            .map_err(map_sqlite_store_error)?;
        Ok(())
    }

    fn read_latest(&self) -> Result<Option<ChannelSnapshot>, ChannelSnapshotStoreError> {
        let Some(payload_bytes) = self
            .backend
            .get("channel_snapshot_store", "latest")
            .map_err(map_sqlite_store_error)?
        else {
            return Ok(None);
        };
        let payload = decode_sqlite_payload(payload_bytes)?;
        if payload.trim().is_empty() {
            return Ok(None);
        }
        let snapshot = parse_channel_snapshot_payload(&payload)?;
        verify_snapshot(&snapshot)?;
        Ok(Some(snapshot))
    }
}

fn verify_snapshot(snapshot: &ChannelSnapshot) -> Result<(), ChannelSnapshotStoreError> {
    let mut verifier = ChannelStore::new();
    verifier
        .restore_snapshot(snapshot.clone())
        .map_err(ChannelSnapshotStoreError::Snapshot)
}

fn decode_sqlite_payload(payload_bytes: Vec<u8>) -> Result<String, ChannelSnapshotStoreError> {
    String::from_utf8(payload_bytes).map_err(|_| {
        ChannelSnapshotStoreError::InvalidPayload(
            "channel snapshot sqlite payload is not utf-8".to_owned(),
        )
    })
}
