//! M3 searchable-index contracts for blind-index and metadata lookups.
//!
//! This module models PRD M3 contracts as deterministic in-memory Rust APIs:
//! owner-scoped blind-index token derivation, exact-match blind-index lookups,
//! and metadata filter queries with stable ordering.

use crate::{ContentRetrievalError, ContentRetrievalRequest, ContentRetrievalScope};
use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

/// Hash algorithm label used by M3 blind-index tokens.
pub const DATA_LAYER_M3_HASH_ALGORITHM: &str = "sha256";

/// Normalization profile label used for blind-index value canonicalization.
pub const DATA_LAYER_M3_BLIND_INDEX_NORMALIZATION_PROFILE: &str =
    "ascii-lowercase-whitespace-collapse";
/// Blind-index determinism reason marker for baseline/observed match.
pub const DATA_LAYER_M3_BLIND_INDEX_DETERMINISM_STABLE_REASON_CODE: &str =
    "m3_blind_index_determinism_stable";
/// Blind-index determinism reason marker for baseline/observed drift.
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

/// Projection input envelope that bridges blind-index search output to retrieval contracts.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataLayerM3BlindIndexRetrievalProjectionInput {
    /// Blind-index query to execute before retrieval projection.
    pub blind_index_query: DataLayerM3BlindIndexQuery,
    /// Requester DID that will be bound to retrieval requests.
    pub requester_did: String,
    /// Retrieval scope to apply for all projected retrieval requests.
    pub retrieval_scope: ContentRetrievalScope,
    /// Request timestamp bound to all projected retrieval requests.
    pub requested_at_unix: u64,
    /// Message-ID to CID map used to bridge search results to retrieval IDs.
    pub message_cids_by_message_id: BTreeMap<String, String>,
}

/// One projected retrieval contract record derived from blind-index search output.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataLayerM3RetrievalProjectionRecord {
    /// Message ID from blind-index search output.
    pub message_id: String,
    /// CID mapped for this message.
    pub cid: String,
    /// Validated retrieval request contract.
    pub retrieval_request: ContentRetrievalRequest,
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
    /// Baseline ordered message IDs expected from deterministic search output.
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
    /// Stable reason marker.
    pub reason_code: &'static str,
    /// Baseline ordered message IDs.
    pub expected_message_ids: Vec<String>,
    /// Observed ordered message IDs from live query.
    pub observed_message_ids: Vec<String>,
    /// Baseline IDs missing from observed output.
    pub missing_message_ids: Vec<String>,
    /// Observed IDs not present in baseline.
    pub unexpected_message_ids: Vec<String>,
    /// IDs present in both sets but in different rank positions.
    pub out_of_order_message_ids: Vec<String>,
}

/// M3 search catalog for owner-scoped blind-index and metadata queries.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct DataLayerM3SearchCatalog {
    records: Vec<DataLayerM3MessageMetadataRecord>,
    seen_message_ids: BTreeSet<String>,
}

impl DataLayerM3SearchCatalog {
    /// Creates an empty search catalog.
    pub fn new() -> Self {
        Self::default()
    }

    /// Registers one metadata record.
    pub fn register_record(
        &mut self,
        mut record: DataLayerM3MessageMetadataRecord,
    ) -> Result<(), DataLayerM3SearchError> {
        validate_non_empty(record.message_id.as_str(), "message_id")?;
        validate_kamn_did(record.owner_did.as_str())?;
        validate_kamn_did(record.sender_did.as_str())?;
        validate_kamn_did(record.recipient_did.as_str())?;
        validate_non_empty(record.message_type.as_str(), "message_type")?;
        if record.created_at_epoch_seconds == 0 {
            return Err(DataLayerM3SearchError::EmptyField(
                "created_at_epoch_seconds",
            ));
        }
        if let Some(session_id) = record.session_id.as_deref() {
            validate_non_empty(session_id, "session_id")?;
        }
        if let Some(escrow_id) = record.escrow_id.as_deref() {
            validate_non_empty(escrow_id, "escrow_id")?;
        }
        if self.seen_message_ids.contains(record.message_id.as_str()) {
            return Err(DataLayerM3SearchError::DuplicateMessageId(
                record.message_id,
            ));
        }

        let mut canonical_blind_indexes = BTreeMap::new();
        for (field_name, token) in &record.blind_indexes {
            let canonical_field_name = canonical_field_name(field_name.as_str())?;
            validate_blind_index_token(canonical_field_name.as_str(), token.as_str())?;
            canonical_blind_indexes.insert(canonical_field_name, token.trim().to_owned());
        }
        record.blind_indexes = canonical_blind_indexes;

        self.seen_message_ids.insert(record.message_id.clone());
        self.records.push(record);
        Ok(())
    }

