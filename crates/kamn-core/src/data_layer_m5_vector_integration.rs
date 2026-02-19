//! M5 vector-layer contracts for embedding storage, semantic query, recall drift, and anomaly scoring.
//!
//! This module models PRD M5 behavior as deterministic Rust contracts:
//! owner-scoped embedding registration with append-only hash chaining,
//! semantic top-k query ranking over cosine similarity, and
//! centroid-distance anomaly detection for agent behavior monitoring.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

use crate::{AgentDid, AgentDidError, ContentLifecycleManager, ContentRetentionClass};

/// Hash algorithm label used by M5 deterministic record digests.
pub const DATA_LAYER_M5_HASH_ALGORITHM: &str = "sha256";
/// Genesis marker used by owner-scoped embedding hash chains.
pub const DATA_LAYER_M5_EMBEDDING_HASH_CHAIN_GENESIS: &str = "GENESIS";
/// Distance metric label used by semantic and anomaly contracts.
pub const DATA_LAYER_M5_VECTOR_DISTANCE_METRIC_COSINE: &str = "cosine";
/// Owner-side privacy mode rejects plaintext vector storage.
pub const DATA_LAYER_M5_OWNER_SIDE_PLAINTEXT_STORAGE_NOT_ALLOWED_REASON_CODE: &str =
    "m5_vector_owner_side_plaintext_storage_not_allowed";
/// Server-side plaintext mode requires plaintext vector storage.
pub const DATA_LAYER_M5_SERVER_SIDE_PLAINTEXT_REQUIRED_REASON_CODE: &str =
    "m5_vector_server_side_plaintext_required";
/// Owner-side encrypted mode requires local semantic query execution.
pub const DATA_LAYER_M5_OWNER_SIDE_QUERY_REQUIRES_LOCAL_INDEX_REASON_CODE: &str =
    "m5_vector_owner_side_query_requires_local_index";
/// Semantic query requires a plaintext index for owner scope.
pub const DATA_LAYER_M5_PLAINTEXT_INDEX_MISSING_FOR_OWNER_SCOPE_REASON_CODE: &str =
    "m5_vector_plaintext_index_missing_for_owner_scope";
/// Owner-side encrypted mode requires local anomaly pipeline execution.
pub const DATA_LAYER_M5_OWNER_SIDE_ANOMALY_REQUIRES_LOCAL_PIPELINE_REASON_CODE: &str =
    "m5_vector_owner_side_anomaly_requires_local_pipeline";
/// Candidate exceeded anomaly threshold.
pub const DATA_LAYER_M5_ANOMALY_THRESHOLD_EXCEEDED_REASON_CODE: &str =
    "m5_vector_anomaly_threshold_exceeded";
/// Candidate remained within anomaly threshold.
pub const DATA_LAYER_M5_ANOMALY_WITHIN_THRESHOLD_REASON_CODE: &str =
    "m5_vector_anomaly_within_threshold";
/// Recall drift remained within configured guardrails.
pub const DATA_LAYER_M5_RECALL_DRIFT_STABLE_REASON_CODE: &str = "m5_vector_recall_drift_stable";
/// Recall drift degraded beyond configured guardrails.
pub const DATA_LAYER_M5_RECALL_DRIFT_DEGRADED_REASON_CODE: &str = "m5_vector_recall_drift_degraded";
/// Agent DID failed canonical parser validation.
pub const DATA_LAYER_M5_INVALID_AGENT_DID_REASON_CODE: &str = "m5_vector_invalid_agent_did";
/// Retention due projection reason marker.
pub const DATA_LAYER_M5_RETENTION_DUE_REASON_CODE: &str = "m5_vector_retention_due";

/// Embedding storage/privacy mode contract.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DataLayerM5EmbeddingPrivacyMode {
    /// Embeddings are stored encrypted only; semantic and anomaly operations run owner-side.
    OwnerSideEncrypted,
    /// Plaintext vectors are opt-in for server-side semantic and anomaly operations.
    ServerSidePlaintextOptIn,
}

impl DataLayerM5EmbeddingPrivacyMode {
    fn marker(self) -> &'static str {
        match self {
            Self::OwnerSideEncrypted => "owner-side-encrypted",
            Self::ServerSidePlaintextOptIn => "server-side-plaintext-opt-in",
        }
    }
}

/// Input payload for registering one embedding record.
#[derive(Debug, Clone, PartialEq)]
pub struct DataLayerM5EmbeddingRecordInput {
    /// Stable embedding identifier.
    pub embedding_id: String,
    /// Source message identifier.
    pub message_id: String,
    /// Owner DID scope.
    pub owner_did: String,
    /// Agent DID associated with this embedding.
    pub agent_did: String,
    /// Retention class aligned with content lifecycle policy.
    pub retention_class: ContentRetentionClass,
    /// Embedding model identifier.
    pub model_id: String,
    /// Encrypted embedding payload bytes.
    pub vector_encrypted: Vec<u8>,
    /// Optional plaintext embedding vector (server-side opt-in mode only).
    pub vector_plaintext: Option<Vec<f32>>,
    /// Embedding creation timestamp in epoch seconds.
    pub created_at_epoch_seconds: u64,
}

