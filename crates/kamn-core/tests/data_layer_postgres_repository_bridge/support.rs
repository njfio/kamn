pub(crate) use kamn_core::{
    data_layer_pg_project_blind_index_search_operation,
    data_layer_pg_project_default_rls_statements, data_layer_pg_project_insert_message_operation,
    data_layer_pg_project_m5_embedding_insert_operation,
    data_layer_pg_project_m5_similarity_search_operation,
    data_layer_pg_project_m6_age_edge_upsert_operation,
    data_layer_pg_project_m6_age_trust_query_operation,
    data_layer_pg_project_m7_timescale_ingest_operation,
    data_layer_pg_project_m7_timescale_owner_rollup_query_operation,
    data_layer_pg_project_select_message_by_id_operation, ContentRetentionClass,
    DataLayerM0EnvelopeRecord, DataLayerM0WrappedKey, DataLayerM5EmbeddingPrivacyMode,
    DataLayerM5EmbeddingRecord, DataLayerM5EmbeddingRecordInput, DataLayerM5EmbeddingRegistry,
    DataLayerM5SemanticQuery, DataLayerM6GraphEdgeInput, DataLayerM6GraphEdgeRecord,
    DataLayerM6GraphEdgeRelation, DataLayerM6GraphNodeInput, DataLayerM6GraphNodeKind,
    DataLayerM6GraphRegistry, DataLayerM6TrustPropagationQuery, DataLayerM7BillingQuery,
    DataLayerM7TelemetryPointInput, DataLayerM7TelemetryPointRecord, DataLayerM7TelemetryRegistry,
    DataLayerPgBlindIndexSearchRequest, DataLayerPgM5PgvectorConfig,
    DataLayerPgM5SimilaritySearchRequest, DataLayerPgM6AgeConfig,
    DataLayerPgM6AgeTrustQueryRequest, DataLayerPgM7TimescaleConfig,
    DataLayerPgM7TimescaleOwnerRollupRequest, DataLayerPgOperationKind,
    DataLayerPgRepositoryBridgeError, DATA_LAYER_PG_AGE_EXTENSION_UNAVAILABLE_REASON_CODE,
    DATA_LAYER_PG_AGE_RELATION_UNSUPPORTED_REASON_CODE,
    DATA_LAYER_PG_INVALID_REQUESTER_DID_REASON_CODE,
    DATA_LAYER_PG_PGVECTOR_DIMENSION_MISMATCH_REASON_CODE,
    DATA_LAYER_PG_PGVECTOR_EXTENSION_UNAVAILABLE_REASON_CODE,
    DATA_LAYER_PG_TIMESCALE_EXTENSION_UNAVAILABLE_REASON_CODE,
    DATA_LAYER_PG_TIMESCALE_INVALID_BUCKET_WINDOW_REASON_CODE,
};

pub(crate) fn fixture_record() -> DataLayerM0EnvelopeRecord {
    DataLayerM0EnvelopeRecord {
        message_id: "msg-1".to_owned(),
        content_hash: "sha256:abc123".to_owned(),
        hash_chain_prev: "sha256:genesis".to_owned(),
        sender_did: "kamn:did:agent:sender-1".to_owned(),
        recipient_dids: vec![
            "kamn:did:agent:recipient-1".to_owned(),
            "kamn:did:agent:recipient-2".to_owned(),
        ],
        message_type: "direct".to_owned(),
        envelope_ciphertext: "ciphertext-b64".to_owned(),
        envelope_nonce: 42,
        envelope_aad_hash: "sha256:aad".to_owned(),
        wrapped_keys: vec![DataLayerM0WrappedKey {
            did: "kamn:did:agent:recipient-1".to_owned(),
            wrapped_cek: "wrapped-cek".to_owned(),
        }],
        compression_codec: "zstd".to_owned(),
        compression_dict_id: Some(7),
        content_size_bytes: 2048,
        compressed_size_bytes: 512,
    }
}

pub(crate) fn fixture_m5_embedding_record() -> DataLayerM5EmbeddingRecord {
    let mut registry =
        DataLayerM5EmbeddingRegistry::new(DataLayerM5EmbeddingPrivacyMode::ServerSidePlaintextOptIn);
    registry
        .append(DataLayerM5EmbeddingRecordInput {
            embedding_id: "embed-pg-1".to_owned(),
            message_id: "msg-pg-1".to_owned(),
            owner_did: "kamn:did:owner:owner-1".to_owned(),
            agent_did: "kamn:did:agent:agent-1".to_owned(),
            retention_class: ContentRetentionClass::Standard,
            model_id: "text-embedding-3-large".to_owned(),
            vector_encrypted: vec![0xde, 0xad, 0xbe, 0xef],
            vector_plaintext: Some(vec![0.1, 0.2, 0.3]),
            created_at_epoch_seconds: 1_900_000_000,
        })
        .expect("fixture embedding append should succeed")
}

pub(crate) fn fixture_m6_edge_record() -> DataLayerM6GraphEdgeRecord {
    let mut registry = DataLayerM6GraphRegistry::new();
    registry
        .register_node(DataLayerM6GraphNodeInput {
            owner_did: "kamn:did:owner:owner-1".to_owned(),
            node_id: "agent-a".to_owned(),
            kind: DataLayerM6GraphNodeKind::Agent,
            label: "agent-a".to_owned(),
        })
        .expect("source node should register");
    registry
        .register_node(DataLayerM6GraphNodeInput {
            owner_did: "kamn:did:owner:owner-1".to_owned(),
            node_id: "agent-b".to_owned(),
            kind: DataLayerM6GraphNodeKind::Agent,
            label: "agent-b".to_owned(),
        })
        .expect("target node should register");
    registry
        .register_edge(DataLayerM6GraphEdgeInput {
            owner_did: "kamn:did:owner:owner-1".to_owned(),
            edge_id: "edge-age-1".to_owned(),
            relation: DataLayerM6GraphEdgeRelation::Trusts,
            from_node_id: "agent-a".to_owned(),
            to_node_id: "agent-b".to_owned(),
            weight: 0.85,
            observed_at_epoch_seconds: 1_900_000_000,
        })
        .expect("trust edge should register")
}

pub(crate) fn fixture_m7_telemetry_record() -> DataLayerM7TelemetryPointRecord {
    let mut registry = DataLayerM7TelemetryRegistry::new();
    registry
        .ingest_point(DataLayerM7TelemetryPointInput {
            owner_did: "kamn:did:owner:owner-1".to_owned(),
            agent_did: "kamn:did:agent:agent-1".to_owned(),
            timestamp_epoch_seconds: 1_900_000_000,
            message_count: 12,
            bytes_stored: 2048,
            query_count: 7,
            embedding_count: 3,
            embedding_anomaly_count: 0,
            ingress_latency_ms_p95: 45,
            egress_latency_ms_p95: 50,
            active_sessions: 2,
        })
        .expect("fixture telemetry point should ingest")
}
