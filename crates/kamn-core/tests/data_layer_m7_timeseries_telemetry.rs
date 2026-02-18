use kamn_core::{
    DataLayerM7BillingQuery, DataLayerM7TelemetryPointInput, DataLayerM7TelemetryRegistry,
    DataLayerM7TelemetryScopeQuery, DataLayerM7TimeseriesError,
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
            reason_code: "m7_timeseries_owner_scope_denied",
        })
    ));
}
