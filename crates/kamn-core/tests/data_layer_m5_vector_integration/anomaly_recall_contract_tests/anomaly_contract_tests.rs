use kamn_core::{
    DataLayerM5AnomalyDecision, DataLayerM5AnomalyEvaluationInput, DataLayerM5EmbeddingPrivacyMode,
    DataLayerM5EmbeddingRegistry, DATA_LAYER_M5_ANOMALY_THRESHOLD_EXCEEDED_REASON_CODE,
    DATA_LAYER_M5_ANOMALY_WITHIN_THRESHOLD_REASON_CODE,
};

use super::super::support::vector_input;

#[test]
fn spec_c05_anomaly_threshold_detection_uses_centroid_distance_rules() {
    let mut registry = DataLayerM5EmbeddingRegistry::new(
        DataLayerM5EmbeddingPrivacyMode::ServerSidePlaintextOptIn,
    );
    append_history(&mut registry);

    assert_anomaly_decision(
        evaluate_candidate(&registry, vec![0.0, 1.0, 0.0], 0.4),
        DATA_LAYER_M5_ANOMALY_THRESHOLD_EXCEEDED_REASON_CODE,
    );
    assert_normal_decision(
        evaluate_candidate(&registry, vec![1.0, 0.0, 0.0], 0.9),
        DATA_LAYER_M5_ANOMALY_WITHIN_THRESHOLD_REASON_CODE,
    );
}

fn append_history(registry: &mut DataLayerM5EmbeddingRegistry) {
    for (embedding_id, message_id, vector) in [
        ("embed-m5-h1", "msg-m5-h1", vec![1.0, 0.0, 0.0]),
        ("embed-m5-h2", "msg-m5-h2", vec![0.95, 0.05, 0.0]),
        ("embed-m5-h3", "msg-m5-h3", vec![0.9, 0.1, 0.0]),
    ] {
        registry
            .append(vector_input(
                embedding_id,
                message_id,
                "kamn:did:owner:alpha",
                "kamn:did:agent:alpha",
                Some(vector),
            ))
            .expect("history append should succeed");
    }
}

fn evaluate_candidate(
    registry: &DataLayerM5EmbeddingRegistry,
    candidate_vector: Vec<f32>,
    anomaly_distance_threshold: f32,
) -> DataLayerM5AnomalyDecision {
    registry
        .evaluate_agent_anomaly(DataLayerM5AnomalyEvaluationInput {
            owner_did: "kamn:did:owner:alpha".to_owned(),
            agent_did: "kamn:did:agent:alpha".to_owned(),
            candidate_vector,
            lookback_window: Some(3),
            anomaly_distance_threshold,
        })
        .expect("anomaly evaluation should succeed")
}

fn assert_anomaly_decision(decision: DataLayerM5AnomalyDecision, reason_code: &str) {
    assert!(matches!(
        decision,
        DataLayerM5AnomalyDecision::Anomalous {
            reason_code: actual_reason_code,
            ..
        } if actual_reason_code == reason_code
    ));
}

fn assert_normal_decision(decision: DataLayerM5AnomalyDecision, reason_code: &str) {
    assert!(matches!(
        decision,
        DataLayerM5AnomalyDecision::Normal {
            reason_code: actual_reason_code,
            ..
        } if actual_reason_code == reason_code
    ));
}
