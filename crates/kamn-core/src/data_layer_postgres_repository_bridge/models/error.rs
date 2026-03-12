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
        formatter.write_str(self.message().as_str())
    }
}

impl std::error::Error for DataLayerPgRepositoryBridgeError {}

impl DataLayerPgRepositoryBridgeError {
    fn message(&self) -> String {
        if let Some(message) = self.invalid_input_message() {
            return message;
        }
        self.capability_message()
    }

    fn invalid_input_message(&self) -> Option<String> {
        match self {
            Self::EmptyField(field) => Some(format!("{field} must not be empty")),
            Self::InvalidRequesterDid { field, reason_code, detail } => {
                Some(invalid_did_message("requester", field, reason_code, detail))
            }
            Self::InvalidOwnerDid { field, reason_code, detail } => {
                Some(invalid_did_message("owner", field, reason_code, detail))
            }
            Self::InvalidSearchLimit { requested, max_allowed } => {
                Some(invalid_limit_message(*requested, *max_allowed))
            }
            _ => None,
        }
    }

    fn capability_message(&self) -> String {
        match self {
            Self::PgvectorExtensionUnavailable { .. }
            | Self::PgvectorDimensionMismatch { .. } => self.pgvector_message(),
            Self::AgeExtensionUnavailable { .. } | Self::AgeUnsupportedRelation { .. } => {
                self.age_message()
            }
            Self::TimescaleExtensionUnavailable { .. }
            | Self::InvalidTimescaleBucketWindow { .. } => self.timescale_message(),
            Self::EmptyField(_)
            | Self::InvalidRequesterDid { .. }
            | Self::InvalidOwnerDid { .. }
            | Self::InvalidSearchLimit { .. } => unreachable!("handled by invalid_input_message"),
        }
    }

    fn pgvector_message(&self) -> String {
        match self {
            Self::PgvectorExtensionUnavailable { reason_code } => {
                extension_unavailable_message("pgvector", reason_code)
            }
            Self::PgvectorDimensionMismatch {
                reason_code,
                expected,
                found,
            } => vector_mismatch_message(reason_code, *expected, *found),
            _ => unreachable!("handled by capability_message"),
        }
    }

    fn age_message(&self) -> String {
        match self {
            Self::AgeExtensionUnavailable { reason_code } => {
                extension_unavailable_message("age", reason_code)
            }
            Self::AgeUnsupportedRelation {
                reason_code,
                relation_marker,
            } => unsupported_relation_message(reason_code, relation_marker),
            _ => unreachable!("handled by capability_message"),
        }
    }

    fn timescale_message(&self) -> String {
        match self {
            Self::TimescaleExtensionUnavailable { reason_code } => {
                extension_unavailable_message("timescale", reason_code)
            }
            Self::InvalidTimescaleBucketWindow {
                reason_code,
                bucket_window_seconds,
            } => invalid_bucket_window_message(reason_code, *bucket_window_seconds),
            _ => unreachable!("handled by capability_message"),
        }
    }
}

fn invalid_did_message(
    label: &str,
    field: &str,
    reason_code: &str,
    detail: &str,
) -> String {
    format!("invalid {label} did field {field}: {reason_code} ({detail})")
}

fn invalid_limit_message(requested: u32, max_allowed: u32) -> String {
    format!("invalid blind-index search limit: requested {requested}, max {max_allowed}")
}

fn extension_unavailable_message(extension: &str, reason_code: &str) -> String {
    format!("{extension} extension unavailable: {reason_code}")
}

fn vector_mismatch_message(reason_code: &str, expected: usize, found: usize) -> String {
    format!("pgvector dimension mismatch: {reason_code} (expected {expected}, found {found})")
}

fn unsupported_relation_message(reason_code: &str, relation_marker: &str) -> String {
    format!("age relation unsupported: {reason_code} ({relation_marker})")
}

fn invalid_bucket_window_message(reason_code: &str, bucket_window_seconds: u64) -> String {
    format!("timescale bucket window invalid: {reason_code} ({bucket_window_seconds})")
}