    /// Returns immutable append-order records.
    pub fn records(&self) -> &[DataLayerM3MessageMetadataRecord] {
        &self.records
    }

    /// Executes one owner-scoped blind-index query.
    pub fn search_blind_index(
        &self,
        query: DataLayerM3BlindIndexQuery,
    ) -> Result<Vec<DataLayerM3MessageMetadataRecord>, DataLayerM3SearchError> {
        validate_kamn_did(query.owner_did.as_str())?;
        let field_name = canonical_field_name(query.field_name.as_str())?;
        validate_blind_index_token(field_name.as_str(), query.token.as_str())?;
        let limit = resolve_limit(query.limit)?;

        if query.mode != DataLayerM3BlindIndexSearchMode::ExactMatch {
            return Err(DataLayerM3SearchError::UnsupportedBlindIndexSearchMode(
                query.mode,
            ));
        }

        let mut results = self
            .records
            .iter()
            .filter(|record| record.owner_did == query.owner_did)
            .filter(|record| {
                record
                    .blind_indexes
                    .get(field_name.as_str())
                    .is_some_and(|token| token == query.token.trim())
            })
            .cloned()
            .collect::<Vec<_>>();
        sort_results_deterministically(&mut results);
        if results.len() > limit {
            results.truncate(limit);
        }
        Ok(results)
    }

    /// Executes blind-index search and projects results to content-retrieval contracts.
    pub fn project_blind_index_to_retrieval_requests(
        &self,
        input: DataLayerM3BlindIndexRetrievalProjectionInput,
    ) -> Result<Vec<DataLayerM3RetrievalProjectionRecord>, DataLayerM3SearchError> {
        let DataLayerM3BlindIndexRetrievalProjectionInput {
            blind_index_query,
            requester_did,
            retrieval_scope,
            requested_at_unix,
            message_cids_by_message_id,
        } = input;
        let search_results = self.search_blind_index(blind_index_query)?;

        let mut projection = Vec::with_capacity(search_results.len());
        for record in search_results {
            let cid = message_cids_by_message_id
                .get(record.message_id.as_str())
                .cloned()
                .ok_or_else(|| DataLayerM3SearchError::MissingContentCidForMessage {
                    message_id: record.message_id.clone(),
                })?;
            let retrieval_request = ContentRetrievalRequest::new(
                cid.as_str(),
                requester_did.as_str(),
                retrieval_scope.clone(),
                requested_at_unix,
            )
            .map_err(map_content_retrieval_error_to_m3_projection_error)?;
            projection.push(DataLayerM3RetrievalProjectionRecord {
                message_id: record.message_id,
                cid,
                retrieval_request,
            });
        }

        Ok(projection)
    }

    /// Evaluates blind-index exact-match output determinism against a baseline ordering.
    pub fn evaluate_blind_index_determinism(
        &self,
        input: DataLayerM3BlindIndexDeterminismInput,
    ) -> Result<DataLayerM3BlindIndexDeterminismReport, DataLayerM3SearchError> {
        validate_kamn_did(input.owner_did.as_str())?;
        let canonical_field_name = canonical_field_name(input.field_name.as_str())?;
        validate_blind_index_token(canonical_field_name.as_str(), input.token.as_str())?;
        let limit = resolve_limit(input.limit)?;
        if input.baseline_ordered_message_ids.is_empty() {
            return Err(DataLayerM3SearchError::EmptyField(
                "baseline_ordered_message_ids",
            ));
        }

        let mut baseline_seen = BTreeSet::new();
        for message_id in &input.baseline_ordered_message_ids {
            validate_non_empty(message_id.as_str(), "baseline_ordered_message_id")?;
            if !baseline_seen.insert(message_id.clone()) {
                return Err(DataLayerM3SearchError::DuplicateMessageId(
                    message_id.clone(),
                ));
            }
        }

        let expected_message_ids = input.baseline_ordered_message_ids.clone();
        let observed_message_ids = self
            .search_blind_index(DataLayerM3BlindIndexQuery {
                owner_did: input.owner_did,
                field_name: canonical_field_name,
                token: input.token,
                mode: DataLayerM3BlindIndexSearchMode::ExactMatch,
                limit: Some(limit),
            })?
            .into_iter()
            .map(|record| record.message_id)
            .collect::<Vec<_>>();
        let observed_ids_set = observed_message_ids
            .iter()
            .cloned()
            .collect::<BTreeSet<_>>();
        let expected_ids_set = expected_message_ids
            .iter()
            .cloned()
            .collect::<BTreeSet<_>>();
        let missing_message_ids = expected_message_ids
            .iter()
            .filter(|message_id| !observed_ids_set.contains(*message_id))
            .cloned()
            .collect::<Vec<_>>();
        let unexpected_message_ids = observed_message_ids
            .iter()
            .filter(|message_id| !expected_ids_set.contains(*message_id))
            .cloned()
            .collect::<Vec<_>>();
        let observed_rank = observed_message_ids
            .iter()
            .enumerate()
            .map(|(rank, message_id)| (message_id.clone(), rank))
            .collect::<BTreeMap<_, _>>();
        let out_of_order_message_ids = expected_message_ids
            .iter()
            .enumerate()
            .filter_map(|(rank, message_id)| {
                observed_rank
                    .get(message_id)
                    .filter(|observed_rank| **observed_rank != rank)
                    .map(|_| message_id.clone())
            })
            .collect::<Vec<_>>();

        let drifted = !missing_message_ids.is_empty()
            || !unexpected_message_ids.is_empty()
            || !out_of_order_message_ids.is_empty();
        let (decision, reason_code) = if drifted {
            (
                DataLayerM3BlindIndexDeterminismDecision::Drifted,
                DATA_LAYER_M3_BLIND_INDEX_DETERMINISM_DRIFTED_REASON_CODE,
            )
        } else {
            (
                DataLayerM3BlindIndexDeterminismDecision::Stable,
                DATA_LAYER_M3_BLIND_INDEX_DETERMINISM_STABLE_REASON_CODE,
            )
        };

        Ok(DataLayerM3BlindIndexDeterminismReport {
            decision,
            reason_code,
            expected_message_ids,
            observed_message_ids,
            missing_message_ids,
            unexpected_message_ids,
            out_of_order_message_ids,
        })
    }

