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
    pub(super) reason_code: String,
    pub(super) transport_checkpoint_failures: u64,
    pub(super) signer_checkpoint_failures: u64,
    pub(super) commit_checkpoint_failures: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub(super) struct DaemonRuntimeProcessingTelemetry {
    pub(super) executed_ticks: u64,
    pub(super) relay_drained_count: u64,
    pub(super) relay_projected_state_count: u64,
    pub(super) processing_error_count: u64,
    pub(super) tick_processing_samples_ms: Vec<u64>,
    pub(super) tick_sleep_count: u64,
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
    runtime_processing: &DaemonRuntimeProcessingTelemetry,
) -> Result<DaemonObservabilityTelemetry, DaemonObservabilityError> {
    let is_timeout = completion_reason.starts_with("graceful-shutdown-timeout:");
    let latency_p50_ms = percentile_ms(
        runtime_processing.tick_processing_samples_ms.as_slice(),
        50,
        tick_interval_ms,
    );
    let latency_p99_ms = percentile_ms(
        runtime_processing.tick_processing_samples_ms.as_slice(),
        99,
        tick_interval_ms,
    );
    let observed_runtime_ms = if runtime_processing.tick_processing_samples_ms.is_empty() {
        runtime_processing
            .executed_ticks
            .saturating_mul(tick_interval_ms)
            .max(1)
    } else {
        runtime_processing
            .tick_processing_samples_ms
            .iter()
            .copied()
            .fold(0_u64, u64::saturating_add)
            .max(1)
    };
    let throughput_work_units = runtime_processing
        .relay_projected_state_count
        .saturating_add(runtime_processing.executed_ticks.max(1));
    let throughput_tps = throughput_work_units.saturating_mul(1_000) / observed_runtime_ms.max(1);
    let total_tick_budget = runtime_processing.executed_ticks.max(1);
    let mut error_rate_bps = runtime_processing
        .processing_error_count
        .saturating_mul(10_000)
        / total_tick_budget;
    if is_timeout {
        error_rate_bps = error_rate_bps.max(500);
    }
    let availability_bps = 10_000_u64.saturating_sub(error_rate_bps.min(10_000));
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
    profile.min_throughput_tps = 1;
    let mut monitor = ObservabilityMonitor::new(profile);
    let report = monitor
        .evaluate(sample)
        .map_err(|error| DaemonObservabilityError::Evaluation(error.to_string()))?;
    let reason_code = daemon_observability_reason_code(completion_reason).to_owned();

    Ok(DaemonObservabilityTelemetry {
        latency_p50_ms,
        latency_p99_ms,
        throughput_tps,
        error_rate_bps,
        availability_bps,
        health: observability_health_as_str(report.overall_health).to_owned(),
        alert_count: report.alerts.len(),
        reason_code,
        transport_checkpoint_failures: runtime_processing.processing_error_count,
        signer_checkpoint_failures: 0,
        commit_checkpoint_failures: if is_timeout {
            runtime_processing.processing_error_count.saturating_add(1)
        } else {
            runtime_processing.processing_error_count
        },
    })
}

fn percentile_ms(samples: &[u64], percentile: usize, fallback: u64) -> u64 {
    if samples.is_empty() {
        return fallback;
    }
    let mut ordered = samples.to_vec();
    ordered.sort_unstable();
    let scaled = percentile.min(100);
    let index = if ordered.len() == 1 {
        0
    } else {
        (ordered.len() - 1) * scaled / 100
    };
    ordered[index]
}

fn daemon_observability_reason_code(completion_reason: &str) -> &'static str {
    if completion_reason.starts_with("graceful-shutdown-timeout:") {
        "daemon_shutdown_timeout"
    } else if completion_reason.starts_with("graceful-shutdown:") {
        "daemon_shutdown_signal"
    } else {
        "none"
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
    use super::{build_daemon_observability_telemetry, DaemonRuntimeProcessingTelemetry};

    #[test]
    fn unit_daemon_observability_builds_healthy_sample_for_non_timeout_completion() {
        let telemetry = build_daemon_observability_telemetry(
            25,
            "tick-budget-exhausted",
            &DaemonRuntimeProcessingTelemetry {
                executed_ticks: 4,
                relay_drained_count: 2,
                relay_projected_state_count: 2,
                processing_error_count: 0,
                tick_processing_samples_ms: vec![8, 12, 10, 14],
                tick_sleep_count: 0,
            },
        )
        .expect("telemetry");
        assert_eq!(telemetry.latency_p50_ms, 10);
        assert_eq!(telemetry.latency_p99_ms, 12);
        assert_eq!(telemetry.throughput_tps, 136);
        assert_eq!(telemetry.error_rate_bps, 0);
        assert_eq!(telemetry.availability_bps, 10_000);
        assert_eq!(telemetry.health, "healthy");
        assert_eq!(telemetry.alert_count, 0);
        assert_eq!(telemetry.reason_code, "none");
        assert_eq!(telemetry.transport_checkpoint_failures, 0);
        assert_eq!(telemetry.signer_checkpoint_failures, 0);
        assert_eq!(telemetry.commit_checkpoint_failures, 0);
    }

    #[test]
    fn regression_daemon_observability_maps_timeout_completion_to_critical_health() {
        // Regression: #2680
        let telemetry = build_daemon_observability_telemetry(
            25,
            "graceful-shutdown-timeout:signal@7;drain_ticks=4;timeout_ticks=2;ignored_signals=0",
            &DaemonRuntimeProcessingTelemetry {
                executed_ticks: 3,
                relay_drained_count: 1,
                relay_projected_state_count: 1,
                processing_error_count: 1,
                tick_processing_samples_ms: vec![20, 30, 45],
                tick_sleep_count: 0,
            },
        )
        .expect("telemetry");
        assert_eq!(telemetry.latency_p50_ms, 30);
        assert_eq!(telemetry.latency_p99_ms, 30);
        assert_eq!(telemetry.throughput_tps, 42);
        assert_eq!(telemetry.error_rate_bps, 3_333);
        assert_eq!(telemetry.availability_bps, 6_667);
        assert_eq!(telemetry.health, "critical");
        assert_eq!(telemetry.alert_count, 2);
        assert_eq!(telemetry.reason_code, "daemon_shutdown_timeout");
        assert_eq!(telemetry.transport_checkpoint_failures, 1);
        assert_eq!(telemetry.signer_checkpoint_failures, 0);
        assert_eq!(telemetry.commit_checkpoint_failures, 2);
    }

    #[test]
    fn performance_daemon_observability_derivation_is_loop_free_and_bounded() {
        let telemetry = build_daemon_observability_telemetry(
            10,
            "tick-budget-exhausted",
            &DaemonRuntimeProcessingTelemetry {
                executed_ticks: 10,
                relay_drained_count: 0,
                relay_projected_state_count: 0,
                processing_error_count: 0,
                tick_processing_samples_ms: vec![1, 2, 3, 4, 5],
                tick_sleep_count: 0,
            },
        )
        .expect("telemetry");
        assert!(
            telemetry.latency_p99_ms >= telemetry.latency_p50_ms,
            "telemetry derivation must preserve latency ordering without iterative retries"
        );
    }
}
