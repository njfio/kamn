use crate::{ChannelPolicySnapshot, DeliveryGuardSnapshot, SqliteStoreBackend, SqliteStoreBackendError};

use super::super::wire_codec::deserialize_bundle;
use super::super::{
    default_bundle, validate_bundle, ChannelPolicySnapshotStore,
    DeliveryGuardSnapshotStore, DurableGuardBundleSnapshotStore, DurableGuardSnapshotBundle,
    DurableGuardSnapshotStoreError,
};
use std::path::PathBuf;

#[derive(Debug)]
/// Sqlite-backed durable snapshot store implementation.
pub struct SqliteDurableGuardSnapshotStore {
    backend: SqliteStoreBackend,
}

impl SqliteDurableGuardSnapshotStore {
    /// Creates a sqlite-backed durable snapshot store rooted at `path`.
    pub fn new(path: PathBuf) -> Result<Self, DurableGuardSnapshotStoreError> {
        let backend = SqliteStoreBackend::open(path.as_path()).map_err(map_sqlite_store_error)?;
        Ok(Self { backend })
    }

    fn load_or_default_bundle(
        &self,
    ) -> Result<DurableGuardSnapshotBundle, DurableGuardSnapshotStoreError> {
        Ok(self.load_bundle()?.unwrap_or_else(default_bundle))
    }
}

impl DurableGuardBundleSnapshotStore for SqliteDurableGuardSnapshotStore {
    fn save_bundle(
        &mut self,
        bundle: DurableGuardSnapshotBundle,
    ) -> Result<(), DurableGuardSnapshotStoreError> {
        validate_bundle(&bundle)?;
        let payload = super::super::wire_codec::serialize_bundle(&bundle)?;
        self.backend
            .put("durable_guard_snapshot_store", "latest", payload.as_bytes())
            .map_err(map_sqlite_store_error)?;
        Ok(())
    }

    fn load_bundle(
        &self,
    ) -> Result<Option<DurableGuardSnapshotBundle>, DurableGuardSnapshotStoreError> {
        let Some(payload_bytes) = self
            .backend
            .get("durable_guard_snapshot_store", "latest")
            .map_err(map_sqlite_store_error)?
        else {
            return Ok(None);
        };
        let payload = String::from_utf8(payload_bytes).map_err(|_| {
            DurableGuardSnapshotStoreError::InvalidPayload(
                "durable guard sqlite payload is not utf-8".to_owned(),
            )
        })?;
        if payload.trim().is_empty() {
            return Ok(None);
        }
        deserialize_bundle(payload.as_str()).map(Some)
    }
}

impl DeliveryGuardSnapshotStore for SqliteDurableGuardSnapshotStore {
    fn save_delivery_guard(
        &mut self,
        snapshot: DeliveryGuardSnapshot,
    ) -> Result<(), DurableGuardSnapshotStoreError> {
        let mut bundle = self.load_or_default_bundle()?;
        bundle.delivery_guard = snapshot;
        self.save_bundle(bundle)
    }

    fn load_delivery_guard(
        &self,
    ) -> Result<Option<DeliveryGuardSnapshot>, DurableGuardSnapshotStoreError> {
        Ok(self.load_bundle()?.map(|bundle| bundle.delivery_guard))
    }
}

impl ChannelPolicySnapshotStore for SqliteDurableGuardSnapshotStore {
    fn save_channel_policy(
        &mut self,
        snapshot: ChannelPolicySnapshot,
    ) -> Result<(), DurableGuardSnapshotStoreError> {
        let mut bundle = self.load_or_default_bundle()?;
        bundle.channel_policy = snapshot;
        self.save_bundle(bundle)
    }

    fn load_channel_policy(
        &self,
    ) -> Result<Option<ChannelPolicySnapshot>, DurableGuardSnapshotStoreError> {
        Ok(self.load_bundle()?.map(|bundle| bundle.channel_policy))
    }
}

fn map_sqlite_store_error(error: SqliteStoreBackendError) -> DurableGuardSnapshotStoreError {
    match error {
        SqliteStoreBackendError::SchemaVersionMissing => {
            DurableGuardSnapshotStoreError::InvalidPayload(
                "durable guard sqlite schema missing".to_owned(),
            )
        }
        SqliteStoreBackendError::SchemaVersionInvalid(value) => {
            DurableGuardSnapshotStoreError::InvalidPayload(format!(
                "durable guard sqlite schema invalid: {value}"
            ))
        }
        SqliteStoreBackendError::SchemaVersionMismatch { expected, found } => {
            DurableGuardSnapshotStoreError::InvalidPayload(format!(
                "durable guard sqlite schema mismatch: expected {expected}, found {found}"
            ))
        }
        SqliteStoreBackendError::InvalidPath => DurableGuardSnapshotStoreError::InvalidPayload(
            "snapshot file path cannot be empty".to_owned(),
        ),
        other => DurableGuardSnapshotStoreError::Io(other.to_string()),
    }
}
