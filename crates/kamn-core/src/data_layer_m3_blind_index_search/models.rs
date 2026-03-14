use std::collections::{BTreeMap, BTreeSet};

pub const DATA_LAYER_M3_HASH_ALGORITHM: &str = "sha256";
pub const DATA_LAYER_M3_BLIND_INDEX_NORMALIZATION_PROFILE: &str =
    "ascii-lowercase-whitespace-collapse";
pub const DATA_LAYER_M3_BLIND_INDEX_DETERMINISM_STABLE_REASON_CODE: &str =
    "m3_blind_index_determinism_stable";
pub const DATA_LAYER_M3_BLIND_INDEX_DETERMINISM_DRIFTED_REASON_CODE: &str =
    "m3_blind_index_determinism_drifted";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataLayerM3MessageMetadataRecord {
    pub message_id: String,
    pub owner_did: String,
    pub sender_did: String,
    pub recipient_did: String,
    pub session_id: Option<String>,
    pub escrow_id: Option<String>,
    pub message_type: String,
    pub created_at_epoch_seconds: u64,
    pub blind_indexes: BTreeMap<String, String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DataLayerM3BlindIndexSearchMode {
    ExactMatch,
    Contains,
    Range,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataLayerM3BlindIndexQuery {
    pub owner_did: String,
    pub field_name: String,
    pub token: String,
    pub mode: DataLayerM3BlindIndexSearchMode,
    pub limit: Option<usize>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataLayerM3BlindIndexRetrievalProjectionInput {
    pub blind_index_query: DataLayerM3BlindIndexQuery,
    pub requester_did: String,
    pub retrieval_scope: crate::ContentRetrievalScope,
    pub requested_at_unix: u64,
    pub message_cids_by_message_id: BTreeMap<String, String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataLayerM3RetrievalProjectionRecord {
    pub message_id: String,
    pub cid: String,
    pub retrieval_request: crate::ContentRetrievalRequest,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataLayerM3MetadataQuery {
    pub owner_did: String,
    pub sender_did: Option<String>,
    pub recipient_did: Option<String>,
    pub session_id: Option<String>,
    pub escrow_id: Option<String>,
    pub message_type: Option<String>,
    pub created_after_inclusive: Option<u64>,
    pub created_before_inclusive: Option<u64>,
    pub limit: Option<usize>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataLayerM3BlindIndexDeterminismInput {
    pub owner_did: String,
    pub field_name: String,
    pub token: String,
    pub baseline_ordered_message_ids: Vec<String>,
    pub limit: Option<usize>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DataLayerM3BlindIndexDeterminismDecision {
    Stable,
    Drifted,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataLayerM3BlindIndexDeterminismReport {
    pub decision: DataLayerM3BlindIndexDeterminismDecision,
    pub reason_code: &'static str,
    pub expected_message_ids: Vec<String>,
    pub observed_message_ids: Vec<String>,
    pub missing_message_ids: Vec<String>,
    pub unexpected_message_ids: Vec<String>,
    pub out_of_order_message_ids: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct DataLayerM3SearchCatalog {
    pub(crate) records: Vec<DataLayerM3MessageMetadataRecord>,
    pub(crate) seen_message_ids: BTreeSet<String>,
}
