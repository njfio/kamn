use crate::{
    ChannelPermissionEngine, ChannelPolicySnapshot, ChannelPolicySnapshotError,
    DeliveryGuardSnapshot, DeliveryGuardSnapshotError, MessageDeliveryGuards,
};
use std::fmt;

/// Schema version for serialized durable guard bundles.
pub const DURABLE_GUARD_BUNDLE_SCHEMA_VERSION: u16 = 1;

#[derive(Debug, Clone, PartialEq, Eq)]
/// Combined snapshot bundle for delivery guards and channel policy state.
pub struct DurableGuardSnapshotBundle {
    /// Bundle schema version.
    pub schema_version: u16,
    /// Delivery guard snapshot payload.
    pub delivery_guard: DeliveryGuardSnapshot,
    /// Channel policy snapshot payload.
    pub channel_policy: ChannelPolicySnapshot,
}

impl DurableGuardSnapshotBundle {
    /// Captures a snapshot bundle from live guard engines.
    pub fn capture(
        delivery_guards: &MessageDeliveryGuards,
        channel_permissions: &ChannelPermissionEngine,
    ) -> Self {
        Self {
            schema_version: DURABLE_GUARD_BUNDLE_SCHEMA_VERSION,
            delivery_guard: delivery_guards.export_snapshot(),
            channel_policy: channel_permissions.export_snapshot(),
        }
    }

    /// Restores snapshot bundle into live guard engines.
    pub fn restore_into(
        self,
        delivery_guards: &mut MessageDeliveryGuards,
        channel_permissions: &mut ChannelPermissionEngine,
    ) -> Result<(), DurableGuardSnapshotStoreError> {
        ensure_schema_version(self.schema_version)?;
        delivery_guards
            .restore_snapshot(self.delivery_guard)
            .map_err(DurableGuardSnapshotStoreError::DeliverySnapshot)?;
        channel_permissions
            .restore_snapshot(self.channel_policy)
            .map_err(DurableGuardSnapshotStoreError::ChannelPolicySnapshot)?;
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// Durable snapshot store error taxonomy.
pub enum DurableGuardSnapshotStoreError {
    /// Bundle schema version does not match current runtime schema.
    BundleSchemaVersionMismatch {
        /// Expected schema version.
        expected: u16,
        /// Found schema version in payload.
        found: u16,
    },
    /// I/O failure while reading or writing snapshot payload.
    Io(String),
    /// Payload failed structural or semantic validation.
    InvalidPayload(String),
    /// Delivery snapshot payload could not be restored.
    DeliverySnapshot(DeliveryGuardSnapshotError),
    /// Channel-policy snapshot payload could not be restored.
    ChannelPolicySnapshot(ChannelPolicySnapshotError),
}

impl fmt::Display for DurableGuardSnapshotStoreError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::BundleSchemaVersionMismatch { expected, found } => write!(
                f,
                "durable guard bundle schema version mismatch, expected {expected}, found {found}"
            ),
            Self::Io(value) => write!(f, "durable guard snapshot store I/O error: {value}"),
            Self::InvalidPayload(value) => {
                write!(f, "durable guard snapshot store invalid payload: {value}")
            }
            Self::DeliverySnapshot(error) => write!(f, "{error}"),
            Self::ChannelPolicySnapshot(error) => write!(f, "{error}"),
        }
    }
}

impl std::error::Error for DurableGuardSnapshotStoreError {}

impl From<DeliveryGuardSnapshotError> for DurableGuardSnapshotStoreError {
    fn from(value: DeliveryGuardSnapshotError) -> Self {
        Self::DeliverySnapshot(value)
    }
}

impl From<ChannelPolicySnapshotError> for DurableGuardSnapshotStoreError {
    fn from(value: ChannelPolicySnapshotError) -> Self {
        Self::ChannelPolicySnapshot(value)
    }
}

/// Store interface for whole durable guard bundles.
pub trait DurableGuardBundleSnapshotStore {
    /// Saves a durable guard bundle.
    fn save_bundle(
        &mut self,
        bundle: DurableGuardSnapshotBundle,
    ) -> Result<(), DurableGuardSnapshotStoreError>;

    /// Loads a durable guard bundle, if present.
    fn load_bundle(&self) -> Result<Option<DurableGuardSnapshotBundle>, DurableGuardSnapshotStoreError>;
}

/// Store interface for delivery-guard snapshot lane.
pub trait DeliveryGuardSnapshotStore {
    /// Saves only delivery-guard snapshot payload.
    fn save_delivery_guard(
        &mut self,
        snapshot: DeliveryGuardSnapshot,
    ) -> Result<(), DurableGuardSnapshotStoreError>;

    /// Loads only delivery-guard snapshot payload, if present.
    fn load_delivery_guard(&self) -> Result<Option<DeliveryGuardSnapshot>, DurableGuardSnapshotStoreError>;
}

/// Store interface for channel-policy snapshot lane.
pub trait ChannelPolicySnapshotStore {
    /// Saves only channel-policy snapshot payload.
    fn save_channel_policy(
        &mut self,
        snapshot: ChannelPolicySnapshot,
    ) -> Result<(), DurableGuardSnapshotStoreError>;

    /// Loads only channel-policy snapshot payload, if present.
    fn load_channel_policy(&self) -> Result<Option<ChannelPolicySnapshot>, DurableGuardSnapshotStoreError>;
}

pub(crate) fn default_bundle() -> DurableGuardSnapshotBundle {
    DurableGuardSnapshotBundle::capture(
        &MessageDeliveryGuards::new(),
        &ChannelPermissionEngine::new(),
    )
}

pub(crate) fn validate_bundle(
    bundle: &DurableGuardSnapshotBundle,
) -> Result<(), DurableGuardSnapshotStoreError> {
    ensure_schema_version(bundle.schema_version)?;
    MessageDeliveryGuards::from_snapshot(bundle.delivery_guard.clone())
        .map_err(DurableGuardSnapshotStoreError::DeliverySnapshot)?;
    ChannelPermissionEngine::from_snapshot(bundle.channel_policy.clone())
        .map_err(DurableGuardSnapshotStoreError::ChannelPolicySnapshot)?;
    Ok(())
}

fn ensure_schema_version(found: u16) -> Result<(), DurableGuardSnapshotStoreError> {
    if found == DURABLE_GUARD_BUNDLE_SCHEMA_VERSION {
        return Ok(());
    }
    Err(DurableGuardSnapshotStoreError::BundleSchemaVersionMismatch {
        expected: DURABLE_GUARD_BUNDLE_SCHEMA_VERSION,
        found,
    })
}
