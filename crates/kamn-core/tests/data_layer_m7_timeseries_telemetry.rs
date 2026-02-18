use kamn_core::{
    DataLayerM7BillingQuery, DataLayerM7BillingReconciliationDecision,
    DataLayerM7BillingReconciliationInput, DataLayerM7TelemetryPointInput,
    DataLayerM7TelemetryRegistry, DataLayerM7TelemetryScopeQuery, DataLayerM7TimeseriesError,
    DATA_LAYER_M7_BILLING_RECONCILIATION_MATCH_REASON_CODE,
    DATA_LAYER_M7_BILLING_RECONCILIATION_MISMATCH_REASON_CODE,
    DATA_LAYER_M7_OWNER_SCOPE_DENIED_REASON_CODE,
};

fn telemetry_point(
    owner_did: &str,
    agent_did: &str,
    timestamp_epoch_seconds: u64,
    message_count: u64,
    bytes_stored: u64,
    query_count: u64,
    embedding_count: u64,
) -> DataLayerM7TelemetryPointInput {
    DataLayerM7TelemetryPointInput {
        owner_did: owner_did.to_owned(),
        agent_did: agent_did.to_owned(),
        timestamp_epoch_seconds,
        message_count,
        bytes_stored,
        query_count,
        embedding_count,
        embedding_anomaly_count: 0,
        ingress_latency_ms_p95: 120,
        egress_latency_ms_p95: 140,
        active_sessions: 3,
    }
}

#[test]
fn spec_c01_telemetry_ingest_indexes_points_into_deterministic_hourly_and_daily_buckets() {
    let mut registry = DataLayerM7TelemetryRegistry::new();
    registry
        .ingest_point(telemetry_point(
            "kamn:did:owner:alpha",
            "kamn:did:agent:alpha-1",
            1_708_560_100,
            5,
            1_000,
            4,
            2,
        ))
        .expect("first telemetry point should ingest");
    registry
        .ingest_point(telemetry_point(
            "kamn:did:owner:alpha",
            "kamn:did:agent:alpha-1",
            1_708_560_900,
            7,
            2_000,
            3,
            1,
        ))
        .expect("second telemetry point should ingest");

    let hourly = registry
        .aggregate_agent_hourly(DataLayerM7TelemetryScopeQuery {
            requester_owner_did: "kamn:did:owner:alpha".to_owned(),
            owner_did: "kamn:did:owner:alpha".to_owned(),
            agent_did: "kamn:did:agent:alpha-1".to_owned(),
        })
        .expect("hourly aggregation should succeed");
    let daily = registry
        .aggregate_agent_daily(DataLayerM7TelemetryScopeQuery {
            requester_owner_did: "kamn:did:owner:alpha".to_owned(),
            owner_did: "kamn:did:owner:alpha".to_owned(),
            agent_did: "kamn:did:agent:alpha-1".to_owned(),
        })
        .expect("daily aggregation should succeed");

    assert_eq!(hourly.len(), 1);
    assert_eq!(daily.len(), 1);
    assert_eq!(hourly[0].message_count_total, 12);
    assert_eq!(daily[0].message_count_total, 12);
}

#[test]
fn spec_c02_invalid_scope_inputs_fail_closed() {
    let mut registry = DataLayerM7TelemetryRegistry::new();
    let invalid = registry.ingest_point(telemetry_point(
        "invalid-owner",
        "kamn:did:agent:alpha-1",
        1_708_560_100,
        1,
        100,
        1,
        1,
    ));
    assert!(matches!(
        invalid,
        Err(DataLayerM7TimeseriesError::InvalidDid(_))
    ));
}

