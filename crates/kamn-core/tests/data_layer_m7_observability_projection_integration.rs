use kamn_core::{
    data_layer_m7_project_observability_sample, DataLayerM7TelemetryPointInput,
    DataLayerM7TelemetryRegistry,
};
use kamn_data_layer::{
    project_data_layer_m7_observability_sample, DataLayerM7ObservabilityProjectionInput,
};

fn telemetry_point() -> DataLayerM7TelemetryPointInput {
    DataLayerM7TelemetryPointInput {
        owner_did: "kamn:did:owner:alpha".to_owned(),
        agent_did: "kamn:did:agent:alpha-1".to_owned(),
        timestamp_epoch_seconds: 1_708_560_100,
        message_count: 11,
        bytes_stored: 1_800,
        query_count: 5,
        embedding_count: 4,
        embedding_anomaly_count: 1,
        ingress_latency_ms_p95: 75,
        egress_latency_ms_p95: 120,
        active_sessions: 2,
    }
}

#[test]
fn integration_core_observability_wrapper_matches_extracted_data_layer_projection() {
    let mut registry = DataLayerM7TelemetryRegistry::new();
    registry
        .ingest_point(telemetry_point())
        .expect("telemetry point should ingest");

    let point = registry
        .points_for_owner("kamn:did:owner:alpha")
        .expect("owner points should exist")
        .first()
        .expect("telemetry point should exist");
    let core_sample = data_layer_m7_project_observability_sample(point);
    let extracted_sample =
        project_data_layer_m7_observability_sample(&DataLayerM7ObservabilityProjectionInput {
            timestamp_epoch_seconds: point.timestamp_epoch_seconds,
            ingress_latency_ms_p95: point.ingress_latency_ms_p95,
            egress_latency_ms_p95: point.egress_latency_ms_p95,
            message_count: point.message_count,
            query_count: point.query_count,
            embedding_count: point.embedding_count,
            embedding_anomaly_count: point.embedding_anomaly_count,
            active_sessions: point.active_sessions,
        });

    assert_eq!(core_sample.latency_p50_ms, extracted_sample.latency_p50_ms);
    assert_eq!(core_sample.latency_p99_ms, extracted_sample.latency_p99_ms);
    assert_eq!(core_sample.throughput_tps, extracted_sample.throughput_tps);
    assert_eq!(core_sample.error_rate_pct, extracted_sample.error_rate_pct);
    assert_eq!(
        core_sample.availability_pct,
        extracted_sample.availability_pct
    );
    assert_eq!(
        core_sample.timestamp_epoch_s,
        extracted_sample.timestamp_epoch_s
    );
}
