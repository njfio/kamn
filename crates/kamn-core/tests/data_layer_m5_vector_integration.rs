use kamn_core::{
    ContentRetentionClass, DataLayerM5AnomalyDecision, DataLayerM5AnomalyEvaluationInput,
    DataLayerM5EmbeddingPrivacyMode, DataLayerM5EmbeddingRecordInput, DataLayerM5EmbeddingRegistry,
    DataLayerM5RecallDriftDecision, DataLayerM5RecallDriftEvaluationInput,
    DataLayerM5RetentionDueCandidate, DataLayerM5SemanticQuery, DataLayerM5VectorIntegrationError,
    DATA_LAYER_M5_ANOMALY_THRESHOLD_EXCEEDED_REASON_CODE,
    DATA_LAYER_M5_ANOMALY_WITHIN_THRESHOLD_REASON_CODE,
    DATA_LAYER_M5_INVALID_AGENT_DID_REASON_CODE,
    DATA_LAYER_M5_OWNER_SIDE_QUERY_REQUIRES_LOCAL_INDEX_REASON_CODE,
    DATA_LAYER_M5_RECALL_DRIFT_DEGRADED_REASON_CODE, DATA_LAYER_M5_RECALL_DRIFT_STABLE_REASON_CODE,
    DATA_LAYER_M5_RETENTION_DUE_REASON_CODE,
};

fn vector_input(
    embedding_id: &str,
    message_id: &str,
    owner_did: &str,
    agent_did: &str,
    vector_plaintext: Option<Vec<f32>>,
) -> DataLayerM5EmbeddingRecordInput {
    DataLayerM5EmbeddingRecordInput {
        embedding_id: embedding_id.to_owned(),
        message_id: message_id.to_owned(),
        owner_did: owner_did.to_owned(),
        agent_did: agent_did.to_owned(),
        retention_class: ContentRetentionClass::Standard,
        model_id: "text-embedding-3-large".to_owned(),
        vector_encrypted: vec![0xde, 0xad, 0xbe, 0xef],
        vector_plaintext,
        created_at_epoch_seconds: 1_708_300_000,
    }
}

#[test]
fn spec_c01_embedding_registry_append_is_deterministic_and_hash_chained() {
    let mut registry_a = DataLayerM5EmbeddingRegistry::new(
        DataLayerM5EmbeddingPrivacyMode::ServerSidePlaintextOptIn,
    );
    let mut registry_b = DataLayerM5EmbeddingRegistry::new(
        DataLayerM5EmbeddingPrivacyMode::ServerSidePlaintextOptIn,
    );

    let input = vector_input(
        "embed-m5-1",
        "msg-m5-1",
        "kamn:did:owner:alpha",
        "kamn:did:agent:alpha",
        Some(vec![0.1, 0.2, 0.3]),
    );
    let record_a = registry_a
        .append(input.clone())
        .expect("append should succeed for registry A");
    let record_b = registry_b
        .append(input)
        .expect("append should succeed for registry B");

    assert_eq!(record_a.record_hash, record_b.record_hash);
    assert!(record_a.record_hash.starts_with("sha256:"));
    registry_a
        .verify_owner_integrity("kamn:did:owner:alpha")
        .expect("integrity check should pass");
}

#[test]
fn spec_c02_duplicate_embedding_id_is_rejected_fail_closed() {
    let mut registry = DataLayerM5EmbeddingRegistry::new(
        DataLayerM5EmbeddingPrivacyMode::ServerSidePlaintextOptIn,
    );
    let input = vector_input(
        "embed-m5-dup",
        "msg-m5-dup",
        "kamn:did:owner:alpha",
        "kamn:did:agent:alpha",
        Some(vec![0.3, 0.2, 0.1]),
    );

    registry
        .append(input.clone())
        .expect("first append should succeed");
    let duplicate = registry.append(input);
    assert!(matches!(
        duplicate,
        Err(DataLayerM5VectorIntegrationError::DuplicateEmbeddingId(_))
    ));
}