    /// Executes one owner-scoped metadata query.
    pub fn search_metadata(
        &self,
        query: DataLayerM3MetadataQuery,
    ) -> Result<Vec<DataLayerM3MessageMetadataRecord>, DataLayerM3SearchError> {
        validate_kamn_did(query.owner_did.as_str())?;
        if let Some(sender_did) = query.sender_did.as_deref() {
            validate_kamn_did(sender_did)?;
        }
        if let Some(recipient_did) = query.recipient_did.as_deref() {
            validate_kamn_did(recipient_did)?;
        }
        if let Some(session_id) = query.session_id.as_deref() {
            validate_non_empty(session_id, "session_id")?;
        }
        if let Some(escrow_id) = query.escrow_id.as_deref() {
            validate_non_empty(escrow_id, "escrow_id")?;
        }
        if let Some(message_type) = query.message_type.as_deref() {
            validate_non_empty(message_type, "message_type")?;
        }
        if let (Some(created_after_inclusive), Some(created_before_inclusive)) = (
            query.created_after_inclusive,
            query.created_before_inclusive,
        ) {
            if created_after_inclusive > created_before_inclusive {
                return Err(DataLayerM3SearchError::InvalidTimestampBounds {
                    created_after_inclusive,
                    created_before_inclusive,
                });
            }
        }
        let limit = resolve_limit(query.limit)?;

        let mut results = self
            .records
            .iter()
            .filter(|record| record.owner_did == query.owner_did)
            .filter(|record| {
                query
                    .sender_did
                    .as_ref()
                    .is_none_or(|sender_did| record.sender_did == *sender_did)
            })
            .filter(|record| {
                query
                    .recipient_did
                    .as_ref()
                    .is_none_or(|recipient_did| record.recipient_did == *recipient_did)
            })
            .filter(|record| {
                query
                    .session_id
                    .as_ref()
                    .is_none_or(|session_id| record.session_id.as_ref() == Some(session_id))
            })
            .filter(|record| {
                query
                    .escrow_id
                    .as_ref()
                    .is_none_or(|escrow_id| record.escrow_id.as_ref() == Some(escrow_id))
            })
            .filter(|record| {
                query
                    .message_type
                    .as_ref()
                    .is_none_or(|message_type| record.message_type == *message_type)
            })
            .filter(|record| {
                query
                    .created_after_inclusive
                    .is_none_or(|lower| record.created_at_epoch_seconds >= lower)
            })
            .filter(|record| {
                query
                    .created_before_inclusive
                    .is_none_or(|upper| record.created_at_epoch_seconds <= upper)
            })
            .cloned()
            .collect::<Vec<_>>();
        sort_results_deterministically(&mut results);
        if results.len() > limit {
            results.truncate(limit);
        }
        Ok(results)
    }
}

/// Derives one deterministic owner-scoped blind-index token.
pub fn data_layer_m3_compute_blind_index(
    blind_index_key_material: &str,
    field_name: &str,
    value: &str,
) -> Result<String, DataLayerM3SearchError> {
    validate_non_empty(blind_index_key_material, "blind_index_key_material")?;
    let normalized_field_name = canonical_field_name(field_name)?;
    let normalized_value = normalize_blind_index_value(value)?;
    Ok(tagged_digest(
        format!(
            "m3-blind-index|key:{}|field:{}|value:{}|profile:{}",
            blind_index_key_material.trim(),
            normalized_field_name,
            normalized_value,
            DATA_LAYER_M3_BLIND_INDEX_NORMALIZATION_PROFILE
        )
        .as_str(),
    ))
}

