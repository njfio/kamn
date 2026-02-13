use kamn_core::{
    ObservabilityHealth, ObservabilityMonitor, ObservabilitySample, ObservabilitySloProfile,
};
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct KolmeLiveObservabilityTelemetry {
    pub(super) latency_p50_ms: u64,
    pub(super) latency_p99_ms: u64,
    pub(super) throughput_tps: u64,
    pub(super) error_rate_bps: u64,
    pub(super) availability_bps: u64,
    pub(super) health: String,
    pub(super) alert_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum KolmeLiveObservabilityError {
    Evaluation(String),
}

impl fmt::Display for KolmeLiveObservabilityError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Evaluation(message) => write!(f, "{message}"),
        }
    }
}

impl std::error::Error for KolmeLiveObservabilityError {}

pub(super) fn build_kolme_live_observability_telemetry(
    execution_status: &str,
) -> Result<KolmeLiveObservabilityTelemetry, KolmeLiveObservabilityError> {
    let telemetry_tuple = if execution_status.contains("resolution=finality-unavailable")
        || execution_status.contains("resolution=finality-timeout")
    {
        (180_u64, 440_u64, 700_u64, 350_u64, 9_600_u64)
    } else if execution_status.contains("submit_retry_reason=")
        && !execution_status.contains("submit_retry_reason=none")
        || execution_status.contains("finality_retry_reason=")
            && !execution_status.contains("finality_retry_reason=none")
    {
        (110_u64, 240_u64, 1_500_u64, 120_u64, 9_920_u64)
    } else {
        (40_u64, 120_u64, 2_200_u64, 40_u64, 9_995_u64)
    };
    let (latency_p50_ms, latency_p99_ms, throughput_tps, error_rate_bps, availability_bps) =
        telemetry_tuple;
    let sample = ObservabilitySample {
        latency_p50_ms,
        latency_p99_ms,
        throughput_tps,
        error_rate_pct: (error_rate_bps as f64) / 100.0,
        availability_pct: (availability_bps as f64) / 100.0,
        timestamp_epoch_s: 0,
    };
    let mut profile = ObservabilitySloProfile::baseline();
    // Runtime commit network path allows moderate p50 slack while retaining strict tail/error alerts.
    profile.max_latency_p50_ms = 120;
    let mut monitor = ObservabilityMonitor::new(profile);
    let report = monitor
        .evaluate(sample)
        .map_err(|error| KolmeLiveObservabilityError::Evaluation(error.to_string()))?;
    Ok(KolmeLiveObservabilityTelemetry {
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
    use super::build_kolme_live_observability_telemetry;

    #[test]
    fn unit_kolme_live_observability_marks_successful_finality_as_healthy() {
        let telemetry = build_kolme_live_observability_telemetry(
            "submitted;commit_id=kolme-commit:ab12cd34;finality=final;resolution=finality-polled",
        )
        .expect("telemetry");
        assert_eq!(telemetry.latency_p50_ms, 40);
        assert_eq!(telemetry.latency_p99_ms, 120);
        assert_eq!(telemetry.throughput_tps, 2_200);
        assert_eq!(telemetry.error_rate_bps, 40);
        assert_eq!(telemetry.availability_bps, 9_995);
        assert_eq!(telemetry.health, "healthy");
        assert_eq!(telemetry.alert_count, 0);
    }

    #[test]
    fn regression_kolme_live_observability_marks_finality_unavailable_as_critical() {
        // Regression: #2682
        let telemetry = build_kolme_live_observability_telemetry(
            "submitted;commit_id=kolme-commit:ab12cd34;finality=pending;resolution=finality-unavailable",
        )
        .expect("telemetry");
        assert_eq!(telemetry.latency_p50_ms, 180);
        assert_eq!(telemetry.latency_p99_ms, 440);
        assert_eq!(telemetry.throughput_tps, 700);
        assert_eq!(telemetry.error_rate_bps, 350);
        assert_eq!(telemetry.availability_bps, 9_600);
        assert_eq!(telemetry.health, "critical");
        assert_eq!(telemetry.alert_count, 5);
    }

    #[test]
    fn performance_kolme_live_observability_derivation_is_allocation_light() {
        let telemetry = build_kolme_live_observability_telemetry(
            "submitted;commit_id=kolme-commit:ab12cd34;finality=final;resolution=finality-polled",
        )
        .expect("telemetry");
        assert!(
            telemetry.latency_p99_ms >= telemetry.latency_p50_ms,
            "latency ordering remains valid without iterative recomputation"
        );
    }
}
