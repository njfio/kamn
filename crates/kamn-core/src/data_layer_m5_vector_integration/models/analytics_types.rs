use crate::ContentRetentionClass;

/// Anomaly evaluation input contract.
#[derive(Debug, Clone, PartialEq)]
pub struct DataLayerM5AnomalyEvaluationInput {
    /// Owner did carried by this public contract model.
    pub owner_did: String,
    /// Agent did carried by this public contract model.
    pub agent_did: String,
    /// Candidate vector carried by this public contract model.
    pub candidate_vector: Vec<f32>,
    /// Lookback window carried by this public contract model.
    pub lookback_window: Option<usize>,
    /// Anomaly distance threshold carried by this public contract model.
    pub anomaly_distance_threshold: f32,
}

/// Retention-due projection row for one embedding.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataLayerM5RetentionDueCandidate {
    /// Owner did carried by this public contract model.
    pub owner_did: String,
    /// Embedding id carried by this public contract model.
    pub embedding_id: String,
    /// Message id carried by this public contract model.
    pub message_id: String,
    /// Retention class carried by this public contract model.
    pub retention_class: ContentRetentionClass,
    /// Due at epoch seconds carried by this public contract model.
    pub due_at_epoch_seconds: u64,
    /// Reason code carried by this public contract model.
    pub reason_code: &'static str,
}

/// Recall-drift evaluation input for owner-scoped semantic top-k outputs.
#[derive(Debug, Clone, PartialEq)]
pub struct DataLayerM5RecallDriftEvaluationInput {
    /// Owner did carried by this public contract model.
    pub owner_did: String,
    /// Query vector carried by this public contract model.
    pub query_vector: Vec<f32>,
    /// Baseline top k embedding ids carried by this public contract model.
    pub baseline_top_k_embedding_ids: Vec<String>,
    /// Min recall at k carried by this public contract model.
    pub min_recall_at_k: f32,
    /// Max allowed rank shift carried by this public contract model.
    pub max_allowed_rank_shift: usize,
}

/// Recall-drift decision marker.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DataLayerM5RecallDriftDecision {
    /// Stable variant for this public contract enum.
    Stable,
    /// Degraded variant for this public contract enum.
    Degraded,
}

/// Recall-drift evaluation report with deterministic evidence fields.
#[derive(Debug, Clone, PartialEq)]
pub struct DataLayerM5RecallDriftReport {
    /// Decision carried by this public contract model.
    pub decision: DataLayerM5RecallDriftDecision,
    /// Reason code carried by this public contract model.
    pub reason_code: &'static str,
    /// Evaluated k carried by this public contract model.
    pub evaluated_k: usize,
    /// Recall at k carried by this public contract model.
    pub recall_at_k: f32,
    /// Max observed rank shift carried by this public contract model.
    pub max_observed_rank_shift: usize,
    /// Matched embedding ids carried by this public contract model.
    pub matched_embedding_ids: Vec<String>,
    /// Missing embedding ids carried by this public contract model.
    pub missing_embedding_ids: Vec<String>,
    /// Current top k embedding ids carried by this public contract model.
    pub current_top_k_embedding_ids: Vec<String>,
}

/// Anomaly decision result.
#[derive(Debug, Clone, PartialEq)]
pub enum DataLayerM5AnomalyDecision {
    /// Normal variant for this public contract enum.
    Normal {
        /// Str carried by this public contract model.
        reason_code: &'static str,
        /// F32 carried by this public contract model.
        centroid_distance: f32,
    },
    /// Anomalous variant for this public contract enum.
    Anomalous {
        /// Str carried by this public contract model.
        reason_code: &'static str,
        /// F32 carried by this public contract model.
        centroid_distance: f32,
    },
}
