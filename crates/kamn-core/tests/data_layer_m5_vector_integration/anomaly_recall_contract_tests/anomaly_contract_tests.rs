use kamn_core::{
    DataLayerM5AnomalyDecision, DataLayerM5AnomalyEvaluationInput,
    DataLayerM5EmbeddingPrivacyMode, DataLayerM5EmbeddingRegistry,
    DATA_LAYER_M5_ANOMALY_THRESHOLD_EXCEEDED_REASON_CODE,
    DATA_LAYER_M5_ANOMALY_WITHIN_THRESHOLD_REASON_CODE,
};

use super::super::support::vector_input;

#[test]
fn spec_c05_anomaly_threshold_detection_uses_centroid_distance_rules() {
    let mut registry =
        DataLayerM5EmbeddingRegistry::new(DataLayerM5EmbeddingPrivacyMode::ServerSidePlaintextOptIn);
    append_history(&mut registry);

    let decision = registry
        .evaluate_agent_anomaly(DataLayerM5AnomalyEvaluationInput {
            owner_did: "kamn:did:owner:alpha".to_owned(),
            agent_did: "kamn:did:agent:alpha".to_owned(),
            candidate_vector: vec![0.0, 1.0, 0.0],
            lookback_window: Some(3),
            anomaly_distance_threshold: 0.4,
        })
        .expect("anomaly evaluation should succeed");

    assert!(matches!(
        decision,
        DataLayerM5AnomalyDecision::Anomalous {
            reason_code: DATA_LAYER_M5_ANOMALY_THRESHOLD_EXCEEDED_REASON_CODE,
            ..
        }
    ));

    let normal = registry
        .evaluate_agent_anomaly(DataLayerM5AnomalyEvaluationInput {
            owner_did: "kamn:did:owner:alpha".to_owned(),
            agent_did: "kamn:did:agent:alpha".to_owned(),
            candidate_vector: vec![1.0, 0.0, 0.0],
            lookback_window: Some(3),
            anomaly_distance_threshold: 0.9,
        })
        .expect("normal anomaly evaluation should succeed");
    assert!(matches!(
        normal,
        DataLayerM5AnomalyDecision::Normal {
            reason_code: DATA_LAYER_M5_ANOMALY_WITHIN_THRESHOLD_REASON_CODE,
            ..
        }
    ));
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
