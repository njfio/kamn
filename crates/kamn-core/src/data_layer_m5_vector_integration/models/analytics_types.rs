use crate::ContentRetentionClass;

/// Anomaly evaluation input contract.
#[derive(Debug, Clone, PartialEq)]
pub struct DataLayerM5AnomalyEvaluationInput {
    pub owner_did: String,
    pub agent_did: String,
    pub candidate_vector: Vec<f32>,
    pub lookback_window: Option<usize>,
    pub anomaly_distance_threshold: f32,
}

/// Retention-due projection row for one embedding.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataLayerM5RetentionDueCandidate {
    pub owner_did: String,
    pub embedding_id: String,
    pub message_id: String,
    pub retention_class: ContentRetentionClass,
    pub due_at_epoch_seconds: u64,
    pub reason_code: &'static str,
}

/// Recall-drift evaluation input for owner-scoped semantic top-k outputs.
#[derive(Debug, Clone, PartialEq)]
pub struct DataLayerM5RecallDriftEvaluationInput {
    pub owner_did: String,
    pub query_vector: Vec<f32>,
    pub baseline_top_k_embedding_ids: Vec<String>,
    pub min_recall_at_k: f32,
    pub max_allowed_rank_shift: usize,
}

/// Recall-drift decision marker.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DataLayerM5RecallDriftDecision {
    Stable,
    Degraded,
}

/// Recall-drift evaluation report with deterministic evidence fields.
#[derive(Debug, Clone, PartialEq)]
pub struct DataLayerM5RecallDriftReport {
    pub decision: DataLayerM5RecallDriftDecision,
    pub reason_code: &'static str,
    pub evaluated_k: usize,
    pub recall_at_k: f32,
    pub max_observed_rank_shift: usize,
    pub matched_embedding_ids: Vec<String>,
    pub missing_embedding_ids: Vec<String>,
    pub current_top_k_embedding_ids: Vec<String>,
}

/// Anomaly decision result.
#[derive(Debug, Clone, PartialEq)]
pub enum DataLayerM5AnomalyDecision {
    Normal {
        reason_code: &'static str,
        centroid_distance: f32,
    },
    Anomalous {
        reason_code: &'static str,
        centroid_distance: f32,
    },
}
