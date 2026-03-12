use super::{
    DataLayerM7TelemetryPointRecord, DataLayerM7TimeseriesError,
    DATA_LAYER_M7_OBSERVABILITY_SAMPLE_INVALID_REASON_CODE,
};
use crate::{ObservabilityError, ObservabilitySample};
use kamn_data_layer::{
    project_data_layer_m7_observability_sample as project_m7_observability_sample,
    DataLayerM7ObservabilityProjection, DataLayerM7ObservabilityProjectionInput,
};

/// Projects one M7 telemetry point into a canonical observability sample.
pub fn data_layer_m7_project_observability_sample(
    point: &DataLayerM7TelemetryPointRecord,
) -> ObservabilitySample {
    observability_sample_from_projection(project_m7_observability_projection(point))
}

pub(crate) fn map_observability_error_to_timeseries(
    _error: ObservabilityError,
) -> DataLayerM7TimeseriesError {
    DataLayerM7TimeseriesError::ObservabilitySampleInvalid {
        reason_code: DATA_LAYER_M7_OBSERVABILITY_SAMPLE_INVALID_REASON_CODE,
    }
}

pub(crate) fn project_m7_observability_projection(
    point: &DataLayerM7TelemetryPointRecord,
) -> DataLayerM7ObservabilityProjection {
    project_m7_observability_sample(&m7_observability_projection_input(point))
}

pub(crate) fn m7_observability_projection_input(
    point: &DataLayerM7TelemetryPointRecord,
) -> DataLayerM7ObservabilityProjectionInput {
    DataLayerM7ObservabilityProjectionInput {
        timestamp_epoch_seconds: point.timestamp_epoch_seconds,
        ingress_latency_ms_p95: point.ingress_latency_ms_p95,
        egress_latency_ms_p95: point.egress_latency_ms_p95,
        message_count: point.message_count,
        query_count: point.query_count,
        embedding_count: point.embedding_count,
        embedding_anomaly_count: point.embedding_anomaly_count,
        active_sessions: point.active_sessions,
    }
}

pub(crate) fn observability_sample_from_projection(
    projection: DataLayerM7ObservabilityProjection,
) -> ObservabilitySample {
    ObservabilitySample {
        latency_p50_ms: projection.latency_p50_ms,
        latency_p99_ms: projection.latency_p99_ms,
        throughput_tps: projection.throughput_tps,
        error_rate_pct: projection.error_rate_pct,
        availability_pct: projection.availability_pct,
        timestamp_epoch_s: projection.timestamp_epoch_s,
    }
}
