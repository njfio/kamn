use crate::support::{
    data_layer_pg_project_m7_timescale_ingest_operation,
    data_layer_pg_project_m7_timescale_owner_rollup_query_operation, fixture_m7_telemetry_record,
    DataLayerM7BillingQuery, DataLayerPgM7TimescaleConfig,
    DataLayerPgM7TimescaleOwnerRollupRequest, DataLayerPgOperationKind,
    DataLayerPgRepositoryBridgeError, DATA_LAYER_PG_TIMESCALE_EXTENSION_UNAVAILABLE_REASON_CODE,
    DATA_LAYER_PG_TIMESCALE_INVALID_BUCKET_WINDOW_REASON_CODE,
};

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
    assert_eq!(ingest_descriptor.kind, DataLayerPgOperationKind::InsertTelemetryPoint);
    assert!(ingest_descriptor.sql.starts_with("INSERT INTO telemetry_points"));
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
    assert_eq!(rollup_descriptor.kind, DataLayerPgOperationKind::QueryTelemetryOwnerRollup);
    assert!(rollup_descriptor.sql.contains("time_bucket(INTERVAL '1 day'"));
    assert_eq!(rollup_descriptor.bind_markers, vec!["owner_did", "limit"]);
}

#[test]
fn spec_c10_m7_timescale_projection_fails_closed_for_extension_and_invalid_bucket_window() {
    let record = fixture_m7_telemetry_record();
    let extension_error = data_layer_pg_project_m7_timescale_ingest_operation(
        &record,
        "kamn:did:agent:agent-1",
        DataLayerPgM7TimescaleConfig::new(false, "telemetry_points")
            .expect("disabled Timescale config should still construct"),
    )
    .expect_err("disabled Timescale extension should fail closed");
    match extension_error {
        DataLayerPgRepositoryBridgeError::TimescaleExtensionUnavailable { reason_code } => {
            assert_eq!(reason_code, DATA_LAYER_PG_TIMESCALE_EXTENSION_UNAVAILABLE_REASON_CODE);
        }
        other => panic!("unexpected Timescale extension error variant: {other:?}"),
    }

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
        DataLayerPgM7TimescaleConfig::new(true, "telemetry_points")
            .expect("enabled Timescale config should construct"),
    )
    .expect_err("invalid bucket window should fail closed");
    match invalid_window_error {
        DataLayerPgRepositoryBridgeError::InvalidTimescaleBucketWindow {
            reason_code,
            bucket_window_seconds,
        } => {
            assert_eq!(reason_code, DATA_LAYER_PG_TIMESCALE_INVALID_BUCKET_WINDOW_REASON_CODE);
            assert_eq!(bucket_window_seconds, 777);
        }
        other => panic!("unexpected invalid-bucket-window error variant: {other:?}"),
    }
}
