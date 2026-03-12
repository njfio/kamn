use super::super::models::*;
use super::super::support::{compute_centroid, cosine_similarity, parse_kamn_did, resolve_lookback_window, validate_agent_did, validate_vector};

impl DataLayerM5EmbeddingRegistry {
    /// Evaluates anomaly decision for one candidate vector relative to agent centroid history.
    pub fn evaluate_agent_anomaly(
        &self,
        input: DataLayerM5AnomalyEvaluationInput,
    ) -> Result<DataLayerM5AnomalyDecision, DataLayerM5VectorIntegrationError> {
        let owner_did = parse_kamn_did(input.owner_did.as_str())?;
        let parsed_agent_did = validate_agent_did(input.agent_did.as_str())?;
        let candidate_vector = validate_vector(input.candidate_vector, "candidate_vector")?;
        validate_anomaly_threshold(input.anomaly_distance_threshold)?;
        let lookback_window = resolve_lookback_window(input.lookback_window)?;
        require_anomaly_mode(self.privacy_mode())?;
        let mut agent_vectors = owner_agent_vectors(self, owner_did.as_str(), parsed_agent_did.as_str())?;
        trim_lookback(&mut agent_vectors, lookback_window);
        validate_candidate_dimensions(&agent_vectors, candidate_vector.len())?;
        anomaly_decision(agent_vectors, candidate_vector, input.anomaly_distance_threshold)
    }
}

fn validate_anomaly_threshold(threshold: f32) -> Result<(), DataLayerM5VectorIntegrationError> {
    if !threshold.is_finite() || threshold <= 0.0 {
        return Err(DataLayerM5VectorIntegrationError::InvalidVectorValue(
            "anomaly_distance_threshold",
        ));
    }
    Ok(())
}

fn require_anomaly_mode(
    privacy_mode: DataLayerM5EmbeddingPrivacyMode,
) -> Result<(), DataLayerM5VectorIntegrationError> {
    if privacy_mode == DataLayerM5EmbeddingPrivacyMode::OwnerSideEncrypted {
        return Err(DataLayerM5VectorIntegrationError::AnomalyEvaluationUnavailable {
            reason_code: DATA_LAYER_M5_OWNER_SIDE_ANOMALY_REQUIRES_LOCAL_PIPELINE_REASON_CODE,
        });
    }
    Ok(())
}

fn owner_agent_vectors(
    registry: &DataLayerM5EmbeddingRegistry,
    owner_did: &str,
    agent_did: &str,
) -> Result<Vec<Vec<f32>>, DataLayerM5VectorIntegrationError> {
    let owner_records = registry.records_by_owner.get(owner_did).ok_or_else(|| {
        DataLayerM5VectorIntegrationError::OwnerNotFound {
            owner_did: owner_did.to_owned(),
        }
    })?;
    let agent_vectors = owner_records
        .iter()
        .filter(|record| record.agent_did == agent_did)
        .filter_map(|record| record.vector_plaintext.as_ref().cloned())
        .collect::<Vec<_>>();
    if agent_vectors.is_empty() {
        return Err(DataLayerM5VectorIntegrationError::InsufficientAgentHistory {
            owner_did: owner_did.to_owned(),
            agent_did: agent_did.to_owned(),
        });
    }
    Ok(agent_vectors)
}

fn trim_lookback(agent_vectors: &mut Vec<Vec<f32>>, lookback_window: usize) {
    if agent_vectors.len() > lookback_window {
        let keep_from = agent_vectors.len() - lookback_window;
        *agent_vectors = agent_vectors.split_off(keep_from);
    }
}

fn validate_candidate_dimensions(
    agent_vectors: &[Vec<f32>],
    candidate_dimensions: usize,
) -> Result<(), DataLayerM5VectorIntegrationError> {
    let expected_dimensions = agent_vectors[0].len();
    if candidate_dimensions != expected_dimensions {
        return Err(DataLayerM5VectorIntegrationError::InvalidVectorDimensions {
            expected: expected_dimensions,
            found: candidate_dimensions,
        });
    }
    if agent_vectors.iter().any(|vector| vector.len() != expected_dimensions) {
        return Err(DataLayerM5VectorIntegrationError::InvalidVectorDimensions {
            expected: expected_dimensions,
            found: 0,
        });
    }
    Ok(())
}

fn anomaly_decision(
    agent_vectors: Vec<Vec<f32>>,
    candidate_vector: Vec<f32>,
    anomaly_distance_threshold: f32,
) -> Result<DataLayerM5AnomalyDecision, DataLayerM5VectorIntegrationError> {
    let centroid = compute_centroid(agent_vectors.as_slice());
    let centroid_distance = 1.0 - cosine_similarity(candidate_vector.as_slice(), centroid.as_slice())?;
    if centroid_distance > anomaly_distance_threshold {
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
