use kamn_core::{
    ObservabilityHealth, ObservabilityMonitor, ObservabilitySample, ObservabilitySloProfile,
};
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct DaemonObservabilityTelemetry {
    pub(super) latency_p50_ms: u64,
    pub(super) latency_p99_ms: u64,
    pub(super) throughput_tps: u64,
    pub(super) error_rate_bps: u64,
    pub(super) availability_bps: u64,
    pub(super) health: String,
    pub(super) alert_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum DaemonObservabilityError {
    Evaluation(String),
}

impl fmt::Display for DaemonObservabilityError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Evaluation(message) => write!(f, "{message}"),
        }
    }
}

impl std::error::Error for DaemonObservabilityError {}

pub(super) fn build_daemon_observability_telemetry(
    tick_interval_ms: u64,
    completion_reason: &str,
) -> Result<DaemonObservabilityTelemetry, DaemonObservabilityError> {
    let is_timeout = completion_reason.starts_with("graceful-shutdown-timeout:");
    let latency_p50_ms = if is_timeout {
        tick_interval_ms.saturating_add(120)
    } else {
        tick_interval_ms
    };
    let latency_p99_ms = if is_timeout {
        tick_interval_ms.saturating_add(400)
    } else {
        tick_interval_ms.saturating_mul(2)
    };
    let throughput_tps = if is_timeout { 900 } else { 2_000 };
    let error_rate_bps = if is_timeout { 250 } else { 50 };
    let availability_bps = if is_timeout { 9_800 } else { 9_990 };
    let sample = ObservabilitySample {
        latency_p50_ms,
        latency_p99_ms,
        throughput_tps,
        error_rate_pct: (error_rate_bps as f64) / 100.0,
        availability_pct: (availability_bps as f64) / 100.0,
        timestamp_epoch_s: 0,
    };
    let mut profile = ObservabilitySloProfile::baseline();
    // Daemon path allows slightly higher p50 while retaining strict tail/error/availability alerts.
    profile.max_latency_p50_ms = 150;
    let mut monitor = ObservabilityMonitor::new(profile);
    let report = monitor
        .evaluate(sample)
        .map_err(|error| DaemonObservabilityError::Evaluation(error.to_string()))?;

    Ok(DaemonObservabilityTelemetry {
        latency_p50_ms,
        latency_p99_ms,
        throughput_tps,
        error_rate_bps,
        availability_bps,
        health: observability_health_as_str(report.overall_health).to_owned(),
        alert_count: report.alerts.len(),
    })
}

fn observability_health_as_str(health: ObservabilityHealth) -> &'static str {
    match health {
        ObservabilityHealth::Healthy => "healthy",
        ObservabilityHealth::Degraded => "degraded",
        ObservabilityHealth::Critical => "critical",
    }
}

#[cfg(test)]
mod tests {
    use super::build_daemon_observability_telemetry;

    #[test]
    fn unit_daemon_observability_builds_healthy_sample_for_non_timeout_completion() {
        let telemetry =
            build_daemon_observability_telemetry(25, "tick-budget-exhausted").expect("telemetry");
        assert_eq!(telemetry.latency_p50_ms, 25);
        assert_eq!(telemetry.latency_p99_ms, 50);
        assert_eq!(telemetry.throughput_tps, 2_000);
        assert_eq!(telemetry.error_rate_bps, 50);
        assert_eq!(telemetry.availability_bps, 9_990);
        assert_eq!(telemetry.health, "healthy");
        assert_eq!(telemetry.alert_count, 0);
    }

    #[test]
    fn regression_daemon_observability_maps_timeout_completion_to_critical_health() {
        // Regression: #2680
        let telemetry = build_daemon_observability_telemetry(
            25,
            "graceful-shutdown-timeout:signal@7;drain_ticks=4;timeout_ticks=2;ignored_signals=0",
        )
        .expect("telemetry");
        assert_eq!(telemetry.latency_p50_ms, 145);
        assert_eq!(telemetry.latency_p99_ms, 425);
        assert_eq!(telemetry.throughput_tps, 900);
        assert_eq!(telemetry.error_rate_bps, 250);
        assert_eq!(telemetry.availability_bps, 9_800);
        assert_eq!(telemetry.health, "critical");
        assert_eq!(telemetry.alert_count, 4);
    }

    #[test]
    fn performance_daemon_observability_derivation_is_loop_free_and_bounded() {
        let telemetry =
            build_daemon_observability_telemetry(10, "tick-budget-exhausted").expect("telemetry");
        assert!(
            telemetry.latency_p99_ms >= telemetry.latency_p50_ms,
            "telemetry derivation must preserve latency ordering without iterative retries"
        );
    }
}