/// Stored append-only embedding record.
#[derive(Debug, Clone, PartialEq)]
pub struct DataLayerM5EmbeddingRecord {
    /// Stable embedding identifier.
    pub embedding_id: String,
    /// Source message identifier.
    pub message_id: String,
    /// Owner DID scope.
    pub owner_did: String,
    /// Agent DID associated with this embedding.
    pub agent_did: String,
    /// Retention class aligned with content lifecycle policy.
    pub retention_class: ContentRetentionClass,
    /// Embedding model identifier.
    pub model_id: String,
    /// Encrypted embedding payload bytes.
    pub vector_encrypted: Vec<u8>,
    /// Privacy mode marker active at ingestion time.
    pub privacy_mode: DataLayerM5EmbeddingPrivacyMode,
    /// Optional plaintext vector projection.
    pub vector_plaintext: Option<Vec<f32>>,
    /// Vector dimensionality.
    pub vector_dimensions: usize,
    /// Zero-based append sequence (1-indexed in storage).
    pub sequence: u64,
    /// Embedding creation timestamp in epoch seconds.
    pub created_at_epoch_seconds: u64,
    /// Previous owner-scoped chain hash.
    pub hash_chain_prev: String,
    /// Deterministic record hash.
    pub record_hash: String,
}

/// Semantic query input contract.
#[derive(Debug, Clone, PartialEq)]
pub struct DataLayerM5SemanticQuery {
    /// Owner DID scope.
    pub owner_did: String,
    /// Query vector used for cosine-similarity ranking.
    pub query_vector: Vec<f32>,
    /// Optional maximum number of results to return.
    pub limit: Option<usize>,
}

/// One semantic query result row.
#[derive(Debug, Clone, PartialEq)]
pub struct DataLayerM5SemanticQueryResult {
    /// Matched embedding identifier.
    pub embedding_id: String,
    /// Matched message identifier.
    pub message_id: String,
    /// Cosine similarity score (higher is better).
    pub similarity_score: f32,
    /// Cosine distance (lower is better).
    pub cosine_distance: f32,
}

/// Anomaly evaluation input contract.
#[derive(Debug, Clone, PartialEq)]
pub struct DataLayerM5AnomalyEvaluationInput {
    /// Owner DID scope.
    pub owner_did: String,
    /// Agent DID whose centroid history is evaluated.
    pub agent_did: String,
    /// Candidate vector to compare against centroid.
    pub candidate_vector: Vec<f32>,
    /// Optional number of recent vectors to include in centroid.
    pub lookback_window: Option<usize>,
    /// Distance threshold for anomaly classification.
    pub anomaly_distance_threshold: f32,
}

/// Retention-due projection row for one embedding.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataLayerM5RetentionDueCandidate {
    /// Owner DID scope.
    pub owner_did: String,
    /// Embedding identifier.
    pub embedding_id: String,
    /// Source message identifier.
    pub message_id: String,
    /// Retention class used for due calculation.
    pub retention_class: ContentRetentionClass,
    /// Due timestamp in epoch seconds.
    pub due_at_epoch_seconds: u64,
    /// Stable reason marker.
    pub reason_code: &'static str,
}

/// Recall-drift evaluation input for owner-scoped semantic top-k outputs.
#[derive(Debug, Clone, PartialEq)]
pub struct DataLayerM5RecallDriftEvaluationInput {
    /// Owner DID scope.
    pub owner_did: String,
    /// Query vector used for current semantic ranking.
    pub query_vector: Vec<f32>,
    /// Baseline top-k embedding-id order expected for the same query.
    pub baseline_top_k_embedding_ids: Vec<String>,
    /// Minimum required recall ratio in `[0.0, 1.0]`.
    pub min_recall_at_k: f32,
    /// Maximum allowed absolute rank shift per matched embedding-id.
    pub max_allowed_rank_shift: usize,
}

/// Recall-drift decision marker.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DataLayerM5RecallDriftDecision {
    /// Current ranking stayed within configured drift thresholds.
    Stable,
    /// Current ranking degraded beyond configured drift thresholds.
    Degraded,
}

/// Recall-drift evaluation report with deterministic evidence fields.
#[derive(Debug, Clone, PartialEq)]
pub struct DataLayerM5RecallDriftReport {
    /// Drift decision.
    pub decision: DataLayerM5RecallDriftDecision,
    /// Stable reason marker.
    pub reason_code: &'static str,
    /// Evaluated top-k size from baseline.
    pub evaluated_k: usize,
    /// Recall-at-k ratio for baseline IDs present in current top-k.
    pub recall_at_k: f32,
    /// Maximum absolute rank shift observed in matched IDs.
    pub max_observed_rank_shift: usize,
    /// Baseline IDs found in current top-k, preserving baseline order.
    pub matched_embedding_ids: Vec<String>,
    /// Baseline IDs missing from current top-k, preserving baseline order.
    pub missing_embedding_ids: Vec<String>,
    /// Current top-k embedding-id order returned by semantic query.
    pub current_top_k_embedding_ids: Vec<String>,
}

/// Anomaly decision result.
#[derive(Debug, Clone, PartialEq)]
pub enum DataLayerM5AnomalyDecision {
    /// Candidate vector is within configured anomaly threshold.
    Normal {
        /// Stable reason marker.
        reason_code: &'static str,
        /// Computed centroid distance.
        centroid_distance: f32,
    },
    /// Candidate vector exceeds configured anomaly threshold.
    Anomalous {
        /// Stable reason marker.
        reason_code: &'static str,
        /// Computed centroid distance.
        centroid_distance: f32,
    },
}

