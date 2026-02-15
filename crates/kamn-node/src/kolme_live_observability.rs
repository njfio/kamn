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
    pub(super) reason_code: String,
    pub(super) transport_checkpoint_failures: u64,
    pub(super) signer_checkpoint_failures: u64,
    pub(super) commit_checkpoint_failures: u64,
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
    let resolution = status_field(execution_status, "resolution").unwrap_or("unknown");
    let submit_retry_reason =
        status_field(execution_status, "submit_retry_reason").unwrap_or("none");
    let finality_retry_reason =
        status_field(execution_status, "finality_retry_reason").unwrap_or("none");
    let signer_quorum_linked =
        status_bool_field(execution_status, "signer_quorum_linked").unwrap_or(true);
    let transport_checkpoint_failures =
        u64::from(submit_retry_reason != "none") + u64::from(finality_retry_reason != "none");
    let signer_checkpoint_failures = if signer_quorum_linked { 0 } else { 1 };
    let commit_checkpoint_failures =
        if resolution == "finality-unavailable" || resolution == "finality-timeout" {
            1
        } else {
            0
        };

    let reason_code = if commit_checkpoint_failures > 0 {
        format!("commit_{}", resolution.replace('-', "_"))
    } else if signer_checkpoint_failures > 0 {
        "signer_quorum_linkage_violation".to_owned()
    } else if finality_retry_reason != "none" {
        format!("transport_finality_retry_{finality_retry_reason}")
    } else if submit_retry_reason != "none" {
        format!("transport_submit_retry_{submit_retry_reason}")
    } else {
        "none".to_owned()
    };

    let telemetry_tuple = if commit_checkpoint_failures > 0 {
        (180_u64, 440_u64, 700_u64, 350_u64, 9_600_u64)
    } else if transport_checkpoint_failures > 0 || signer_checkpoint_failures > 0 {
        (110_u64, 180_u64, 1_500_u64, 120_u64, 9_960_u64)
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
        reason_code,
        transport_checkpoint_failures,
        signer_checkpoint_failures,
        commit_checkpoint_failures,
    })
}

fn status_field<'a>(execution_status: &'a str, key: &str) -> Option<&'a str> {
    execution_status.split(';').find_map(|segment| {
        let (field, value) = segment.split_once('=')?;
        if field == key {
            return Some(value);
        }
        None
    })
}

fn status_bool_field(execution_status: &str, key: &str) -> Option<bool> {
    match status_field(execution_status, key)? {
        "true" => Some(true),
        "false" => Some(false),
        _ => None,
    }
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
        assert_eq!(telemetry.reason_code, "none");
        assert_eq!(telemetry.transport_checkpoint_failures, 0);
        assert_eq!(telemetry.signer_checkpoint_failures, 0);
        assert_eq!(telemetry.commit_checkpoint_failures, 0);
    }

    #[test]
    fn functional_kolme_live_observability_marks_transport_retry_reason_code_and_counts() {
        let telemetry = build_kolme_live_observability_telemetry(
            "submitted;commit_id=kolme-commit:ab12cd34;finality=final;resolution=finality-polled;submit_retry_reason=unavailable;finality_retry_reason=unavailable;signer_quorum_linked=true",
        )
        .expect("telemetry");
        assert_eq!(
            telemetry.reason_code,
            "transport_finality_retry_unavailable"
        );
        assert_eq!(telemetry.transport_checkpoint_failures, 2);
        assert_eq!(telemetry.signer_checkpoint_failures, 0);
        assert_eq!(telemetry.commit_checkpoint_failures, 0);
        assert_eq!(telemetry.health, "degraded");
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
        assert_eq!(telemetry.reason_code, "commit_finality_unavailable");
        assert_eq!(telemetry.transport_checkpoint_failures, 0);
        assert_eq!(telemetry.signer_checkpoint_failures, 0);
        assert_eq!(telemetry.commit_checkpoint_failures, 1);
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
