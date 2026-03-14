use std::collections::{BTreeMap, BTreeSet};

/// Hash algorithm label used by M3 blind-index tokens.
pub const DATA_LAYER_M3_HASH_ALGORITHM: &str = "sha256";
/// Normalization profile label used for blind-index value canonicalization.
pub const DATA_LAYER_M3_BLIND_INDEX_NORMALIZATION_PROFILE: &str =
    "ascii-lowercase-whitespace-collapse";
/// Determinism reason marker for stable blind-index output.
pub const DATA_LAYER_M3_BLIND_INDEX_DETERMINISM_STABLE_REASON_CODE: &str =
    "m3_blind_index_determinism_stable";
/// Determinism reason marker for drifted blind-index output.
pub const DATA_LAYER_M3_BLIND_INDEX_DETERMINISM_DRIFTED_REASON_CODE: &str =
    "m3_blind_index_determinism_drifted";

/// One stored message metadata projection with optional blind-index tokens.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataLayerM3MessageMetadataRecord {
    /// Stable message identifier.
    pub message_id: String,
    /// Owner DID that scopes this record.
    pub owner_did: String,
    /// Sender DID.
    pub sender_did: String,
    /// Recipient DID.
    pub recipient_did: String,
    /// Optional session identifier.
    pub session_id: Option<String>,
    /// Optional escrow identifier.
    pub escrow_id: Option<String>,
    /// Message type marker.
    pub message_type: String,
    /// Message created timestamp in epoch seconds.
    pub created_at_epoch_seconds: u64,
    /// Field-name to blind-index-token map.
    pub blind_indexes: BTreeMap<String, String>,
}

/// Supported blind-index query mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DataLayerM3BlindIndexSearchMode {
    /// Exact-match lookup over one blind-index token.
    ExactMatch,
    /// Unsupported substring search mode.
    Contains,
    /// Unsupported range search mode.
    Range,
}

/// Blind-index query input envelope.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataLayerM3BlindIndexQuery {
    /// Owner DID scope for this query.
    pub owner_did: String,
    /// Blind-index field name.
    pub field_name: String,
    /// Blind-index token value.
    pub token: String,
    /// Search mode.
    pub mode: DataLayerM3BlindIndexSearchMode,
    /// Optional maximum number of rows to return.
    pub limit: Option<usize>,
}

/// Projection input that bridges blind-index search output to retrieval contracts.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataLayerM3BlindIndexRetrievalProjectionInput {
    /// Blind-index query to execute before retrieval projection.
    pub blind_index_query: DataLayerM3BlindIndexQuery,
    /// Requester DID bound to retrieval requests.
    pub requester_did: String,
    /// Retrieval scope applied to each projected request.
    pub retrieval_scope: crate::ContentRetrievalScope,
    /// Request timestamp bound to each projected request.
    pub requested_at_unix: u64,
    /// Message-ID to CID bridge used for retrieval projection.
    pub message_cids_by_message_id: BTreeMap<String, String>,
}

/// One projected retrieval contract record derived from blind-index output.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataLayerM3RetrievalProjectionRecord {
    /// Message identifier returned by search.
    pub message_id: String,
    /// CID mapped for this message.
    pub cid: String,
    /// Validated retrieval request contract.
    pub retrieval_request: crate::ContentRetrievalRequest,
}

/// Metadata query input envelope.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataLayerM3MetadataQuery {
    /// Owner DID scope for this query.
    pub owner_did: String,
    /// Optional sender DID filter.
    pub sender_did: Option<String>,
    /// Optional recipient DID filter.
    pub recipient_did: Option<String>,
    /// Optional session identifier filter.
    pub session_id: Option<String>,
    /// Optional escrow identifier filter.
    pub escrow_id: Option<String>,
    /// Optional message type filter.
    pub message_type: Option<String>,
    /// Inclusive lower timestamp bound.
    pub created_after_inclusive: Option<u64>,
    /// Inclusive upper timestamp bound.
    pub created_before_inclusive: Option<u64>,
    /// Optional maximum number of rows to return.
    pub limit: Option<usize>,
}

/// Input contract for blind-index search determinism evaluation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataLayerM3BlindIndexDeterminismInput {
    /// Owner DID scope for this query.
    pub owner_did: String,
    /// Blind-index field name.
    pub field_name: String,
    /// Blind-index token value.
    pub token: String,
    /// Baseline ordered message IDs expected from deterministic output.
    pub baseline_ordered_message_ids: Vec<String>,
    /// Optional maximum number of rows to evaluate.
    pub limit: Option<usize>,
}

/// Determinism decision for blind-index output comparison.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DataLayerM3BlindIndexDeterminismDecision {
    /// Baseline and observed outputs match.
    Stable,
    /// Baseline and observed outputs diverge.
    Drifted,
}

/// Determinism report for blind-index output comparison.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataLayerM3BlindIndexDeterminismReport {
    /// Determinism decision.
    pub decision: DataLayerM3BlindIndexDeterminismDecision,
    /// Stable or drifted reason marker.
    pub reason_code: &'static str,
    /// Baseline ordered message IDs.
    pub expected_message_ids: Vec<String>,
    /// Observed ordered message IDs from live query.
    pub observed_message_ids: Vec<String>,
    /// Baseline IDs missing from observed output.
    pub missing_message_ids: Vec<String>,
    /// Observed IDs not present in baseline.
    pub unexpected_message_ids: Vec<String>,
    /// IDs present in both sets but ranked differently.
    pub out_of_order_message_ids: Vec<String>,
}

/// M3 search catalog for owner-scoped blind-index and metadata queries.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct DataLayerM3SearchCatalog {
    pub(crate) records: Vec<DataLayerM3MessageMetadataRecord>,
    pub(crate) seen_message_ids: BTreeSet<String>,
}
