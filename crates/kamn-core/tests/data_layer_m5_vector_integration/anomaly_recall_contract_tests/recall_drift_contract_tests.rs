use kamn_core::{
    DataLayerM5EmbeddingPrivacyMode, DataLayerM5EmbeddingRegistry, DataLayerM5RecallDriftDecision,
    DataLayerM5RecallDriftEvaluationInput, DataLayerM5VectorIntegrationError,
    DATA_LAYER_M5_RECALL_DRIFT_DEGRADED_REASON_CODE, DATA_LAYER_M5_RECALL_DRIFT_STABLE_REASON_CODE,
};

use super::super::support::vector_input;

#[test]
fn spec_c06_recall_drift_is_stable_when_recall_and_rank_shift_are_within_thresholds() {
    let registry = make_recall_registry(&[
        ("embed-m5-r1", "msg-m5-r1", vec![1.0, 0.0, 0.0]),
        ("embed-m5-r2", "msg-m5-r2", vec![0.9, 0.1, 0.0]),
    ]);

    let report = registry
        .evaluate_recall_drift(DataLayerM5RecallDriftEvaluationInput {
            owner_did: "kamn:did:owner:alpha".to_owned(),
            query_vector: vec![1.0, 0.0, 0.0],
            baseline_top_k_embedding_ids: vec!["embed-m5-r1".to_owned(), "embed-m5-r2".to_owned()],
            min_recall_at_k: 1.0,
            max_allowed_rank_shift: 0,
        })
        .expect("recall drift should succeed");

    assert_eq!(report.decision, DataLayerM5RecallDriftDecision::Stable);
    assert_eq!(report.reason_code, DATA_LAYER_M5_RECALL_DRIFT_STABLE_REASON_CODE);
    assert_eq!(report.recall_at_k, 1.0);
    assert_eq!(report.max_observed_rank_shift, 0);
    assert!(report.missing_embedding_ids.is_empty());
}

#[test]
fn spec_c07_recall_drift_is_degraded_when_baseline_ids_are_missing() {
    let registry = make_recall_registry(&[("embed-m5-r3", "msg-m5-r3", vec![1.0, 0.0, 0.0])]);

    let report = registry
        .evaluate_recall_drift(DataLayerM5RecallDriftEvaluationInput {
            owner_did: "kamn:did:owner:alpha".to_owned(),
            query_vector: vec![1.0, 0.0, 0.0],
            baseline_top_k_embedding_ids: vec!["embed-m5-r3".to_owned(), "embed-m5-r4-missing".to_owned()],
            min_recall_at_k: 1.0,
            max_allowed_rank_shift: 0,
        })
        .expect("recall drift should succeed");

    assert_eq!(report.decision, DataLayerM5RecallDriftDecision::Degraded);
    assert_eq!(report.reason_code, DATA_LAYER_M5_RECALL_DRIFT_DEGRADED_REASON_CODE);
    assert_eq!(report.missing_embedding_ids, vec!["embed-m5-r4-missing".to_owned()]);
}

#[test]
fn spec_c08_recall_drift_is_degraded_when_rank_shift_exceeds_threshold() {
    let registry = make_recall_registry(&[
        ("embed-m5-r5", "msg-m5-r5", vec![1.0, 0.0, 0.0]),
        ("embed-m5-r6", "msg-m5-r6", vec![0.9, 0.1, 0.0]),
    ]);

    let report = registry
        .evaluate_recall_drift(DataLayerM5RecallDriftEvaluationInput {
            owner_did: "kamn:did:owner:alpha".to_owned(),
            query_vector: vec![1.0, 0.0, 0.0],
            baseline_top_k_embedding_ids: vec!["embed-m5-r6".to_owned(), "embed-m5-r5".to_owned()],
            min_recall_at_k: 1.0,
            max_allowed_rank_shift: 0,
        })
        .expect("recall drift should succeed");

    assert_eq!(report.decision, DataLayerM5RecallDriftDecision::Degraded);
    assert_eq!(report.reason_code, DATA_LAYER_M5_RECALL_DRIFT_DEGRADED_REASON_CODE);
    assert_eq!(report.max_observed_rank_shift, 1);
}

#[test]
fn spec_c09_recall_drift_rejects_invalid_threshold_and_empty_baseline() {
    let registry =
        DataLayerM5EmbeddingRegistry::new(DataLayerM5EmbeddingPrivacyMode::ServerSidePlaintextOptIn);

    assert_empty_baseline_rejected(&registry);
    assert_invalid_threshold_rejected(&registry);
}

fn make_recall_registry(entries: &[(&str, &str, Vec<f32>)]) -> DataLayerM5EmbeddingRegistry {
    let mut registry =
        DataLayerM5EmbeddingRegistry::new(DataLayerM5EmbeddingPrivacyMode::ServerSidePlaintextOptIn);
    for (embedding_id, message_id, vector) in entries {
        registry
            .append(vector_input(
                embedding_id,
                message_id,
                "kamn:did:owner:alpha",
                "kamn:did:agent:alpha",
                Some(vector.clone()),
            ))
            .expect("recall append should succeed");
    }
    registry
}

fn assert_empty_baseline_rejected(registry: &DataLayerM5EmbeddingRegistry) {
    let empty_baseline = registry.evaluate_recall_drift(DataLayerM5RecallDriftEvaluationInput {
        owner_did: "kamn:did:owner:alpha".to_owned(),
        query_vector: vec![1.0, 0.0, 0.0],
        baseline_top_k_embedding_ids: Vec::new(),
        min_recall_at_k: 0.8,
        max_allowed_rank_shift: 1,
    });
    assert!(matches!(
        empty_baseline,
        Err(DataLayerM5VectorIntegrationError::EmptyField("baseline_top_k_embedding_ids"))
    ));
}

fn assert_invalid_threshold_rejected(registry: &DataLayerM5EmbeddingRegistry) {
    let invalid_threshold = registry.evaluate_recall_drift(DataLayerM5RecallDriftEvaluationInput {
        owner_did: "kamn:did:owner:alpha".to_owned(),
        query_vector: vec![1.0, 0.0, 0.0],
        baseline_top_k_embedding_ids: vec!["embed-m5-r1".to_owned()],
        min_recall_at_k: 1.5,
        max_allowed_rank_shift: 1,
    });
    assert!(matches!(
        invalid_threshold,
        Err(DataLayerM5VectorIntegrationError::InvalidVectorValue("min_recall_at_k"))
    ));
}