#[test]
fn spec_c03_semantic_query_is_owner_scoped_and_ranked_deterministically() {
    let mut registry = DataLayerM5EmbeddingRegistry::new(
        DataLayerM5EmbeddingPrivacyMode::ServerSidePlaintextOptIn,
    );
    registry
        .append(vector_input(
            "embed-m5-a",
            "msg-m5-a",
            "kamn:did:owner:alpha",
            "kamn:did:agent:alpha",
            Some(vec![1.0, 0.0, 0.0]),
        ))
        .expect("append a should succeed");
    registry
        .append(vector_input(
            "embed-m5-b",
            "msg-m5-b",
            "kamn:did:owner:alpha",
            "kamn:did:agent:alpha",
            Some(vec![0.8, 0.2, 0.0]),
        ))
        .expect("append b should succeed");
    registry
        .append(vector_input(
            "embed-m5-other-owner",
            "msg-m5-other-owner",
            "kamn:did:owner:beta",
            "kamn:did:agent:beta",
            Some(vec![1.0, 0.0, 0.0]),
        ))
        .expect("append other owner should succeed");

    let results = registry
        .semantic_query(DataLayerM5SemanticQuery {
            owner_did: "kamn:did:owner:alpha".to_owned(),
            query_vector: vec![1.0, 0.0, 0.0],
            limit: Some(2),
        })
        .expect("semantic query should succeed");

    assert_eq!(results.len(), 2);
    assert_eq!(results[0].message_id, "msg-m5-a");
    assert_eq!(results[1].message_id, "msg-m5-b");
}

#[test]
fn spec_c04_owner_side_encrypted_mode_rejects_server_side_semantic_query() {
    let mut registry =
        DataLayerM5EmbeddingRegistry::new(DataLayerM5EmbeddingPrivacyMode::OwnerSideEncrypted);
    registry
        .append(vector_input(
            "embed-m5-enc",
            "msg-m5-enc",
            "kamn:did:owner:alpha",
            "kamn:did:agent:alpha",
            None,
        ))
        .expect("owner-side encrypted append should succeed");

    let denied = registry.semantic_query(DataLayerM5SemanticQuery {
        owner_did: "kamn:did:owner:alpha".to_owned(),
        query_vector: vec![1.0, 0.0, 0.0],
        limit: Some(5),
    });
    assert!(matches!(
        denied,
        Err(
            DataLayerM5VectorIntegrationError::SemanticQueryUnavailable {
                reason_code: DATA_LAYER_M5_OWNER_SIDE_QUERY_REQUIRES_LOCAL_INDEX_REASON_CODE,
            }
        )
    ));
}

