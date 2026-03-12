use crate::{ChannelPolicySnapshot, DeliveryGuardSnapshot};

use super::super::wire_codec::{deserialize_bundle, serialize_bundle};
use super::super::{
    default_bundle, validate_bundle, ChannelPolicySnapshotStore,
    DeliveryGuardSnapshotStore, DurableGuardBundleSnapshotStore, DurableGuardSnapshotBundle,
    DurableGuardSnapshotStoreError,
};
use std::fs;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::PathBuf;

#[derive(Debug, Clone)]
/// File-backed durable snapshot store implementation.
pub struct FileDurableGuardSnapshotStore {
    path: PathBuf,
}

impl FileDurableGuardSnapshotStore {
    /// Creates a file-backed snapshot store rooted at `path`.
    pub fn new(path: PathBuf) -> Result<Self, DurableGuardSnapshotStoreError> {
        if path.is_dir() {
            return Err(DurableGuardSnapshotStoreError::InvalidPayload(
                "snapshot file path must not be a directory".to_owned(),
            ));
        }
        Ok(Self { path })
    }

    fn load_or_default_bundle(
        &self,
    ) -> Result<DurableGuardSnapshotBundle, DurableGuardSnapshotStoreError> {
        Ok(self.load_bundle()?.unwrap_or_else(default_bundle))
    }
}

impl DurableGuardBundleSnapshotStore for FileDurableGuardSnapshotStore {
    fn save_bundle(
        &mut self,
        bundle: DurableGuardSnapshotBundle,
    ) -> Result<(), DurableGuardSnapshotStoreError> {
        validate_bundle(&bundle)?;
        let payload = serialize_bundle(&bundle)?;
        let mut file = OpenOptions::new()
            .create(true)
            .truncate(true)
            .write(true)
            .open(&self.path)
            .map_err(|error| DurableGuardSnapshotStoreError::Io(error.to_string()))?;
        file.write_all(payload.as_bytes())
            .map_err(|error| DurableGuardSnapshotStoreError::Io(error.to_string()))
    }

    fn load_bundle(
        &self,
    ) -> Result<Option<DurableGuardSnapshotBundle>, DurableGuardSnapshotStoreError> {
        if !self.path.exists() {
            return Ok(None);
        }
        let payload = fs::read_to_string(&self.path)
            .map_err(|error| DurableGuardSnapshotStoreError::Io(error.to_string()))?;
        if payload.trim().is_empty() {
            return Ok(None);
        }
        deserialize_bundle(&payload).map(Some)
    }
}

impl DeliveryGuardSnapshotStore for FileDurableGuardSnapshotStore {
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

impl ChannelPolicySnapshotStore for FileDurableGuardSnapshotStore {
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
