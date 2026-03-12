use crate::{ChannelPolicySnapshot, DeliveryGuardSnapshot};

use super::super::{
    default_bundle, validate_bundle, ChannelPolicySnapshotStore,
    DeliveryGuardSnapshotStore, DurableGuardBundleSnapshotStore, DurableGuardSnapshotBundle,
    DurableGuardSnapshotStoreError,
};

#[derive(Debug, Clone, Default)]
/// In-memory durable snapshot store implementation for tests and local runtime.
pub struct InMemoryDurableGuardSnapshotStore {
    bundle: Option<DurableGuardSnapshotBundle>,
}

impl DurableGuardBundleSnapshotStore for InMemoryDurableGuardSnapshotStore {
    fn save_bundle(
        &mut self,
        bundle: DurableGuardSnapshotBundle,
    ) -> Result<(), DurableGuardSnapshotStoreError> {
        validate_bundle(&bundle)?;
        self.bundle = Some(bundle);
        Ok(())
    }

    fn load_bundle(
        &self,
    ) -> Result<Option<DurableGuardSnapshotBundle>, DurableGuardSnapshotStoreError> {
        if let Some(bundle) = &self.bundle {
            validate_bundle(bundle)?;
        }
        Ok(self.bundle.clone())
    }
}

impl DeliveryGuardSnapshotStore for InMemoryDurableGuardSnapshotStore {
    fn save_delivery_guard(
        &mut self,
        snapshot: DeliveryGuardSnapshot,
    ) -> Result<(), DurableGuardSnapshotStoreError> {
        let mut bundle = self.bundle.clone().unwrap_or_else(default_bundle);
        bundle.delivery_guard = snapshot;
        self.save_bundle(bundle)
    }

    fn load_delivery_guard(
        &self,
    ) -> Result<Option<DeliveryGuardSnapshot>, DurableGuardSnapshotStoreError> {
        Ok(self.load_bundle()?.map(|bundle| bundle.delivery_guard))
    }
}

impl ChannelPolicySnapshotStore for InMemoryDurableGuardSnapshotStore {
    fn save_channel_policy(
        &mut self,
        snapshot: ChannelPolicySnapshot,
    ) -> Result<(), DurableGuardSnapshotStoreError> {
        let mut bundle = self.bundle.clone().unwrap_or_else(default_bundle);
        bundle.channel_policy = snapshot;
        self.save_bundle(bundle)
    }

    fn load_channel_policy(
        &self,
    ) -> Result<Option<ChannelPolicySnapshot>, DurableGuardSnapshotStoreError> {
        Ok(self.load_bundle()?.map(|bundle| bundle.channel_policy))
    }
}
