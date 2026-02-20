use kamn_core::{
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

fn fixture_record() -> DataLayerM0EnvelopeRecord {
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

fn fixture_m5_embedding_record() -> DataLayerM5EmbeddingRecord {
    let mut registry = DataLayerM5EmbeddingRegistry::new(
        DataLayerM5EmbeddingPrivacyMode::ServerSidePlaintextOptIn,
    );
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

fn fixture_m6_edge_record() -> DataLayerM6GraphEdgeRecord {
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

fn fixture_m7_telemetry_record() -> DataLayerM7TelemetryPointRecord {
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

#[test]
fn spec_c01_insert_projection_is_deterministic() {
    let descriptor = data_layer_pg_project_insert_message_operation(
        &fixture_record(),
        "kamn:did:owner:owner-1",
        "kamn:did:agent:agent-1",
    )
    .expect("valid record and identities should project insert descriptor");

    assert_eq!(descriptor.kind, DataLayerPgOperationKind::InsertMessage);
    assert!(
        descriptor.sql.starts_with("INSERT INTO messages"),
        "insert descriptor should target messages table"
    );
    assert_eq!(
        descriptor.bind_markers,
        vec![
            "message_id",
            "owner_did",
            "sender_did",
            "recipient_did",
            "envelope_ciphertext",
            "envelope_nonce",
            "content_hash_sha256",
            "hash_chain_prev",
            "blind_indexes",
            "retention_class"
        ],
        "insert bind marker order should be stable"
    );
    assert_eq!(
        descriptor.session.requester_did, "kamn:did:agent:agent-1",
        "requester did should be projected into session metadata"
    );
}

#[test]
fn spec_c02_query_and_search_projection_include_session_context() {
    let lookup =
        data_layer_pg_project_select_message_by_id_operation("msg-1", "kamn:did:agent:agent-1")
            .expect("valid lookup request should project");
    assert_eq!(lookup.kind, DataLayerPgOperationKind::SelectMessageById);
    assert!(
        lookup.sql.contains("WHERE message_id = $1"),
        "lookup SQL should bind message_id first"
    );
    assert_eq!(lookup.session.setting_key, "kamn.requester_did");

    let search =
        data_layer_pg_project_blind_index_search_operation(DataLayerPgBlindIndexSearchRequest {
            requester_did: "kamn:did:agent:agent-1".to_owned(),
            owner_did: "kamn:did:owner:owner-1".to_owned(),
            index_key: "subject".to_owned(),
            index_value_hash: "sha256:token".to_owned(),
            limit: 25,
        })
        .expect("valid search request should project");
    assert_eq!(
        search.kind,
        DataLayerPgOperationKind::SearchMessagesByBlindIndex
    );
    assert!(
        search.sql.contains("blind_indexes ->> $2 = $3"),
        "search SQL should project blind-index key/value lookup"
    );
}

#[test]
fn spec_c03_default_rls_projection_is_deterministic() {
    let statements = data_layer_pg_project_default_rls_statements();
    assert!(
        statements.len() >= 4,
        "default RLS projection should include enable/drop/create statements"
    );
    assert_eq!(
        statements[0].table_name, "access_log",
        "statement ordering should be deterministic by table/policy"
    );
    assert!(
        statements
            .iter()
            .any(|entry| entry.sql.contains("ENABLE ROW LEVEL SECURITY")),
        "projection should include enable RLS statements"
    );
    assert!(
        statements
            .iter()
            .any(|entry| entry.sql.contains("CREATE POLICY")),
        "projection should include create policy statements"
    );
}

#[test]
fn spec_c04_invalid_requester_did_fails_closed() {
    let error =
        data_layer_pg_project_select_message_by_id_operation("msg-1", "kamn:did:agent:Agent-1")
            .expect_err("invalid requester did should fail closed");
    match error {
        DataLayerPgRepositoryBridgeError::InvalidRequesterDid {
            field, reason_code, ..
        } => {
            assert_eq!(field, "requester_did");
            assert_eq!(reason_code, DATA_LAYER_PG_INVALID_REQUESTER_DID_REASON_CODE);
        }
        other => panic!("unexpected error variant: {other:?}"),
    }
}

#[test]
fn spec_c05_m5_pgvector_projection_is_deterministic_for_insert_and_search() {
    let record = fixture_m5_embedding_record();
    let config = DataLayerPgM5PgvectorConfig::new(true, 3)
        .expect("pgvector config should construct deterministically");

    let insert_descriptor = data_layer_pg_project_m5_embedding_insert_operation(
        &record,
        "kamn:did:agent:agent-1",
        config,
    )
    .expect("valid m5 embedding should project pgvector insert descriptor");
    assert_eq!(
        insert_descriptor.kind,
        DataLayerPgOperationKind::InsertEmbeddingVector
    );
    assert!(
        insert_descriptor.sql.starts_with("INSERT INTO embeddings"),
        "insert projection should target embeddings table"
    );
    assert_eq!(
        insert_descriptor.bind_markers,
        vec![
            "embedding_id",
            "message_id",
            "owner_did",
            "agent_did",
            "model_id",
            "vector_plaintext",
            "vector_dimensions",
            "created_at_epoch_seconds",
        ]
    );

    let search_descriptor = data_layer_pg_project_m5_similarity_search_operation(
        DataLayerPgM5SimilaritySearchRequest {
            requester_did: "kamn:did:agent:agent-1".to_owned(),
            query: DataLayerM5SemanticQuery {
                owner_did: "kamn:did:owner:owner-1".to_owned(),
                query_vector: vec![0.1, 0.2, 0.3],
                limit: Some(10),
            },
        },
        config,
    )
    .expect("valid semantic query should project pgvector similarity descriptor");
    assert_eq!(
        search_descriptor.kind,
        DataLayerPgOperationKind::SearchEmbeddingVectors
    );
    assert!(
        search_descriptor
            .sql
            .contains("ORDER BY vector_plaintext <=> $2::vector"),
        "search projection should use pgvector cosine-distance operator ordering"
    );
    assert_eq!(
        search_descriptor.bind_markers,
        vec!["owner_did", "query_vector", "limit"]
    );
}

#[test]
fn spec_c06_m5_pgvector_projection_fails_closed_for_extension_and_dimension_mismatch() {
    let record = fixture_m5_embedding_record();
    let extension_disabled_config = DataLayerPgM5PgvectorConfig::new(false, 3)
        .expect("disabled pgvector config should still construct");

    let unavailable_error = data_layer_pg_project_m5_embedding_insert_operation(
        &record,
        "kamn:did:agent:agent-1",
        extension_disabled_config,
    )
    .expect_err("disabled pgvector extension should fail closed");
    match unavailable_error {
        DataLayerPgRepositoryBridgeError::PgvectorExtensionUnavailable { reason_code } => {
            assert_eq!(
                reason_code,
                DATA_LAYER_PG_PGVECTOR_EXTENSION_UNAVAILABLE_REASON_CODE
            );
        }
        other => panic!("unexpected extension error variant: {other:?}"),
    }

    let dimension_config = DataLayerPgM5PgvectorConfig::new(true, 4)
        .expect("enabled pgvector config should construct");
    let mismatch_error = data_layer_pg_project_m5_similarity_search_operation(
        DataLayerPgM5SimilaritySearchRequest {
            requester_did: "kamn:did:agent:agent-1".to_owned(),
            query: DataLayerM5SemanticQuery {
                owner_did: "kamn:did:owner:owner-1".to_owned(),
                query_vector: vec![0.1, 0.2, 0.3],
                limit: Some(10),
            },
        },
        dimension_config,
    )
    .expect_err("dimension mismatch should fail closed");
    match mismatch_error {
        DataLayerPgRepositoryBridgeError::PgvectorDimensionMismatch {
            reason_code,
            expected,
            found,
        } => {
            assert_eq!(
                reason_code,
                DATA_LAYER_PG_PGVECTOR_DIMENSION_MISMATCH_REASON_CODE
            );
            assert_eq!(expected, 4);
            assert_eq!(found, 3);
        }
        other => panic!("unexpected mismatch error variant: {other:?}"),
    }
}

#[test]
fn spec_c07_m6_age_projection_is_deterministic_for_edge_upsert_and_trust_query() {
    let edge = fixture_m6_edge_record();
    let config = DataLayerPgM6AgeConfig::new(true, "kamn_graph")
        .expect("AGE config should construct deterministically");

    let upsert_descriptor = data_layer_pg_project_m6_age_edge_upsert_operation(
        &edge,
        "kamn:did:agent:agent-1",
        config.clone(),
    )
    .expect("valid m6 edge should project AGE upsert descriptor");
    assert_eq!(
        upsert_descriptor.kind,
        DataLayerPgOperationKind::UpsertGraphEdge
    );
    assert!(
        upsert_descriptor.sql.contains("SELECT * FROM cypher"),
        "upsert projection should target AGE cypher entrypoint"
    );
    assert_eq!(
        upsert_descriptor.bind_markers,
        vec![
            "owner_did",
            "edge_id",
            "relation_marker",
            "from_node_id",
            "to_node_id",
            "weight",
            "observed_at_epoch_seconds",
        ]
    );

    let trust_query_descriptor = data_layer_pg_project_m6_age_trust_query_operation(
        DataLayerPgM6AgeTrustQueryRequest {
            requester_did: "kamn:did:agent:agent-1".to_owned(),
            query: DataLayerM6TrustPropagationQuery {
                requester_owner_did: "kamn:did:owner:owner-1".to_owned(),
                owner_did: "kamn:did:owner:owner-1".to_owned(),
                source_agent_node_id: "agent-a".to_owned(),
                max_depth: 2,
                attenuation_factor: 0.85,
                limit: Some(10),
            },
        },
        config,
    )
    .expect("valid trust query should project AGE query descriptor");
    assert_eq!(
        trust_query_descriptor.kind,
        DataLayerPgOperationKind::QueryGraphTrustPropagation
    );
    assert!(
        trust_query_descriptor
            .sql
            .contains("MATCH (source:Agent {node_id: $2, owner_did: $1})"),
        "trust query descriptor should project deterministic owner-scoped source-node match"
    );
    assert_eq!(
        trust_query_descriptor.bind_markers,
        vec!["owner_did", "source_agent_node_id", "max_depth", "limit"]
    );
}

#[test]
fn spec_c08_m6_age_projection_fails_closed_for_extension_and_relation_mismatch() {
    let edge = fixture_m6_edge_record();

    let disabled_config = DataLayerPgM6AgeConfig::new(false, "kamn_graph")
        .expect("disabled AGE config should construct");
    let extension_error = data_layer_pg_project_m6_age_edge_upsert_operation(
        &edge,
        "kamn:did:agent:agent-1",
        disabled_config,
    )
    .expect_err("disabled AGE extension should fail closed");
    match extension_error {
        DataLayerPgRepositoryBridgeError::AgeExtensionUnavailable { reason_code } => {
            assert_eq!(
                reason_code,
                DATA_LAYER_PG_AGE_EXTENSION_UNAVAILABLE_REASON_CODE
            );
        }
        other => panic!("unexpected extension error variant: {other:?}"),
    }

    let enabled_config = DataLayerPgM6AgeConfig::new(true, "kamn_graph")
        .expect("enabled AGE config should construct");
    let unsupported_relation_error = data_layer_pg_project_m6_age_edge_upsert_operation(
        &DataLayerM6GraphEdgeRecord {
            relation: DataLayerM6GraphEdgeRelation::Messaged,
            ..edge
        },
        "kamn:did:agent:agent-1",
        enabled_config,
    )
    .expect_err("unsupported relation projection should fail closed");
    match unsupported_relation_error {
        DataLayerPgRepositoryBridgeError::AgeUnsupportedRelation { reason_code, .. } => {
            assert_eq!(
                reason_code,
                DATA_LAYER_PG_AGE_RELATION_UNSUPPORTED_REASON_CODE
            );
        }
        other => panic!("unexpected relation error variant: {other:?}"),
    }
}

#[test]
fn spec_c09_m7_timescale_projection_is_deterministic_for_ingest_and_rollup() {
    let record = fixture_m7_telemetry_record();
    let config = DataLayerPgM7TimescaleConfig::new(true, "telemetry_points")
        .expect("Timescale config should construct deterministically");

    let ingest_descriptor = data_layer_pg_project_m7_timescale_ingest_operation(
        &record,
        "kamn:did:agent:agent-1",
        config.clone(),
    )
    .expect("valid telemetry record should project Timescale ingest descriptor");
    assert_eq!(
        ingest_descriptor.kind,
        DataLayerPgOperationKind::InsertTelemetryPoint
    );
    assert!(
        ingest_descriptor
            .sql
            .starts_with("INSERT INTO telemetry_points"),
        "Timescale ingest descriptor should target telemetry_points table"
    );
    assert_eq!(
        ingest_descriptor.bind_markers,
        vec![
            "owner_did",
            "agent_did",
            "timestamp_epoch_seconds",
            "bucket_hour_epoch_seconds",
            "bucket_day_epoch_seconds",
            "message_count",
            "bytes_stored",
            "query_count",
            "embedding_count",
            "embedding_anomaly_count",
            "ingress_latency_ms_p95",
            "egress_latency_ms_p95",
            "active_sessions",
            "sequence",
        ]
    );

    let rollup_descriptor = data_layer_pg_project_m7_timescale_owner_rollup_query_operation(
        DataLayerPgM7TimescaleOwnerRollupRequest {
            requester_did: "kamn:did:agent:agent-1".to_owned(),
            query: DataLayerM7BillingQuery {
                requester_owner_did: "kamn:did:owner:owner-1".to_owned(),
                owner_did: "kamn:did:owner:owner-1".to_owned(),
            },
            bucket_window_seconds: 86_400,
            limit: Some(30),
        },
        config,
    )
    .expect("valid billing rollup request should project Timescale rollup descriptor");
    assert_eq!(
        rollup_descriptor.kind,
        DataLayerPgOperationKind::QueryTelemetryOwnerRollup
    );
    assert!(
        rollup_descriptor
            .sql
            .contains("time_bucket(INTERVAL '1 day'"),
        "daily rollup descriptor should use deterministic 1-day time_bucket interval"
    );
    assert_eq!(rollup_descriptor.bind_markers, vec!["owner_did", "limit"]);
}

#[test]
fn spec_c10_m7_timescale_projection_fails_closed_for_extension_and_invalid_bucket_window() {
    let record = fixture_m7_telemetry_record();
    let disabled_config = DataLayerPgM7TimescaleConfig::new(false, "telemetry_points")
        .expect("disabled Timescale config should still construct");

    let extension_error = data_layer_pg_project_m7_timescale_ingest_operation(
        &record,
        "kamn:did:agent:agent-1",
        disabled_config,
    )
    .expect_err("disabled Timescale extension should fail closed");
    match extension_error {
        DataLayerPgRepositoryBridgeError::TimescaleExtensionUnavailable { reason_code } => {
            assert_eq!(
                reason_code,
                DATA_LAYER_PG_TIMESCALE_EXTENSION_UNAVAILABLE_REASON_CODE
            );
        }
        other => panic!("unexpected Timescale extension error variant: {other:?}"),
    }

    let enabled_config = DataLayerPgM7TimescaleConfig::new(true, "telemetry_points")
        .expect("enabled Timescale config should construct");
    let invalid_window_error = data_layer_pg_project_m7_timescale_owner_rollup_query_operation(
        DataLayerPgM7TimescaleOwnerRollupRequest {
            requester_did: "kamn:did:agent:agent-1".to_owned(),
            query: DataLayerM7BillingQuery {
                requester_owner_did: "kamn:did:owner:owner-1".to_owned(),
                owner_did: "kamn:did:owner:owner-1".to_owned(),
            },
            bucket_window_seconds: 777,
            limit: Some(30),
        },
        enabled_config,
    )
    .expect_err("invalid bucket window should fail closed");
    match invalid_window_error {
        DataLayerPgRepositoryBridgeError::InvalidTimescaleBucketWindow {
            reason_code,
            bucket_window_seconds,
        } => {
            assert_eq!(
                reason_code,
                DATA_LAYER_PG_TIMESCALE_INVALID_BUCKET_WINDOW_REASON_CODE
            );
            assert_eq!(bucket_window_seconds, 777);
        }
        other => panic!("unexpected invalid-bucket-window error variant: {other:?}"),
    }
}
