use crate::data_layer_m3_blind_index_search::DataLayerM3BlindIndexSearchMode;
use std::fmt;

/// Error taxonomy for M3 blind-index and metadata search contracts.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DataLayerM3SearchError {
    /// Required field was empty.
    EmptyField(&'static str),
    /// DID failed M3 validation.
    InvalidDid(String),
    /// Duplicate message identifier registration was attempted.
    DuplicateMessageId(String),
    /// Blind-index token is malformed for one field.
    InvalidBlindIndexToken {
        /// Field name associated with the invalid token.
        field_name: String,
    },
    /// Blind-index mode is unsupported by M3.
    UnsupportedBlindIndexSearchMode(DataLayerM3BlindIndexSearchMode),
    /// Created-at bounds are invalid.
    InvalidTimestampBounds {
        /// Inclusive lower bound.
        created_after_inclusive: u64,
        /// Inclusive upper bound.
        created_before_inclusive: u64,
    },
    /// Limit must be positive.
    InvalidLimit(usize),
    /// Blind-index output message lacked a CID bridge mapping.
    MissingContentCidForMessage {
        /// Message ID missing from CID bridge map.
        message_id: String,
    },
    /// Retrieval request projection failed validation.
    InvalidRetrievalRequestProjection {
        /// Deterministic validation reason.
        reason: String,
    },
}

impl fmt::Display for DataLayerM3SearchError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyField(field) => write!(f, "{field} must not be empty"),
            Self::InvalidDid(value) => write!(f, "invalid did: {value}"),
            Self::DuplicateMessageId(message_id) => write!(f, "duplicate message_id: {message_id}"),
            Self::InvalidBlindIndexToken { field_name } => write_invalid_token(f, field_name),
            Self::UnsupportedBlindIndexSearchMode(mode) => {
                write!(f, "unsupported blind-index search mode: {mode:?}")
            }
            Self::InvalidTimestampBounds {
                created_after_inclusive,
                created_before_inclusive,
            } => write_invalid_timestamp_bounds(
                f,
                *created_after_inclusive,
                *created_before_inclusive,
            ),
            Self::InvalidLimit(limit) => write!(f, "invalid limit: {limit}"),
            Self::MissingContentCidForMessage { message_id } => write_missing_cid(f, message_id),
            Self::InvalidRetrievalRequestProjection { reason } => {
                write!(f, "invalid retrieval request projection: {reason}")
            }
        }
    }
}

impl std::error::Error for DataLayerM3SearchError {}

fn write_invalid_token(f: &mut fmt::Formatter<'_>, field_name: &str) -> fmt::Result {
    write!(f, "invalid blind-index token for field: {field_name}")
}

fn write_invalid_timestamp_bounds(
    f: &mut fmt::Formatter<'_>,
    created_after_inclusive: u64,
    created_before_inclusive: u64,
) -> fmt::Result {
    write!(
        f,
        "invalid timestamp bounds: after={created_after_inclusive}, before={created_before_inclusive}"
    )
}

fn write_missing_cid(f: &mut fmt::Formatter<'_>, message_id: &str) -> fmt::Result {
    write!(
        f,
        "missing content cid mapping for message_id: {message_id}"
    )
}