/// Normalizes one value for M3 blind-index derivation.
pub fn data_layer_m3_normalize_blind_index_value(
    value: &str,
) -> Result<String, DataLayerM3SearchError> {
    normalize_blind_index_value(value)
}

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
            Self::DuplicateMessageId(message_id) => {
                write!(f, "duplicate message_id: {message_id}")
            }
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

fn validate_kamn_did(value: &str) -> Result<(), DataLayerM3SearchError> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Err(DataLayerM3SearchError::InvalidDid(value.to_owned()));
    }
    if !trimmed.starts_with("kamn:did:") {
        return Err(DataLayerM3SearchError::InvalidDid(value.to_owned()));
    }
    let segments = trimmed.split(':').collect::<Vec<_>>();
    if segments.len() < 4 || segments.iter().any(|segment| segment.is_empty()) {
        return Err(DataLayerM3SearchError::InvalidDid(value.to_owned()));
    }
    Ok(())
}

fn validate_non_empty(value: &str, field_name: &'static str) -> Result<(), DataLayerM3SearchError> {
    if value.trim().is_empty() {
        return Err(DataLayerM3SearchError::EmptyField(field_name));
    }
    Ok(())
}

fn canonical_field_name(field_name: &str) -> Result<String, DataLayerM3SearchError> {
    let trimmed = field_name.trim();
    if trimmed.is_empty() {
        return Err(DataLayerM3SearchError::EmptyField("field_name"));
    }
    let canonical = trimmed.to_ascii_lowercase();
    if canonical
        .chars()
        .any(|character| !(character.is_ascii_alphanumeric() || character == '_'))
    {
        return Err(DataLayerM3SearchError::EmptyField("field_name"));
    }
    Ok(canonical)
}

fn validate_blind_index_token(field_name: &str, token: &str) -> Result<(), DataLayerM3SearchError> {
    let trimmed = token.trim();
    if trimmed.is_empty() || !trimmed.starts_with("sha256:") {
        return Err(DataLayerM3SearchError::InvalidBlindIndexToken {
            field_name: field_name.to_owned(),
        });
    }
    Ok(())
}

fn resolve_limit(limit: Option<usize>) -> Result<usize, DataLayerM3SearchError> {
    match limit {
        Some(0) => Err(DataLayerM3SearchError::InvalidLimit(0)),
        Some(value) => Ok(value),
        None => Ok(usize::MAX),
    }
}

fn map_content_retrieval_error_to_m3_projection_error(
    error: ContentRetrievalError,
) -> DataLayerM3SearchError {
    DataLayerM3SearchError::InvalidRetrievalRequestProjection {
        reason: error.to_string(),
    }
}

fn normalize_blind_index_value(value: &str) -> Result<String, DataLayerM3SearchError> {
    let normalized = value
        .split_whitespace()
        .map(str::to_ascii_lowercase)
        .collect::<Vec<_>>()
        .join(" ");
    if normalized.is_empty() {
        return Err(DataLayerM3SearchError::EmptyField("value"));
    }
    Ok(normalized)
}

fn sort_results_deterministically(results: &mut [DataLayerM3MessageMetadataRecord]) {
    results.sort_by(|left, right| {
        right
            .created_at_epoch_seconds
            .cmp(&left.created_at_epoch_seconds)
            .then(left.message_id.cmp(&right.message_id))
    });
}

fn tagged_digest(value: &str) -> String {
    format!(
        "{DATA_LAYER_M3_HASH_ALGORITHM}:{}",
        deterministic_digest_256_hex(value)
    )
}

fn deterministic_digest_256_hex(value: &str) -> String {
    const SEEDS: [u64; 4] = [
        0x243f6a8885a308d3,
        0x13198a2e03707344,
        0xa4093822299f31d0,
        0x082efa98ec4e6c89,
    ];
    let mut output = String::with_capacity(64);
    for (index, seed) in SEEDS.iter().enumerate() {
        let mut acc = *seed ^ (index as u64).wrapping_mul(0x9e3779b97f4a7c15);
        for (offset, byte) in value.as_bytes().iter().enumerate() {
            let mix = ((*byte as u64) << ((offset % 8) * 8))
                ^ ((offset as u64).wrapping_mul(0x100000001b3));
            acc ^= mix;
            acc = acc.rotate_left(((offset + index) % 63 + 1) as u32);
            acc = acc.wrapping_mul(0x100000001b3);
            acc ^= acc >> 29;
            acc = acc.wrapping_add(0x9e3779b97f4a7c15 ^ (index as u64));
        }
        output.push_str(format!("{acc:016x}").as_str());
    }
    output
}
