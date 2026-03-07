use kamn_data_layer::{
    project_data_layer_m7_observability_sample, DataLayerM7ObservabilityProjection,
    DataLayerM7ObservabilityProjectionInput,
};

fn projection_input(
    message_count: u64,
    query_count: u64,
    embedding_count: u64,
    embedding_anomaly_count: u64,
    active_sessions: u32,
) -> DataLayerM7ObservabilityProjectionInput {
    DataLayerM7ObservabilityProjectionInput {
        timestamp_epoch_seconds: 1_901_111_222,
        ingress_latency_ms_p95: 42,
        egress_latency_ms_p95: 87,
        message_count,
        query_count,
        embedding_count,
        embedding_anomaly_count,
        active_sessions,
    }
}

#[test]
fn integration_projection_maps_latency_and_activity_into_observability_fields() {
    let input = projection_input(9, 4, 2, 0, 3);

    let sample = project_data_layer_m7_observability_sample(&input);
    assert_eq!(
        sample,
        DataLayerM7ObservabilityProjection {
            latency_p50_ms: 42,
            latency_p99_ms: 87,
            throughput_tps: 3_015,
            error_rate_pct: 0.0,
            availability_pct: 100.0,
            timestamp_epoch_s: 1_901_111_222,
        }
    );
}

#[test]
fn integration_projection_applies_floor_clamp_and_zero_availability_rules() {
    let zero_activity = projection_input(0, 0, 0, 9, 0);
    let zero_sample = project_data_layer_m7_observability_sample(&zero_activity);
    assert_eq!(zero_sample.throughput_tps, 1);
    assert_eq!(zero_sample.error_rate_pct, 0.0);
    assert_eq!(zero_sample.availability_pct, 0.0);

    let clamped = projection_input(0, 0, 10, 25, 1);
    let clamped_sample = project_data_layer_m7_observability_sample(&clamped);
    assert_eq!(clamped_sample.error_rate_pct, 100.0);
    assert_eq!(clamped_sample.availability_pct, 100.0);
}