#[test]
fn spec_c05_anomaly_threshold_detection_uses_centroid_distance_rules() {
    let mut registry = DataLayerM5EmbeddingRegistry::new(
        DataLayerM5EmbeddingPrivacyMode::ServerSidePlaintextOptIn,
    );
    registry
        .append(vector_input(
            "embed-m5-h1",
            "msg-m5-h1",
            "kamn:did:owner:alpha",
            "kamn:did:agent:alpha",
            Some(vec![1.0, 0.0, 0.0]),
        ))
        .expect("append history 1 should succeed");
    registry
        .append(vector_input(
            "embed-m5-h2",
            "msg-m5-h2",
            "kamn:did:owner:alpha",
            "kamn:did:agent:alpha",
            Some(vec![0.95, 0.05, 0.0]),
        ))
        .expect("append history 2 should succeed");
    registry
        .append(vector_input(
            "embed-m5-h3",
            "msg-m5-h3",
            "kamn:did:owner:alpha",
            "kamn:did:agent:alpha",
            Some(vec![0.9, 0.1, 0.0]),
        ))
        .expect("append history 3 should succeed");

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

#[test]
fn spec_c06_recall_drift_is_stable_when_recall_and_rank_shift_are_within_thresholds() {
    let mut registry = DataLayerM5EmbeddingRegistry::new(
        DataLayerM5EmbeddingPrivacyMode::ServerSidePlaintextOptIn,
    );
    registry
        .append(vector_input(
            "embed-m5-r1",
            "msg-m5-r1",
            "kamn:did:owner:alpha",
            "kamn:did:agent:alpha",
            Some(vec![1.0, 0.0, 0.0]),
        ))
        .expect("append r1 should succeed");
    registry
        .append(vector_input(
            "embed-m5-r2",
            "msg-m5-r2",
            "kamn:did:owner:alpha",
            "kamn:did:agent:alpha",
            Some(vec![0.9, 0.1, 0.0]),
        ))
        .expect("append r2 should succeed");

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
    assert_eq!(
        report.reason_code,
        DATA_LAYER_M5_RECALL_DRIFT_STABLE_REASON_CODE
    );
    assert_eq!(report.recall_at_k, 1.0);
    assert_eq!(report.max_observed_rank_shift, 0);
    assert!(report.missing_embedding_ids.is_empty());
}

#[test]
fn spec_c07_recall_drift_is_degraded_when_baseline_ids_are_missing() {
    let mut registry = DataLayerM5EmbeddingRegistry::new(
        DataLayerM5EmbeddingPrivacyMode::ServerSidePlaintextOptIn,
    );
    registry
        .append(vector_input(
            "embed-m5-r3",
            "msg-m5-r3",
            "kamn:did:owner:alpha",
            "kamn:did:agent:alpha",
            Some(vec![1.0, 0.0, 0.0]),
        ))
        .expect("append r3 should succeed");

    let report = registry
        .evaluate_recall_drift(DataLayerM5RecallDriftEvaluationInput {
            owner_did: "kamn:did:owner:alpha".to_owned(),
            query_vector: vec![1.0, 0.0, 0.0],
            baseline_top_k_embedding_ids: vec![
                "embed-m5-r3".to_owned(),
                "embed-m5-r4-missing".to_owned(),
            ],
            min_recall_at_k: 1.0,
            max_allowed_rank_shift: 0,
        })
        .expect("recall drift should succeed");

    assert_eq!(report.decision, DataLayerM5RecallDriftDecision::Degraded);
    assert_eq!(
        report.reason_code,
        DATA_LAYER_M5_RECALL_DRIFT_DEGRADED_REASON_CODE
    );
    assert_eq!(
        report.missing_embedding_ids,
        vec!["embed-m5-r4-missing".to_owned()]
    );
}

#[test]
fn spec_c08_recall_drift_is_degraded_when_rank_shift_exceeds_threshold() {
    let mut registry = DataLayerM5EmbeddingRegistry::new(
        DataLayerM5EmbeddingPrivacyMode::ServerSidePlaintextOptIn,
    );
    registry
        .append(vector_input(
            "embed-m5-r5",
            "msg-m5-r5",
            "kamn:did:owner:alpha",
            "kamn:did:agent:alpha",
            Some(vec![1.0, 0.0, 0.0]),
        ))
        .expect("append r5 should succeed");
    registry
        .append(vector_input(
            "embed-m5-r6",
            "msg-m5-r6",
            "kamn:did:owner:alpha",
            "kamn:did:agent:alpha",
            Some(vec![0.9, 0.1, 0.0]),
        ))
        .expect("append r6 should succeed");

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
    assert_eq!(
        report.reason_code,
        DATA_LAYER_M5_RECALL_DRIFT_DEGRADED_REASON_CODE
    );
    assert_eq!(report.max_observed_rank_shift, 1);
}

#[test]
fn spec_c09_recall_drift_rejects_invalid_threshold_and_empty_baseline() {
    let registry = DataLayerM5EmbeddingRegistry::new(
        DataLayerM5EmbeddingPrivacyMode::ServerSidePlaintextOptIn,
    );

    let empty_baseline = registry.evaluate_recall_drift(DataLayerM5RecallDriftEvaluationInput {
        owner_did: "kamn:did:owner:alpha".to_owned(),
        query_vector: vec![1.0, 0.0, 0.0],
        baseline_top_k_embedding_ids: Vec::new(),
        min_recall_at_k: 0.8,
        max_allowed_rank_shift: 1,
    });
    assert!(matches!(
        empty_baseline,
        Err(DataLayerM5VectorIntegrationError::EmptyField(
            "baseline_top_k_embedding_ids"
        ))
    ));

    let invalid_threshold = registry.evaluate_recall_drift(DataLayerM5RecallDriftEvaluationInput {
        owner_did: "kamn:did:owner:alpha".to_owned(),
        query_vector: vec![1.0, 0.0, 0.0],
        baseline_top_k_embedding_ids: vec!["embed-m5-r1".to_owned()],
        min_recall_at_k: 1.5,
        max_allowed_rank_shift: 1,
    });
    assert!(matches!(
        invalid_threshold,
        Err(DataLayerM5VectorIntegrationError::InvalidVectorValue(
            "min_recall_at_k"
        ))
    ));
}

#[test]
fn spec_c10_agent_did_validation_uses_canonical_parser_and_fails_closed() {
    let mut registry = DataLayerM5EmbeddingRegistry::new(
        DataLayerM5EmbeddingPrivacyMode::ServerSidePlaintextOptIn,
    );
    let invalid_agent = registry.append(vector_input(
        "embed-m5-invalid-agent",
        "msg-m5-invalid-agent",
        "kamn:did:owner:alpha",
        "kamn:did:owner:not-an-agent",
        Some(vec![0.4, 0.4, 0.2]),
    ));
    assert!(matches!(
        invalid_agent,
        Err(DataLayerM5VectorIntegrationError::InvalidAgentDid {
            reason_code: DATA_LAYER_M5_INVALID_AGENT_DID_REASON_CODE,
            ..
        })
    ));
}

#[test]
fn spec_c11_retention_due_projection_aligns_with_content_lifecycle_windows() {
    let mut registry = DataLayerM5EmbeddingRegistry::new(
        DataLayerM5EmbeddingPrivacyMode::ServerSidePlaintextOptIn,
    );
    registry
        .append(DataLayerM5EmbeddingRecordInput {
            embedding_id: "embed-m5-retention-1".to_owned(),
            message_id: "msg-m5-retention-1".to_owned(),
            owner_did: "kamn:did:owner:alpha".to_owned(),
            agent_did: "kamn:did:agent:alpha".to_owned(),
            retention_class: ContentRetentionClass::ShortLived,
            model_id: "text-embedding-3-large".to_owned(),
            vector_encrypted: vec![0xde, 0xad, 0xbe, 0xef],
            vector_plaintext: Some(vec![0.3, 0.4, 0.3]),
            created_at_epoch_seconds: 1_708_300_000,
        })
        .expect("short-lived embedding should append");
    registry
        .append(DataLayerM5EmbeddingRecordInput {
            embedding_id: "embed-m5-retention-2".to_owned(),
            message_id: "msg-m5-retention-2".to_owned(),
            owner_did: "kamn:did:owner:alpha".to_owned(),
            agent_did: "kamn:did:agent:alpha".to_owned(),
            retention_class: ContentRetentionClass::Compliance,
            model_id: "text-embedding-3-large".to_owned(),
            vector_encrypted: vec![0xde, 0xad, 0xbe, 0xef],
            vector_plaintext: Some(vec![0.2, 0.5, 0.3]),
            created_at_epoch_seconds: 1_708_300_010,
        })
        .expect("compliance embedding should append");

    let due: Vec<DataLayerM5RetentionDueCandidate> = registry
        .retention_due_for_owner("kamn:did:owner:alpha", 1_708_500_000)
        .expect("retention due should succeed");
    assert_eq!(due.len(), 1);
    assert_eq!(due[0].embedding_id, "embed-m5-retention-1");
    assert_eq!(due[0].message_id, "msg-m5-retention-1");
    assert_eq!(due[0].reason_code, DATA_LAYER_M5_RETENTION_DUE_REASON_CODE);

    let invalid_now = registry.retention_due_for_owner("kamn:did:owner:alpha", 0);
    assert!(matches!(
        invalid_now,
        Err(DataLayerM5VectorIntegrationError::EmptyField(
            "now_epoch_seconds"
        ))
    ));
}