/// M5 embedding registry, semantic query, and anomaly evaluation engine.
#[derive(Debug, Clone, PartialEq)]
pub struct DataLayerM5EmbeddingRegistry {
    privacy_mode: DataLayerM5EmbeddingPrivacyMode,
    records_by_owner: BTreeMap<String, Vec<DataLayerM5EmbeddingRecord>>,
    seen_embedding_ids: BTreeSet<String>,
}

impl DataLayerM5EmbeddingRegistry {
    /// Creates an empty registry for one privacy mode.
    pub fn new(privacy_mode: DataLayerM5EmbeddingPrivacyMode) -> Self {
        Self {
            privacy_mode,
            records_by_owner: BTreeMap::new(),
            seen_embedding_ids: BTreeSet::new(),
        }
    }

    /// Returns the configured privacy mode.
    pub fn privacy_mode(&self) -> DataLayerM5EmbeddingPrivacyMode {
        self.privacy_mode
    }

    /// Appends one embedding record under owner scope.
    pub fn append(
        &mut self,
        input: DataLayerM5EmbeddingRecordInput,
    ) -> Result<DataLayerM5EmbeddingRecord, DataLayerM5VectorIntegrationError> {
        validate_non_empty(input.embedding_id.as_str(), "embedding_id")?;
        validate_non_empty(input.message_id.as_str(), "message_id")?;
        validate_kamn_did(input.owner_did.as_str())?;
        let parsed_agent_did = validate_agent_did(input.agent_did.as_str())?;
        validate_non_empty(input.model_id.as_str(), "model_id")?;
        if input.vector_encrypted.is_empty() {
            return Err(DataLayerM5VectorIntegrationError::EmptyField(
                "vector_encrypted",
            ));
        }
        if input.created_at_epoch_seconds == 0 {
            return Err(DataLayerM5VectorIntegrationError::EmptyField(
                "created_at_epoch_seconds",
            ));
        }
        if self
            .seen_embedding_ids
            .contains(input.embedding_id.as_str())
        {
            return Err(DataLayerM5VectorIntegrationError::DuplicateEmbeddingId(
                input.embedding_id,
            ));
        }

        let vector_plaintext = match (self.privacy_mode, input.vector_plaintext) {
            (DataLayerM5EmbeddingPrivacyMode::OwnerSideEncrypted, Some(_)) => {
                return Err(DataLayerM5VectorIntegrationError::PrivacyModeViolation {
                    reason_code: DATA_LAYER_M5_OWNER_SIDE_PLAINTEXT_STORAGE_NOT_ALLOWED_REASON_CODE,
                });
            }
            (DataLayerM5EmbeddingPrivacyMode::OwnerSideEncrypted, None) => None,
            (DataLayerM5EmbeddingPrivacyMode::ServerSidePlaintextOptIn, None) => {
                return Err(DataLayerM5VectorIntegrationError::PrivacyModeViolation {
                    reason_code: DATA_LAYER_M5_SERVER_SIDE_PLAINTEXT_REQUIRED_REASON_CODE,
                });
            }
            (DataLayerM5EmbeddingPrivacyMode::ServerSidePlaintextOptIn, Some(vector)) => {
                Some(validate_vector(vector, "vector_plaintext")?)
            }
        };
        let vector_dimensions = vector_plaintext.as_ref().map_or(0, Vec::len);

        let owner_records = self
            .records_by_owner
            .entry(input.owner_did.clone())
            .or_default();
        if let Some(expected_dimensions) = owner_vector_dimensions(owner_records) {
            if vector_dimensions > 0 && vector_dimensions != expected_dimensions {
                return Err(DataLayerM5VectorIntegrationError::InvalidVectorDimensions {
                    expected: expected_dimensions,
                    found: vector_dimensions,
                });
            }
        }

        let sequence = owner_records.len() as u64 + 1;
        let hash_chain_prev = owner_records
            .last()
            .map(|record| record.record_hash.clone())
            .unwrap_or_else(|| DATA_LAYER_M5_EMBEDDING_HASH_CHAIN_GENESIS.to_owned());

        let record_hash_material = DataLayerM5RecordHashMaterial {
            embedding_id: input.embedding_id.as_str(),
            message_id: input.message_id.as_str(),
            owner_did: input.owner_did.as_str(),
            agent_did: parsed_agent_did.as_str(),
            retention_class: input.retention_class,
            model_id: input.model_id.as_str(),
            vector_encrypted: input.vector_encrypted.as_slice(),
            vector_plaintext: vector_plaintext.as_deref(),
            vector_dimensions,
            created_at_epoch_seconds: input.created_at_epoch_seconds,
            privacy_mode: self.privacy_mode,
        };
        let record_hash = compute_embedding_record_hash(
            sequence,
            &record_hash_material,
            hash_chain_prev.as_str(),
        );

        let record = DataLayerM5EmbeddingRecord {
            embedding_id: input.embedding_id.clone(),
            message_id: input.message_id,
            owner_did: input.owner_did.clone(),
            agent_did: parsed_agent_did.as_str().to_owned(),
            retention_class: input.retention_class,
            model_id: input.model_id,
            vector_encrypted: input.vector_encrypted,
            privacy_mode: self.privacy_mode,
            vector_plaintext,
            vector_dimensions,
            sequence,
            created_at_epoch_seconds: input.created_at_epoch_seconds,
            hash_chain_prev,
            record_hash,
        };
        owner_records.push(record.clone());
        self.seen_embedding_ids.insert(input.embedding_id);
        Ok(record)
    }