#[test]
fn spec_c03_hourly_daily_and_network_rollups_are_deterministic() {
    let mut registry = DataLayerM7TelemetryRegistry::new();
    registry
        .ingest_point(telemetry_point(
            "kamn:did:owner:alpha",
            "kamn:did:agent:alpha-1",
            1_708_560_100,
            10,
            1_500,
            6,
            3,
        ))
        .expect("alpha point should ingest");
    registry
        .ingest_point(telemetry_point(
            "kamn:did:owner:alpha",
            "kamn:did:agent:alpha-2",
            1_708_560_400,
            8,
            800,
            4,
            2,
        ))
        .expect("alpha second agent point should ingest");
    registry
        .ingest_point(telemetry_point(
            "kamn:did:owner:beta",
            "kamn:did:agent:beta-1",
            1_708_560_500,
            9,
            1_100,
            5,
            2,
        ))
        .expect("beta point should ingest");

    let network = registry
        .aggregate_network_hourly()
        .expect("network hourly aggregation should succeed");
    assert_eq!(network.len(), 1);
    assert_eq!(network[0].message_count_total, 27);
    assert_eq!(network[0].active_agent_count, 3);
}

#[test]
fn spec_c04_owner_billing_daily_projection_is_deterministic_and_complete() {
    let mut registry = DataLayerM7TelemetryRegistry::new();
    registry
        .ingest_point(telemetry_point(
            "kamn:did:owner:alpha",
            "kamn:did:agent:alpha-1",
            1_708_560_100,
            12,
            5_000,
            7,
            4,
        ))
        .expect("telemetry point should ingest");
    registry
        .ingest_point(telemetry_point(
            "kamn:did:owner:alpha",
            "kamn:did:agent:alpha-2",
            1_708_561_200,
            8,
            3_000,
            5,
            3,
        ))
        .expect("second telemetry point should ingest");

    let billing = registry
        .project_owner_billing_daily(DataLayerM7BillingQuery {
            requester_owner_did: "kamn:did:owner:alpha".to_owned(),
            owner_did: "kamn:did:owner:alpha".to_owned(),
        })
        .expect("billing projection should succeed");
    assert_eq!(billing.len(), 1);
    assert_eq!(billing[0].messages_stored_total, 20);
    assert_eq!(billing[0].bytes_stored_total, 8_000);
    assert_eq!(billing[0].queries_executed_total, 12);
    assert_eq!(billing[0].embeddings_generated_total, 7);
}

#[test]
fn spec_c05_cross_owner_billing_query_is_denied_fail_closed() {
    let mut registry = DataLayerM7TelemetryRegistry::new();
    registry
        .ingest_point(telemetry_point(
            "kamn:did:owner:alpha",
            "kamn:did:agent:alpha-1",
            1_708_560_100,
            5,
            1_000,
            2,
            1,
        ))
        .expect("telemetry point should ingest");

    let denied = registry.project_owner_billing_daily(DataLayerM7BillingQuery {
        requester_owner_did: "kamn:did:owner:intruder".to_owned(),
        owner_did: "kamn:did:owner:alpha".to_owned(),
    });
    assert!(matches!(
        denied,
        Err(DataLayerM7TimeseriesError::OwnerScopeViolation {
            reason_code: DATA_LAYER_M7_OWNER_SCOPE_DENIED_REASON_CODE,
        })
    ));
}

#[test]
fn spec_c06_billing_reconciliation_matches_projected_daily_totals() {
    let mut registry = DataLayerM7TelemetryRegistry::new();
    registry
        .ingest_point(telemetry_point(
            "kamn:did:owner:alpha",
            "kamn:did:agent:alpha-1",
            1_708_560_100,
            10,
            2_000,
            5,
            3,
        ))
        .expect("telemetry point should ingest");

    let projection = registry
        .project_owner_billing_daily(DataLayerM7BillingQuery {
            requester_owner_did: "kamn:did:owner:alpha".to_owned(),
            owner_did: "kamn:did:owner:alpha".to_owned(),
        })
        .expect("billing projection should succeed");
    let day = projection.first().expect("projection row should exist");

    let reconciliation = registry
        .reconcile_owner_billing_daily(DataLayerM7BillingReconciliationInput {
            requester_owner_did: "kamn:did:owner:alpha".to_owned(),
            owner_did: "kamn:did:owner:alpha".to_owned(),
            bucket_day_epoch_seconds: day.bucket_day_epoch_seconds,
            messages_stored_total: day.messages_stored_total,
            bytes_stored_total: day.bytes_stored_total,
            queries_executed_total: day.queries_executed_total,
            embeddings_generated_total: day.embeddings_generated_total,
        })
        .expect("reconciliation should succeed");

    assert_eq!(
        reconciliation.decision,
        DataLayerM7BillingReconciliationDecision::Match
    );
    assert_eq!(
        reconciliation.reason_code,
        DATA_LAYER_M7_BILLING_RECONCILIATION_MATCH_REASON_CODE
    );
}

