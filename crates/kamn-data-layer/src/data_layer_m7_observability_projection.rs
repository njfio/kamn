//! M7 observability sample projection extracted from core telemetry contracts.
//!
//! The projection stays deterministic and side-effect free so `kamn-core` can
//! preserve its existing public observability wrapper while moving the math into
//! `kamn-data-layer`.

/// Projection input built from an M7 telemetry point.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DataLayerM7ObservabilityProjectionInput {
    /// Observation timestamp (epoch seconds).
    pub timestamp_epoch_seconds: u64,
    /// P95 ingress latency in ms.
    pub ingress_latency_ms_p95: u32,
    /// P95 egress latency in ms.
    pub egress_latency_ms_p95: u32,
    /// Message count for this sample.
    pub message_count: u64,
    /// Query count for this sample.
    pub query_count: u64,
    /// Embedding generation count for this sample.
    pub embedding_count: u64,
    /// Embedding anomaly count for this sample.
    pub embedding_anomaly_count: u64,
    /// Active session count at sample time.
    pub active_sessions: u32,
}

/// Deterministic observability projection for one M7 telemetry point.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DataLayerM7ObservabilityProjection {
    /// Observed p50 latency (milliseconds).
    pub latency_p50_ms: u64,
    /// Observed p99 latency (milliseconds).
    pub latency_p99_ms: u64,
    /// Observed throughput (operations per second).
    pub throughput_tps: u64,
    /// Observed error-rate percentage.
    pub error_rate_pct: f64,
    /// Observed availability percentage.
    pub availability_pct: f64,
    /// Sample timestamp (epoch seconds).
    pub timestamp_epoch_s: u64,
}

/// Projects one M7 telemetry point into deterministic observability fields.
pub fn project_data_layer_m7_observability_sample(
    input: &DataLayerM7ObservabilityProjectionInput,
) -> DataLayerM7ObservabilityProjection {
    DataLayerM7ObservabilityProjection {
        latency_p50_ms: u64::from(input.ingress_latency_ms_p95),
        latency_p99_ms: u64::from(
            input
                .ingress_latency_ms_p95
                .max(input.egress_latency_ms_p95),
        ),
        throughput_tps: derive_observability_throughput_tps(input),
        error_rate_pct: derive_observability_error_rate_pct(input),
        availability_pct: derive_observability_availability_pct(input.active_sessions),
        timestamp_epoch_s: input.timestamp_epoch_seconds,
    }
}

fn derive_observability_throughput_tps(input: &DataLayerM7ObservabilityProjectionInput) -> u64 {
    let activity = input
        .message_count
        .saturating_add(input.query_count)
        .saturating_add(input.embedding_count);
    let session_boost = u64::from(input.active_sessions).saturating_mul(1_000);
    activity.saturating_add(session_boost).max(1)
}

fn derive_observability_error_rate_pct(input: &DataLayerM7ObservabilityProjectionInput) -> f64 {
    if input.embedding_count == 0 {
        return 0.0;
    }
    let ratio =
        (input.embedding_anomaly_count as f64 / input.embedding_count as f64) * 100.0;
    ratio.clamp(0.0, 100.0)
}

fn derive_observability_availability_pct(active_sessions: u32) -> f64 {
    if active_sessions == 0 {
        0.0
    } else {
        100.0
    }
}
