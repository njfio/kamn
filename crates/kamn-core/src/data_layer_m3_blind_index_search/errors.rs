use crate::data_layer_m3_blind_index_search::DataLayerM3BlindIndexSearchMode;
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DataLayerM3SearchError {
    EmptyField(&'static str),
    InvalidDid(String),
    DuplicateMessageId(String),
    InvalidBlindIndexToken {
        field_name: String,
    },
    UnsupportedBlindIndexSearchMode(DataLayerM3BlindIndexSearchMode),
    InvalidTimestampBounds {
        created_after_inclusive: u64,
        created_before_inclusive: u64,
    },
    InvalidLimit(usize),
    MissingContentCidForMessage {
        message_id: String,
    },
    InvalidRetrievalRequestProjection {
        reason: String,
    },
}

impl fmt::Display for DataLayerM3SearchError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyField(field) => write!(f, "{field} must not be empty"),
            Self::InvalidDid(value) => write!(f, "invalid did: {value}"),
            Self::DuplicateMessageId(message_id) => write!(f, "duplicate message_id: {message_id}"),
            Self::InvalidBlindIndexToken { field_name } => {
                write!(f, "invalid blind-index token for field: {field_name}")
            }
            Self::UnsupportedBlindIndexSearchMode(mode) => {
                write!(f, "unsupported blind-index search mode: {mode:?}")
            }
            Self::InvalidTimestampBounds {
                created_after_inclusive,
                created_before_inclusive,
            } => write!(
                f,
                "invalid timestamp bounds: after={created_after_inclusive}, before={created_before_inclusive}"
            ),
            Self::InvalidLimit(limit) => write!(f, "invalid limit: {limit}"),
            Self::MissingContentCidForMessage { message_id } => {
                write!(f, "missing content cid mapping for message_id: {message_id}")
            }
            Self::InvalidRetrievalRequestProjection { reason } => {
                write!(f, "invalid retrieval request projection: {reason}")
            }
        }
    }
}

impl std::error::Error for DataLayerM3SearchError {}
