use std::fmt;

use crate::{DataLayerPgOperationKind, DataLayerPgRepositoryBridgeError};

/// Error taxonomy for live PostgreSQL adapter behavior.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DataLayerPgExecutionAdapterError {
    /// Required field was empty.
    EmptyField(&'static str),
    /// Pool max-connections configuration is invalid.
    InvalidMaxConnections(u32),
    /// Database URL failed validation.
    InvalidDatabaseUrl {
        /// Field name carrying the URL.
        field: &'static str,
        /// Stable reason marker.
        reason_code: &'static str,
        /// Parser detail.
        detail: String,
    },
    /// Bridge projection failed before SQL execution.
    BridgeProjectionFailed {
        /// Operation kind being projected.
        operation: DataLayerPgOperationKind,
        /// Error detail from bridge layer.
        detail: String,
    },
    /// SQL execution failed.
    SqlExecutionFailed {
        /// Operation that failed.
        operation: DataLayerPgOperationKind,
        /// Stable reason marker.
        reason_code: &'static str,
        /// SQL error detail.
        detail: String,
    },
    /// Migration discovery or migration-IO failed.
    MigrationIoFailed {
        /// Stable reason marker.
        reason_code: &'static str,
        /// IO error detail.
        detail: String,
    },
    /// Migration application failed.
    MigrationFailed {
        /// Stable reason marker.
        reason_code: &'static str,
        /// Migration error detail.
        detail: String,
    },
    /// Default RLS statement application failed.
    RlsStatementApplyFailed {
        /// Stable reason marker.
        reason_code: &'static str,
        /// SQL execution detail.
        detail: String,
    },
    /// Blind-index JSON payload failed fail-closed validation.
    InvalidBlindIndexesPayload {
        /// Field-name carrying invalid blind-index payload data.
        field: &'static str,
        /// Fail-closed detail.
        detail: String,
    },
    /// Merkle-batch payload failed fail-closed validation.
    InvalidMerkleBatchPayload {
        /// Field-name carrying invalid merkle-batch payload data.
        field: &'static str,
        /// Fail-closed detail.
        detail: String,
    },
    /// Row decoding failed.
    DecodeFailed {
        /// Field that failed to decode.
        field: &'static str,
        /// Decode error detail.
        detail: String,
    },
}

impl fmt::Display for DataLayerPgExecutionAdapterError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyField(field) => write!(formatter, "{field} must not be empty"),
            Self::InvalidMaxConnections(value) => {
                write!(formatter, "max_connections must be > 0 (got {value})")
            }
            Self::InvalidDatabaseUrl {
                field,
                reason_code,
                detail,
            } => write!(
                formatter,
                "invalid database url field {field}: {reason_code} ({detail})"
            ),
            Self::BridgeProjectionFailed { operation, detail } => {
                write!(
                    formatter,
                    "bridge projection failed for {operation:?}: {detail}"
                )
            }
            Self::SqlExecutionFailed {
                operation,
                reason_code,
                detail,
            } => write!(
                formatter,
                "sql execution failed for {operation:?}: {reason_code} ({detail})"
            ),
            Self::MigrationIoFailed {
                reason_code,
                detail,
            } => write!(formatter, "migration io failed: {reason_code} ({detail})"),
            Self::MigrationFailed {
                reason_code,
                detail,
            } => write!(formatter, "migration failed: {reason_code} ({detail})"),
            Self::RlsStatementApplyFailed {
                reason_code,
                detail,
            } => write!(
                formatter,
                "RLS statement application failed: {reason_code} ({detail})"
            ),
            Self::InvalidBlindIndexesPayload { field, detail } => {
                write!(
                    formatter,
                    "invalid blind-index payload for {field}: {detail}"
                )
            }
            Self::InvalidMerkleBatchPayload { field, detail } => {
                write!(
                    formatter,
                    "invalid merkle-batch payload for {field}: {detail}"
                )
            }
            Self::DecodeFailed { field, detail } => {
                write!(formatter, "decode failed for {field}: {detail}")
            }
        }
    }
}

impl std::error::Error for DataLayerPgExecutionAdapterError {}

impl From<DataLayerPgRepositoryBridgeError> for DataLayerPgExecutionAdapterError {
    fn from(error: DataLayerPgRepositoryBridgeError) -> Self {
        Self::BridgeProjectionFailed {
            operation: DataLayerPgOperationKind::SelectMessageById,
            detail: error.to_string(),
        }
    }
}