    /// Returns embedding records for one owner scope in append order.
    pub fn embedding_records(&self, owner_did: &str) -> Option<&[DataLayerM5EmbeddingRecord]> {
        self.records_by_owner.get(owner_did).map(Vec::as_slice)
    }

    /// Returns owner-scoped retention-due embeddings aligned with content lifecycle policy.
    pub fn retention_due_for_owner(
        &self,
        owner_did: &str,
        now_epoch_seconds: u64,
    ) -> Result<Vec<DataLayerM5RetentionDueCandidate>, DataLayerM5VectorIntegrationError> {
        validate_kamn_did(owner_did)?;
        if now_epoch_seconds == 0 {
            return Err(DataLayerM5VectorIntegrationError::EmptyField(
                "now_epoch_seconds",
            ));
        }

        let owner_records = self.records_by_owner.get(owner_did).ok_or_else(|| {
            DataLayerM5VectorIntegrationError::OwnerNotFound {
                owner_did: owner_did.to_owned(),
            }
        })?;

        let mut due_candidates = owner_records
            .iter()
            .filter_map(|record| {
                let retention_profile =
                    ContentLifecycleManager::retention_profile(record.retention_class);
                let due_at_epoch_seconds = record
                    .created_at_epoch_seconds
                    .saturating_add(retention_profile.retain_for_secs);
                if now_epoch_seconds >= due_at_epoch_seconds {
                    Some(DataLayerM5RetentionDueCandidate {
                        owner_did: record.owner_did.clone(),
                        embedding_id: record.embedding_id.clone(),
                        message_id: record.message_id.clone(),
                        retention_class: record.retention_class,
                        due_at_epoch_seconds,
                        reason_code: DATA_LAYER_M5_RETENTION_DUE_REASON_CODE,
                    })
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();

        due_candidates.sort_by(|left, right| {
            left.due_at_epoch_seconds
                .cmp(&right.due_at_epoch_seconds)
                .then(left.embedding_id.cmp(&right.embedding_id))
                .then(left.message_id.cmp(&right.message_id))
        });
        Ok(due_candidates)
    }

    /// Executes deterministic owner-scoped semantic top-k ranking.
    pub fn semantic_query(
        &self,
        query: DataLayerM5SemanticQuery,
    ) -> Result<Vec<DataLayerM5SemanticQueryResult>, DataLayerM5VectorIntegrationError> {
        validate_kamn_did(query.owner_did.as_str())?;
        let query_vector = validate_vector(query.query_vector, "query_vector")?;
        let limit = resolve_limit(query.limit)?;
        if self.privacy_mode == DataLayerM5EmbeddingPrivacyMode::OwnerSideEncrypted {
            return Err(
                DataLayerM5VectorIntegrationError::SemanticQueryUnavailable {
                    reason_code: DATA_LAYER_M5_OWNER_SIDE_QUERY_REQUIRES_LOCAL_INDEX_REASON_CODE,
                },
            );
        }

        let owner_records = self
            .records_by_owner
            .get(query.owner_did.as_str())
            .ok_or_else(|| DataLayerM5VectorIntegrationError::OwnerNotFound {
                owner_did: query.owner_did.clone(),
            })?;
        let expected_dimensions = owner_vector_dimensions(owner_records).ok_or(
            DataLayerM5VectorIntegrationError::SemanticQueryUnavailable {
                reason_code: DATA_LAYER_M5_PLAINTEXT_INDEX_MISSING_FOR_OWNER_SCOPE_REASON_CODE,
            },
        )?;
        if query_vector.len() != expected_dimensions {
            return Err(DataLayerM5VectorIntegrationError::InvalidVectorDimensions {
                expected: expected_dimensions,
                found: query_vector.len(),
            });
        }

        let mut rows = owner_records
            .iter()
            .filter_map(|record| {
                record.vector_plaintext.as_ref().map(|vector| {
                    let similarity = cosine_similarity(query_vector.as_slice(), vector.as_slice())?;
                    Ok(DataLayerM5SemanticQueryResult {
                        embedding_id: record.embedding_id.clone(),
                        message_id: record.message_id.clone(),
                        similarity_score: similarity,
                        cosine_distance: 1.0 - similarity,
                    })
                })
            })
            .collect::<Result<Vec<_>, DataLayerM5VectorIntegrationError>>()?;

        rows.sort_by(|left, right| {
            right
                .similarity_score
                .total_cmp(&left.similarity_score)
                .then_with(|| left.message_id.cmp(&right.message_id))
                .then_with(|| left.embedding_id.cmp(&right.embedding_id))
        });
        if rows.len() > limit {
            rows.truncate(limit);
        }
        Ok(rows)
    }

    /// Evaluates semantic top-k recall drift against a deterministic baseline ranking.
    pub fn evaluate_recall_drift(
        &self,
        input: DataLayerM5RecallDriftEvaluationInput,
    ) -> Result<DataLayerM5RecallDriftReport, DataLayerM5VectorIntegrationError> {
        validate_kamn_did(input.owner_did.as_str())?;
        let query_vector = validate_vector(input.query_vector, "query_vector")?;
        if input.baseline_top_k_embedding_ids.is_empty() {
            return Err(DataLayerM5VectorIntegrationError::EmptyField(
                "baseline_top_k_embedding_ids",
            ));
        }
        if !input.min_recall_at_k.is_finite() || !(0.0..=1.0).contains(&input.min_recall_at_k) {
            return Err(DataLayerM5VectorIntegrationError::InvalidVectorValue(
                "min_recall_at_k",
            ));
        }

        let mut baseline_seen = BTreeSet::new();
        for embedding_id in &input.baseline_top_k_embedding_ids {
            validate_non_empty(embedding_id.as_str(), "baseline_top_k_embedding_id")?;
            if !baseline_seen.insert(embedding_id.clone()) {
                return Err(DataLayerM5VectorIntegrationError::DuplicateEmbeddingId(
                    embedding_id.clone(),
                ));
            }
        }

        let evaluated_k = input.baseline_top_k_embedding_ids.len();
        let current_top_k_embedding_ids = self
            .semantic_query(DataLayerM5SemanticQuery {
                owner_did: input.owner_did,
                query_vector,
                limit: Some(evaluated_k),
            })?
            .into_iter()
            .map(|row| row.embedding_id)
            .collect::<Vec<_>>();

        let current_rank_by_embedding_id = current_top_k_embedding_ids
            .iter()
            .enumerate()
            .map(|(rank, embedding_id)| (embedding_id.clone(), rank))
            .collect::<BTreeMap<_, _>>();

        let mut matched_embedding_ids = Vec::new();
        let mut missing_embedding_ids = Vec::new();
        let mut max_observed_rank_shift = 0usize;
        for (baseline_rank, baseline_embedding_id) in
            input.baseline_top_k_embedding_ids.iter().enumerate()
        {
            if let Some(current_rank) = current_rank_by_embedding_id.get(baseline_embedding_id) {
                matched_embedding_ids.push(baseline_embedding_id.clone());
                max_observed_rank_shift =
                    max_observed_rank_shift.max(baseline_rank.abs_diff(*current_rank));
            } else {
                missing_embedding_ids.push(baseline_embedding_id.clone());
            }
        }
        let recall_at_k = matched_embedding_ids.len() as f32 / evaluated_k as f32;
        let degraded = !missing_embedding_ids.is_empty()
            || recall_at_k < input.min_recall_at_k
            || max_observed_rank_shift > input.max_allowed_rank_shift;
        let (decision, reason_code) = if degraded {
            (
                DataLayerM5RecallDriftDecision::Degraded,
                DATA_LAYER_M5_RECALL_DRIFT_DEGRADED_REASON_CODE,
            )
        } else {
            (
                DataLayerM5RecallDriftDecision::Stable,
                DATA_LAYER_M5_RECALL_DRIFT_STABLE_REASON_CODE,
            )
        };

        Ok(DataLayerM5RecallDriftReport {
            decision,
            reason_code,
            evaluated_k,
            recall_at_k,
            max_observed_rank_shift,
            matched_embedding_ids,
            missing_embedding_ids,
            current_top_k_embedding_ids,
        })
    }

    /// Evaluates anomaly decision for one candidate vector relative to agent centroid history.
    pub fn evaluate_agent_anomaly(
        &self,
        input: DataLayerM5AnomalyEvaluationInput,
    ) -> Result<DataLayerM5AnomalyDecision, DataLayerM5VectorIntegrationError> {
        validate_kamn_did(input.owner_did.as_str())?;
        let parsed_agent_did = validate_agent_did(input.agent_did.as_str())?;
        let candidate_vector = validate_vector(input.candidate_vector, "candidate_vector")?;
        if !input.anomaly_distance_threshold.is_finite() || input.anomaly_distance_threshold <= 0.0
        {
            return Err(DataLayerM5VectorIntegrationError::InvalidVectorValue(
                "anomaly_distance_threshold",
            ));
        }
        let lookback_window = resolve_lookback_window(input.lookback_window)?;

        if self.privacy_mode == DataLayerM5EmbeddingPrivacyMode::OwnerSideEncrypted {
            return Err(
                DataLayerM5VectorIntegrationError::AnomalyEvaluationUnavailable {
                    reason_code:
                        DATA_LAYER_M5_OWNER_SIDE_ANOMALY_REQUIRES_LOCAL_PIPELINE_REASON_CODE,
                },
            );
        }

        let owner_records = self
            .records_by_owner
            .get(input.owner_did.as_str())
            .ok_or_else(|| DataLayerM5VectorIntegrationError::OwnerNotFound {
                owner_did: input.owner_did.clone(),
            })?;
        let mut agent_vectors = owner_records
            .iter()
            .filter(|record| record.agent_did == parsed_agent_did.as_str())
            .filter_map(|record| record.vector_plaintext.as_ref().cloned())
            .collect::<Vec<_>>();
        if agent_vectors.is_empty() {
            return Err(
                DataLayerM5VectorIntegrationError::InsufficientAgentHistory {
                    owner_did: input.owner_did,
                    agent_did: parsed_agent_did.as_str().to_owned(),
                },
            );
        }

        if agent_vectors.len() > lookback_window {
            let keep_from = agent_vectors.len() - lookback_window;
            agent_vectors = agent_vectors.split_off(keep_from);
        }

        let expected_dimensions = agent_vectors[0].len();
        if candidate_vector.len() != expected_dimensions {
            return Err(DataLayerM5VectorIntegrationError::InvalidVectorDimensions {
                expected: expected_dimensions,
                found: candidate_vector.len(),
            });
        }
        if agent_vectors
            .iter()
            .any(|vector| vector.len() != expected_dimensions)
        {
            return Err(DataLayerM5VectorIntegrationError::InvalidVectorDimensions {
                expected: expected_dimensions,
                found: 0,
            });
        }

        let centroid = compute_centroid(agent_vectors.as_slice());
        let centroid_distance =
            1.0 - cosine_similarity(candidate_vector.as_slice(), centroid.as_slice())?;
        if centroid_distance > input.anomaly_distance_threshold {
            return Ok(DataLayerM5AnomalyDecision::Anomalous {
                reason_code: DATA_LAYER_M5_ANOMALY_THRESHOLD_EXCEEDED_REASON_CODE,
                centroid_distance,
            });
        }
        Ok(DataLayerM5AnomalyDecision::Normal {
            reason_code: DATA_LAYER_M5_ANOMALY_WITHIN_THRESHOLD_REASON_CODE,
            centroid_distance,
        })
    }

    /// Verifies hash-chain integrity for one owner-scoped embedding stream.
    pub fn verify_owner_integrity(
        &self,
        owner_did: &str,
    ) -> Result<(), DataLayerM5VectorIntegrationError> {
        validate_kamn_did(owner_did)?;
        let records = self.records_by_owner.get(owner_did).ok_or_else(|| {
            DataLayerM5VectorIntegrationError::OwnerNotFound {
                owner_did: owner_did.to_owned(),
            }
        })?;

        let mut expected_prev = DATA_LAYER_M5_EMBEDDING_HASH_CHAIN_GENESIS.to_owned();
        for (position, record) in records.iter().enumerate() {
            if record.hash_chain_prev != expected_prev {
                return Err(
                    DataLayerM5VectorIntegrationError::InvalidEmbeddingHashChain {
                        owner_did: owner_did.to_owned(),
                        position,
                        reason: "hash_chain_prev mismatch",
                    },
                );
            }
            let hash_material = DataLayerM5RecordHashMaterial {
                embedding_id: record.embedding_id.as_str(),
                message_id: record.message_id.as_str(),
                owner_did: record.owner_did.as_str(),
                agent_did: record.agent_did.as_str(),
                retention_class: record.retention_class,
                model_id: record.model_id.as_str(),
                vector_encrypted: record.vector_encrypted.as_slice(),
                vector_plaintext: record.vector_plaintext.as_deref(),
                vector_dimensions: record.vector_dimensions,
                created_at_epoch_seconds: record.created_at_epoch_seconds,
                privacy_mode: record.privacy_mode,
            };
            let expected_hash = compute_embedding_record_hash(
                record.sequence,
                &hash_material,
                record.hash_chain_prev.as_str(),
            );
            if record.record_hash != expected_hash {
                return Err(
                    DataLayerM5VectorIntegrationError::InvalidEmbeddingHashChain {
                        owner_did: owner_did.to_owned(),
                        position,
                        reason: "record_hash mismatch",
                    },
                );
            }
            expected_prev = record.record_hash.clone();
        }
        Ok(())
    }

    /// Replaces one record hash without recomputing chain links.
    ///
    /// This helper intentionally bypasses integrity checks for tamper regression tests.
    pub fn replace_record_hash_unchecked(
        &mut self,
        owner_did: &str,
        sequence: u64,
        record_hash: &str,
    ) -> Result<(), DataLayerM5VectorIntegrationError> {
        validate_kamn_did(owner_did)?;
        validate_non_empty(record_hash, "record_hash")?;
        let records = self.records_by_owner.get_mut(owner_did).ok_or_else(|| {
            DataLayerM5VectorIntegrationError::OwnerNotFound {
                owner_did: owner_did.to_owned(),
            }
        })?;
        let record = records
            .iter_mut()
            .find(|entry| entry.sequence == sequence)
            .ok_or_else(
                || DataLayerM5VectorIntegrationError::EmbeddingSequenceNotFound {
                    owner_did: owner_did.to_owned(),
                    sequence,
                },
            )?;
        record.record_hash = record_hash.to_owned();
        Ok(())
    }
}

/// Error taxonomy for M5 vector-layer contracts.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DataLayerM5VectorIntegrationError {
    /// Required field was empty.
    EmptyField(&'static str),
    /// DID failed validation.
    InvalidDid(String),
    /// Agent DID failed canonical parser validation.
    InvalidAgentDid {
        /// Stable reason marker.
        reason_code: &'static str,
        /// Stable detail from parser.
        detail: String,
    },
    /// Duplicate embedding identifier registration was attempted.
    DuplicateEmbeddingId(String),
    /// Vector dimensions did not match expected shape.
    InvalidVectorDimensions {
        /// Expected vector length.
        expected: usize,
        /// Found vector length.
        found: usize,
    },
    /// Vector value or scalar control was invalid.
    InvalidVectorValue(&'static str),
    /// Query limit must be positive.
    InvalidLimit(usize),
    /// Lookback window must be positive.
    InvalidLookbackWindow(usize),
    /// Owner scope was not found.
    OwnerNotFound {
        /// Missing owner DID.
        owner_did: String,
    },
    /// Semantic query is unavailable under current mode/configuration.
    SemanticQueryUnavailable {
        /// Stable reason marker.
        reason_code: &'static str,
    },
    /// Anomaly evaluation is unavailable under current mode/configuration.
    AnomalyEvaluationUnavailable {
        /// Stable reason marker.
        reason_code: &'static str,
    },
    /// Privacy-mode ingestion contract was violated.
    PrivacyModeViolation {
        /// Stable reason marker.
        reason_code: &'static str,
    },
    /// No agent history was available for anomaly evaluation.
    InsufficientAgentHistory {
        /// Owner DID scope.
        owner_did: String,
        /// Agent DID scope.
        agent_did: String,
    },
    /// Owner embedding hash-chain integrity failed.
    InvalidEmbeddingHashChain {
        /// Owner DID scope.
        owner_did: String,
        /// Zero-based record position.
        position: usize,
        /// Mismatch reason marker.
        reason: &'static str,
    },
    /// Owner record sequence was not found.
    EmbeddingSequenceNotFound {
        /// Owner DID scope.
        owner_did: String,
        /// Missing sequence.
        sequence: u64,
    },
}

impl fmt::Display for DataLayerM5VectorIntegrationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyField(field) => write!(f, "{field} must not be empty"),
            Self::InvalidDid(value) => write!(f, "invalid did: {value}"),
            Self::InvalidAgentDid {
                reason_code,
                detail,
            } => {
                write!(f, "invalid agent did: {reason_code} ({detail})")
            }
            Self::DuplicateEmbeddingId(embedding_id) => {
                write!(f, "duplicate embedding_id: {embedding_id}")
            }
            Self::InvalidVectorDimensions { expected, found } => {
                write!(
                    f,
                    "invalid vector dimensions: expected {expected}, found {found}"
                )
            }
            Self::InvalidVectorValue(field) => write!(f, "invalid vector value for {field}"),
            Self::InvalidLimit(limit) => write!(f, "invalid limit: {limit}"),
            Self::InvalidLookbackWindow(window) => write!(f, "invalid lookback window: {window}"),
            Self::OwnerNotFound { owner_did } => write!(f, "owner not found: {owner_did}"),
            Self::SemanticQueryUnavailable { reason_code } => {
                write!(f, "semantic query unavailable: {reason_code}")
            }
            Self::AnomalyEvaluationUnavailable { reason_code } => {
                write!(f, "anomaly evaluation unavailable: {reason_code}")
            }
            Self::PrivacyModeViolation { reason_code } => {
                write!(f, "privacy mode violation: {reason_code}")
            }
            Self::InsufficientAgentHistory {
                owner_did,
                agent_did,
            } => {
                write!(
                    f,
                    "insufficient agent history for owner {owner_did}, agent {agent_did}"
                )
            }
            Self::InvalidEmbeddingHashChain {
                owner_did,
                position,
                reason,
            } => {
                write!(
                    f,
                    "invalid embedding hash chain for {owner_did} at {position}: {reason}"
                )
            }
            Self::EmbeddingSequenceNotFound {
                owner_did,
                sequence,
            } => {
                write!(
                    f,
                    "embedding sequence not found for owner {owner_did}: {sequence}"
                )
            }
        }
    }
}

impl std::error::Error for DataLayerM5VectorIntegrationError {}

fn validate_non_empty(
    value: &str,
    field_name: &'static str,
) -> Result<(), DataLayerM5VectorIntegrationError> {
    if value.trim().is_empty() {
        return Err(DataLayerM5VectorIntegrationError::EmptyField(field_name));
    }
    Ok(())
}

fn validate_kamn_did(value: &str) -> Result<(), DataLayerM5VectorIntegrationError> {
    let trimmed = value.trim();
    if trimmed.is_empty() || !trimmed.starts_with("kamn:did:") {
        return Err(DataLayerM5VectorIntegrationError::InvalidDid(
            value.to_owned(),
        ));
    }
    let segments = trimmed.split(':').collect::<Vec<_>>();
    if segments.len() < 4 || segments.iter().any(|segment| segment.is_empty()) {
        return Err(DataLayerM5VectorIntegrationError::InvalidDid(
            value.to_owned(),
        ));
    }
    Ok(())
}

fn map_agent_did_error_to_m5(error: AgentDidError) -> DataLayerM5VectorIntegrationError {
    DataLayerM5VectorIntegrationError::InvalidAgentDid {
        reason_code: DATA_LAYER_M5_INVALID_AGENT_DID_REASON_CODE,
        detail: error.to_string(),
    }
}

fn validate_agent_did(value: &str) -> Result<AgentDid, DataLayerM5VectorIntegrationError> {
    AgentDid::parse(value).map_err(map_agent_did_error_to_m5)
}

fn validate_vector(
    vector: Vec<f32>,
    field_name: &'static str,
) -> Result<Vec<f32>, DataLayerM5VectorIntegrationError> {
    if vector.is_empty() {
        return Err(DataLayerM5VectorIntegrationError::EmptyField(field_name));
    }
    if vector.iter().any(|value| !value.is_finite()) {
        return Err(DataLayerM5VectorIntegrationError::InvalidVectorValue(
            field_name,
        ));
    }
    Ok(vector)
}

fn resolve_limit(limit: Option<usize>) -> Result<usize, DataLayerM5VectorIntegrationError> {
    let resolved = limit.unwrap_or(20);
    if resolved == 0 {
        return Err(DataLayerM5VectorIntegrationError::InvalidLimit(resolved));
    }
    Ok(resolved)
}

fn resolve_lookback_window(
    lookback_window: Option<usize>,
) -> Result<usize, DataLayerM5VectorIntegrationError> {
    let resolved = lookback_window.unwrap_or(500);
    if resolved == 0 {
        return Err(DataLayerM5VectorIntegrationError::InvalidLookbackWindow(
            resolved,
        ));
    }
    Ok(resolved)
}

fn owner_vector_dimensions(records: &[DataLayerM5EmbeddingRecord]) -> Option<usize> {
    records
        .iter()
        .find_map(|record| record.vector_plaintext.as_ref().map(Vec::len))
}

fn compute_centroid(vectors: &[Vec<f32>]) -> Vec<f32> {
    let dimensions = vectors[0].len();
    let mut accum = vec![0.0_f32; dimensions];
    for vector in vectors {
        for (index, value) in vector.iter().enumerate() {
            accum[index] += *value;
        }
    }
    let divisor = vectors.len() as f32;
    accum.iter_mut().for_each(|value| *value /= divisor);
    accum
}

fn cosine_similarity(
    left: &[f32],
    right: &[f32],
) -> Result<f32, DataLayerM5VectorIntegrationError> {
    if left.len() != right.len() {
        return Err(DataLayerM5VectorIntegrationError::InvalidVectorDimensions {
            expected: left.len(),
            found: right.len(),
        });
    }
    let mut dot = 0.0_f64;
    let mut left_norm = 0.0_f64;
    let mut right_norm = 0.0_f64;
    for (left_value, right_value) in left.iter().zip(right.iter()) {
        dot += *left_value as f64 * *right_value as f64;
        left_norm += (*left_value as f64).powi(2);
        right_norm += (*right_value as f64).powi(2);
    }
    if left_norm <= f64::EPSILON || right_norm <= f64::EPSILON {
        return Err(DataLayerM5VectorIntegrationError::InvalidVectorValue(
            "zero_norm_vector",
        ));
    }
    Ok((dot / (left_norm.sqrt() * right_norm.sqrt())) as f32)
}

struct DataLayerM5RecordHashMaterial<'a> {
    embedding_id: &'a str,
    message_id: &'a str,
    owner_did: &'a str,
    agent_did: &'a str,
    retention_class: ContentRetentionClass,
    model_id: &'a str,
    vector_encrypted: &'a [u8],
    vector_plaintext: Option<&'a [f32]>,
    vector_dimensions: usize,
    created_at_epoch_seconds: u64,
    privacy_mode: DataLayerM5EmbeddingPrivacyMode,
}

