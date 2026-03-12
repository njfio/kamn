use std::fmt;

use crate::{ContentLifecycleManager, ContentRetentionClass};

/// Ephemeral retention window (24 hours).
pub const DATA_LAYER_M8_EPHEMERAL_RETENTION_SECONDS: u64 = 86_400;
/// Standard retention window (90 days).
pub const DATA_LAYER_M8_STANDARD_RETENTION_SECONDS: u64 = 7_776_000;
/// Extended retention window (365 days).
pub const DATA_LAYER_M8_EXTENDED_RETENTION_SECONDS: u64 = 31_536_000;
/// Stable wrapped-CEK tombstone marker for crypto-shredded messages.
pub const DATA_LAYER_M8_CEK_TOMBSTONE_MARKER: &str = "m8:cek:crypto-shredded";
/// Stable reason marker for owner-scope authorization failures.
pub const DATA_LAYER_M8_OWNER_SCOPE_DENIED_REASON_CODE: &str = "m8_compliance_owner_scope_denied";
/// Stable reason marker for retention-due projections.
pub const DATA_LAYER_M8_RETENTION_DUE_REASON_CODE: &str = "m8_compliance_retention_due";
/// Stable reason marker for successful crypto-shredding transitions.
pub const DATA_LAYER_M8_CRYPTO_SHRED_REASON_CODE: &str = "m8_compliance_crypto_shred_applied";

/// Retention class contract for M8 compliance lifecycle controls.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum DataLayerM8RetentionClass {
    /// Default profile: 90 days.
    Standard,
    /// Compliance-sensitive profile: 1 year.
    Extended,
    /// Hold profile: shredding blocked until explicit release.
    LegalHold,
    /// Never shred automatically.
    Permanent,
    /// Short-lived profile: 24 hours.
    Ephemeral,
}

impl DataLayerM8RetentionClass {
    pub(crate) fn retention_window_seconds(self) -> Option<u64> {
        data_layer_m8_retention_window_seconds(self)
    }
}

/// Errors returned when converting M8 retention classes to legacy lifecycle classes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DataLayerM8RetentionInteropError {
    /// Legacy lifecycle has no equivalent class for the M8 policy.
    LegacyRetentionClassUnavailable(DataLayerM8RetentionClass),
    /// Legacy lifecycle class does not have an equivalent M8 class without retention-window drift.
    M8RetentionClassUnavailable(ContentRetentionClass),
}

impl fmt::Display for DataLayerM8RetentionInteropError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::LegacyRetentionClassUnavailable(class) => {
                write!(
                    f,
                    "no legacy content lifecycle retention class for M8 class: {class:?}"
                )
            }
            Self::M8RetentionClassUnavailable(class) => {
                write!(
                    f,
                    "no equivalent M8 retention class for legacy class: {class:?}"
                )
            }
        }
    }
}

impl std::error::Error for DataLayerM8RetentionInteropError {}

impl TryFrom<DataLayerM8RetentionClass> for ContentRetentionClass {
    type Error = DataLayerM8RetentionInteropError;

    fn try_from(value: DataLayerM8RetentionClass) -> Result<Self, Self::Error> {
        match value {
            DataLayerM8RetentionClass::Extended => Ok(ContentRetentionClass::Compliance),
            _ => Err(DataLayerM8RetentionInteropError::LegacyRetentionClassUnavailable(value)),
        }
    }
}

impl TryFrom<ContentRetentionClass> for DataLayerM8RetentionClass {
    type Error = DataLayerM8RetentionInteropError;

    fn try_from(value: ContentRetentionClass) -> Result<Self, Self::Error> {
        match value {
            ContentRetentionClass::Compliance => Ok(DataLayerM8RetentionClass::Extended),
            _ => Err(DataLayerM8RetentionInteropError::M8RetentionClassUnavailable(value)),
        }
    }
}

/// Returns the effective M8 retention window in seconds for classes with finite TTL.
pub fn data_layer_m8_retention_window_seconds(class: DataLayerM8RetentionClass) -> Option<u64> {
    match class {
        DataLayerM8RetentionClass::Standard => Some(DATA_LAYER_M8_STANDARD_RETENTION_SECONDS),
        DataLayerM8RetentionClass::Extended => Some(DATA_LAYER_M8_EXTENDED_RETENTION_SECONDS),
        DataLayerM8RetentionClass::LegalHold => None,
        DataLayerM8RetentionClass::Permanent => None,
        DataLayerM8RetentionClass::Ephemeral => Some(DATA_LAYER_M8_EPHEMERAL_RETENTION_SECONDS),
    }
}

/// Returns whether an M8 class has a retention-window equivalent in legacy `content_lifecycle`.
pub fn data_layer_m8_retention_window_aligned_with_content_lifecycle(
    class: DataLayerM8RetentionClass,
) -> Option<bool> {
    let (legacy_class, m8_window_seconds) = match class {
        DataLayerM8RetentionClass::Ephemeral => (
            ContentRetentionClass::ShortLived,
            DATA_LAYER_M8_EPHEMERAL_RETENTION_SECONDS,
        ),
        DataLayerM8RetentionClass::Standard => (
            ContentRetentionClass::Standard,
            DATA_LAYER_M8_STANDARD_RETENTION_SECONDS,
        ),
        DataLayerM8RetentionClass::Extended => (
            ContentRetentionClass::Compliance,
            DATA_LAYER_M8_EXTENDED_RETENTION_SECONDS,
        ),
        DataLayerM8RetentionClass::LegalHold | DataLayerM8RetentionClass::Permanent => return None,
    };

    let legacy_window_seconds =
        ContentLifecycleManager::retention_profile(legacy_class).retain_for_secs;
    Some(m8_window_seconds == legacy_window_seconds)
}
