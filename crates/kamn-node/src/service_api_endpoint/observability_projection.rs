use crate::NodeBootstrapReport;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct ServiceApiObservabilitySnapshot {
    pub(super) source: String,
    pub(super) latency_p50_ms: u64,
    pub(super) latency_p99_ms: u64,
    pub(super) throughput_tps: u64,
    pub(super) error_rate_bps: u64,
    pub(super) availability_bps: u64,
    pub(super) health: String,
    pub(super) alert_count: usize,
}

pub(super) fn resolve_service_api_observability(
    report: &NodeBootstrapReport,
) -> ServiceApiObservabilitySnapshot {
    if let (
        Some(latency_p50_ms),
        Some(latency_p99_ms),
        Some(throughput_tps),
        Some(error_rate_bps),
        Some(availability_bps),
        Some(health),
        Some(alert_count),
    ) = (
        report.daemon_observability_latency_p50_ms,
        report.daemon_observability_latency_p99_ms,
        report.daemon_observability_throughput_tps,
        report.daemon_observability_error_rate_bps,
        report.daemon_observability_availability_bps,
        report.daemon_observability_health.as_deref(),
        report.daemon_observability_alert_count,
    ) {
        return ServiceApiObservabilitySnapshot {
            source: "daemon".to_owned(),
            latency_p50_ms,
            latency_p99_ms,
            throughput_tps,
            error_rate_bps,
            availability_bps,
            health: health.to_owned(),
            alert_count,
        };
    }

    if let (
        Some(latency_p50_ms),
        Some(latency_p99_ms),
        Some(throughput_tps),
        Some(error_rate_bps),
        Some(availability_bps),
        Some(health),
        Some(alert_count),
    ) = (
        report.kolme_live_observability_latency_p50_ms,
        report.kolme_live_observability_latency_p99_ms,
        report.kolme_live_observability_throughput_tps,
        report.kolme_live_observability_error_rate_bps,
        report.kolme_live_observability_availability_bps,
        report.kolme_live_observability_health.as_deref(),
        report.kolme_live_observability_alert_count,
    ) {
        return ServiceApiObservabilitySnapshot {
            source: "kolme-live".to_owned(),
            latency_p50_ms,
            latency_p99_ms,
            throughput_tps,
            error_rate_bps,
            availability_bps,
            health: health.to_owned(),
            alert_count,
        };
    }

    ServiceApiObservabilitySnapshot {
        source: "unknown".to_owned(),
        latency_p50_ms: 0,
        latency_p99_ms: 0,
        throughput_tps: 0,
        error_rate_bps: 0,
        availability_bps: 0,
        health: "unknown".to_owned(),
        alert_count: 0,
    }
}