fn compute_embedding_record_hash(
    sequence: u64,
    material: &DataLayerM5RecordHashMaterial<'_>,
    hash_chain_prev: &str,
) -> String {
    tagged_digest(
        format!(
            "m5-embedding|seq:{sequence}|embedding:{embedding_id}|message:{message_id}|owner:{owner_did}|agent:{agent_did}|retention:{retention_class}|model:{model_id}|encrypted:{}|plaintext:{}|dims:{vector_dimensions}|created:{created_at_epoch_seconds}|mode:{}|metric:{}|prev:{hash_chain_prev}",
            bytes_marker(material.vector_encrypted),
            vector_marker(material.vector_plaintext),
            material.privacy_mode.marker(),
            DATA_LAYER_M5_VECTOR_DISTANCE_METRIC_COSINE,
            embedding_id = material.embedding_id,
            message_id = material.message_id,
            owner_did = material.owner_did,
            agent_did = material.agent_did,
            retention_class = retention_class_marker(material.retention_class),
            model_id = material.model_id,
            vector_dimensions = material.vector_dimensions,
            created_at_epoch_seconds = material.created_at_epoch_seconds
        )
        .as_str(),
    )
}

fn retention_class_marker(class: ContentRetentionClass) -> &'static str {
    match class {
        ContentRetentionClass::ShortLived => "short_lived",
        ContentRetentionClass::Standard => "standard",
        ContentRetentionClass::Compliance => "compliance",
    }
}

fn bytes_marker(value: &[u8]) -> String {
    if value.is_empty() {
        return "none".to_owned();
    }
    value
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<Vec<_>>()
        .join("")
}

fn vector_marker(value: Option<&[f32]>) -> String {
    match value {
        Some(vector) => vector
            .iter()
            .map(|coordinate| format!("{coordinate:.8}"))
            .collect::<Vec<_>>()
            .join(","),
        None => "none".to_owned(),
    }
}

fn tagged_digest(value: &str) -> String {
    format!(
        "{DATA_LAYER_M5_HASH_ALGORITHM}:{}",
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