#[test]
fn spec_c07_billing_reconciliation_reports_mismatch_totals() {
    let mut registry = DataLayerM7TelemetryRegistry::new();
    registry
        .ingest_point(telemetry_point(
            "kamn:did:owner:alpha",
            "kamn:did:agent:alpha-1",
            1_708_560_100,
            9,
            1_500,
            4,
            2,
        ))
        .expect("telemetry point should ingest");

    let projection = registry
        .project_owner_billing_daily(DataLayerM7BillingQuery {
            requester_owner_did: "kamn:did:owner:alpha".to_owned(),
            owner_did: "kamn:did:owner:alpha".to_owned(),
        })
        .expect("billing projection should succeed");
    let day = projection.first().expect("projection row should exist");

    let reconciliation = registry
        .reconcile_owner_billing_daily(DataLayerM7BillingReconciliationInput {
            requester_owner_did: "kamn:did:owner:alpha".to_owned(),
            owner_did: "kamn:did:owner:alpha".to_owned(),
            bucket_day_epoch_seconds: day.bucket_day_epoch_seconds,
            messages_stored_total: day.messages_stored_total + 1,
            bytes_stored_total: day.bytes_stored_total,
            queries_executed_total: day.queries_executed_total,
            embeddings_generated_total: day.embeddings_generated_total,
        })
        .expect("reconciliation should succeed");

    assert_eq!(
        reconciliation.decision,
        DataLayerM7BillingReconciliationDecision::Mismatch
    );
    assert_eq!(
        reconciliation.reason_code,
        DATA_LAYER_M7_BILLING_RECONCILIATION_MISMATCH_REASON_CODE
    );
    assert_eq!(
        reconciliation.projected_messages_stored_total,
        day.messages_stored_total
    );
    assert_eq!(
        reconciliation.statement_messages_stored_total,
        day.messages_stored_total + 1
    );
}

#[test]
fn spec_c08_cross_owner_reconciliation_is_denied_fail_closed() {
    let mut registry = DataLayerM7TelemetryRegistry::new();
    registry
        .ingest_point(telemetry_point(
            "kamn:did:owner:alpha",
            "kamn:did:agent:alpha-1",
            1_708_560_100,
            3,
            900,
            2,
            1,
        ))
        .expect("telemetry point should ingest");

    let denied = registry.reconcile_owner_billing_daily(DataLayerM7BillingReconciliationInput {
        requester_owner_did: "kamn:did:owner:intruder".to_owned(),
        owner_did: "kamn:did:owner:alpha".to_owned(),
        bucket_day_epoch_seconds: 1_708_473_600,
        messages_stored_total: 3,
        bytes_stored_total: 900,
        queries_executed_total: 2,
        embeddings_generated_total: 1,
    });
    assert!(matches!(
        denied,
        Err(DataLayerM7TimeseriesError::OwnerScopeViolation {
            reason_code: DATA_LAYER_M7_OWNER_SCOPE_DENIED_REASON_CODE,
        })
    ));
}

#[test]
fn spec_c09_reconciliation_rejects_non_daily_aligned_bucket() {
    let registry = DataLayerM7TelemetryRegistry::new();
    let invalid = registry.reconcile_owner_billing_daily(DataLayerM7BillingReconciliationInput {
        requester_owner_did: "kamn:did:owner:alpha".to_owned(),
        owner_did: "kamn:did:owner:alpha".to_owned(),
        bucket_day_epoch_seconds: 1_708_473_601,
        messages_stored_total: 0,
        bytes_stored_total: 0,
        queries_executed_total: 0,
        embeddings_generated_total: 0,
    });
    assert!(matches!(
        invalid,
        Err(DataLayerM7TimeseriesError::InvalidBucketDayEpochSeconds(
            1_708_473_601
        ))
    ));
}
