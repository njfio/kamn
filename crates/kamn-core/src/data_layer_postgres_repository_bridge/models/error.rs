use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DataLayerPgRepositoryBridgeError {
    EmptyField(&'static str),
    InvalidRequesterDid {
        field: &'static str,
        reason_code: &'static str,
        detail: String,
    },
    InvalidOwnerDid {
        field: &'static str,
        reason_code: &'static str,
        detail: String,
    },
    InvalidSearchLimit {
        requested: u32,
        max_allowed: u32,
    },
    PgvectorExtensionUnavailable {
        reason_code: &'static str,
    },
    PgvectorDimensionMismatch {
        reason_code: &'static str,
        expected: usize,
        found: usize,
    },
    AgeExtensionUnavailable {
        reason_code: &'static str,
    },
    AgeUnsupportedRelation {
        reason_code: &'static str,
        relation_marker: &'static str,
    },
    TimescaleExtensionUnavailable {
        reason_code: &'static str,
    },
    InvalidTimescaleBucketWindow {
        reason_code: &'static str,
        bucket_window_seconds: u64,
    },
}

impl fmt::Display for DataLayerPgRepositoryBridgeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyField(field) => write!(formatter, "{field} must not be empty"),
            Self::InvalidRequesterDid {
                field,
                reason_code,
                detail,
            } => write!(
                formatter,
                "invalid requester did field {field}: {reason_code} ({detail})"
            ),
            Self::InvalidOwnerDid {
                field,
                reason_code,
                detail,
            } => write!(
                formatter,
                "invalid owner did field {field}: {reason_code} ({detail})"
            ),
            Self::InvalidSearchLimit {
                requested,
                max_allowed,
            } => write!(
                formatter,
                "invalid blind-index search limit: requested {requested}, max {max_allowed}"
            ),
            Self::PgvectorExtensionUnavailable { reason_code } => {
                write!(formatter, "pgvector extension unavailable: {reason_code}")
            }
            Self::PgvectorDimensionMismatch {
                reason_code,
                expected,
                found,
            } => write!(
                formatter,
                "pgvector dimension mismatch: {reason_code} (expected {expected}, found {found})"
            ),
            Self::AgeExtensionUnavailable { reason_code } => {
                write!(formatter, "age extension unavailable: {reason_code}")
            }
            Self::AgeUnsupportedRelation {
                reason_code,
                relation_marker,
            } => write!(
                formatter,
                "age relation unsupported: {reason_code} ({relation_marker})"
            ),
            Self::TimescaleExtensionUnavailable { reason_code } => {
                write!(formatter, "timescale extension unavailable: {reason_code}")
            }
            Self::InvalidTimescaleBucketWindow {
                reason_code,
                bucket_window_seconds,
            } => write!(
                formatter,
                "timescale bucket window invalid: {reason_code} ({bucket_window_seconds})"
            ),
        }
    }
}

impl std::error::Error for DataLayerPgRepositoryBridgeError {}
