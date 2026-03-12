use super::super::models::*;
use super::super::support::{parse_kamn_did, validate_non_empty, validate_vector};
use std::collections::{BTreeMap, BTreeSet};

impl DataLayerM5EmbeddingRegistry {
    /// Evaluates semantic top-k recall drift against a deterministic baseline ranking.
    pub fn evaluate_recall_drift(
        &self,
        input: DataLayerM5RecallDriftEvaluationInput,
    ) -> Result<DataLayerM5RecallDriftReport, DataLayerM5VectorIntegrationError> {
        validate_recall_input(&input)?;
        let owner_did = parse_kamn_did(input.owner_did.as_str())?;
        let query_vector = validate_vector(input.query_vector, "query_vector")?;
        let evaluated_k = input.baseline_top_k_embedding_ids.len();
        let current_top_k_embedding_ids = self
            .semantic_query(DataLayerM5SemanticQuery {
                owner_did: owner_did.as_str().to_owned(),
                query_vector,
                limit: Some(evaluated_k),
            })?
            .into_iter()
            .map(|row| row.embedding_id)
            .collect::<Vec<_>>();
        Ok(build_recall_report(
            &input.baseline_top_k_embedding_ids,
            &current_top_k_embedding_ids,
            input.min_recall_at_k,
            input.max_allowed_rank_shift,
        ))
    }
}

fn validate_recall_input(
    input: &DataLayerM5RecallDriftEvaluationInput,
) -> Result<(), DataLayerM5VectorIntegrationError> {
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
    Ok(())
}

fn build_recall_report(
    baseline_top_k_embedding_ids: &[String],
    current_top_k_embedding_ids: &[String],
    min_recall_at_k: f32,
    max_allowed_rank_shift: usize,
) -> DataLayerM5RecallDriftReport {
    let current_rank_by_embedding_id = current_top_k_embedding_ids
        .iter()
        .enumerate()
        .map(|(rank, embedding_id)| (embedding_id.clone(), rank))
        .collect::<BTreeMap<_, _>>();
    let mut matched_embedding_ids = Vec::new();
    let mut missing_embedding_ids = Vec::new();
    let mut max_observed_rank_shift = 0usize;
    for (baseline_rank, baseline_embedding_id) in baseline_top_k_embedding_ids.iter().enumerate() {
        if let Some(current_rank) = current_rank_by_embedding_id.get(baseline_embedding_id) {
            matched_embedding_ids.push(baseline_embedding_id.clone());
            max_observed_rank_shift = max_observed_rank_shift.max(baseline_rank.abs_diff(*current_rank));
        } else {
            missing_embedding_ids.push(baseline_embedding_id.clone());
        }
    }
    let evaluated_k = baseline_top_k_embedding_ids.len();
    let recall_at_k = matched_embedding_ids.len() as f32 / evaluated_k as f32;
    let degraded = !missing_embedding_ids.is_empty()
        || recall_at_k < min_recall_at_k
        || max_observed_rank_shift > max_allowed_rank_shift;
    let (decision, reason_code) = if degraded {
        (DataLayerM5RecallDriftDecision::Degraded, DATA_LAYER_M5_RECALL_DRIFT_DEGRADED_REASON_CODE)
    } else {
        (DataLayerM5RecallDriftDecision::Stable, DATA_LAYER_M5_RECALL_DRIFT_STABLE_REASON_CODE)
    };
    DataLayerM5RecallDriftReport {
        decision,
        reason_code,
        evaluated_k,
        recall_at_k,
        max_observed_rank_shift,
        matched_embedding_ids,
        missing_embedding_ids,
        current_top_k_embedding_ids: current_top_k_embedding_ids.to_vec(),
    }
}
