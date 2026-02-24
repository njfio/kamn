use super::{ServiceApiRuntimeState, ServiceApiSnapshot};
use std::collections::VecDeque;
use std::time::{Duration, Instant};

const SERVICE_API_RUNTIME_OBSERVABILITY_SOURCE: &str = "service-api-runtime";
const SERVICE_API_RUNTIME_OBSERVABILITY_SAMPLE_CAPACITY: usize = 1_024;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct ServiceApiRuntimeObservabilityProjection {
    pub(super) source: String,
    pub(super) latency_p50_ms: u64,
    pub(super) latency_p99_ms: u64,
    pub(super) throughput_tps: u64,
    pub(super) error_rate_bps: u64,
    pub(super) availability_bps: u64,
    pub(super) health: String,
    pub(super) alert_count: usize,
}

#[derive(Debug)]
pub(super) struct ServiceApiRuntimeObservability {
    started_at: Instant,
    total_requests: u64,
    error_requests: u64,
    latency_samples_ms: VecDeque<u64>,
}

impl ServiceApiRuntimeObservability {
    pub(super) fn new(started_at: Instant) -> Self {
        Self {
            started_at,
            total_requests: 0,
            error_requests: 0,
            latency_samples_ms: VecDeque::new(),
        }
    }

    pub(super) fn record_request(&mut self, status_code: u16, duration: Duration) {
        self.total_requests = self.total_requests.saturating_add(1);
        if status_code >= 400 {
            self.error_requests = self.error_requests.saturating_add(1);
        }
        if self.latency_samples_ms.len() >= SERVICE_API_RUNTIME_OBSERVABILITY_SAMPLE_CAPACITY {
            let _ = self.latency_samples_ms.pop_front();
        }
        let latency_ms = duration.as_millis().min(u128::from(u64::MAX)) as u64;
        self.latency_samples_ms.push_back(latency_ms.max(1));
    }

    pub(super) fn project(&self) -> Option<ServiceApiRuntimeObservabilityProjection> {
        if self.total_requests == 0 {
            return None;
        }

        let latency_samples: Vec<u64> = self.latency_samples_ms.iter().copied().collect();
        let latency_p50_ms = percentile_ms(latency_samples.as_slice(), 50, 1);
        let latency_p99_ms = percentile_ms(latency_samples.as_slice(), 99, latency_p50_ms.max(1));
        let elapsed_ms = self
            .started_at
            .elapsed()
            .as_millis()
            .min(u128::from(u64::MAX)) as u64;
        let throughput_tps = self.total_requests.saturating_mul(1_000) / elapsed_ms.max(1);
        let error_rate_bps = self.error_requests.saturating_mul(10_000) / self.total_requests;
        let availability_bps = 10_000_u64.saturating_sub(error_rate_bps.min(10_000));
        let health = if error_rate_bps >= 1_000 || latency_p99_ms >= 2_000 {
            "critical"
        } else if error_rate_bps > 0 || latency_p99_ms >= 500 {
            "degraded"
        } else {
            "healthy"
        };
        let alert_count = usize::from(error_rate_bps > 0)
            .saturating_add(usize::from(latency_p99_ms >= 500))
            .saturating_add(usize::from(throughput_tps == 0));

        Some(ServiceApiRuntimeObservabilityProjection {
            source: SERVICE_API_RUNTIME_OBSERVABILITY_SOURCE.to_owned(),
            latency_p50_ms,
            latency_p99_ms,
            throughput_tps,
            error_rate_bps,
            availability_bps,
            health: health.to_owned(),
            alert_count,
        })
    }
}

pub(super) async fn snapshot_with_runtime_observability(
    state: &ServiceApiRuntimeState,
) -> ServiceApiSnapshot {
    let runtime_projection = {
        let observability = state.runtime_observability.lock().await;
        observability.project()
    };
    match runtime_projection {
        Some(runtime_projection) => {
            let mut snapshot = state.snapshot.clone();
            snapshot.observability_source = runtime_projection.source;
            snapshot.observability_latency_p50_ms = runtime_projection.latency_p50_ms;
            snapshot.observability_latency_p99_ms = runtime_projection.latency_p99_ms;
            snapshot.observability_throughput_tps = runtime_projection.throughput_tps;
            snapshot.observability_error_rate_bps = runtime_projection.error_rate_bps;
            snapshot.observability_availability_bps = runtime_projection.availability_bps;
            snapshot.observability_health = runtime_projection.health;
            snapshot.observability_alert_count = runtime_projection.alert_count;
            snapshot
        }
        None => state.snapshot.clone(),
    }
}

pub(super) async fn record_runtime_observation(
    state: &ServiceApiRuntimeState,
    status_code: u16,
    duration: Duration,
) {
    let mut observability = state.runtime_observability.lock().await;
    observability.record_request(status_code, duration);
}

fn percentile_ms(samples: &[u64], percentile: usize, fallback: u64) -> u64 {
    if samples.is_empty() {
        return fallback.max(1);
    }
    let mut ordered = samples.to_vec();
    ordered.sort_unstable();
    let scaled = percentile.min(100);
    let index = if ordered.len() == 1 {
        0
    } else {
        (ordered.len() - 1) * scaled / 100
    };
    ordered[index].max(1)
}
