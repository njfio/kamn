use kamn_core::{
    ObservabilityHealth, ObservabilityMetric, ObservabilityMonitor, ObservabilitySample,
    ObservabilitySeverity, ObservabilitySloProfile,
};

fn healthy_sample() -> ObservabilitySample {
    ObservabilitySample {
        latency_p50_ms: 80,
        latency_p99_ms: 220,
        throughput_tps: 2_000,
        error_rate_pct: 0.2,
        availability_pct: 99.95,
        timestamp_epoch_s: 1_720_000_000,
    }
}

#[test]
fn healthy_sample_produces_no_alerts() {
    let mut monitor = ObservabilityMonitor::new(ObservabilitySloProfile::baseline());
    let report = monitor
        .evaluate(healthy_sample())
        .expect("healthy sample should evaluate");

    assert_eq!(report.overall_health, ObservabilityHealth::Healthy);
    assert!(report.alerts.is_empty());
}

#[test]
fn degraded_sample_flags_latency_and_error_alerts() {
    let mut monitor = ObservabilityMonitor::new(ObservabilitySloProfile::baseline());
    let report = monitor
        .evaluate(ObservabilitySample {
            latency_p50_ms: 180,
            latency_p99_ms: 520,
            throughput_tps: 1_600,
            error_rate_pct: 2.8,
            availability_pct: 99.40,
            timestamp_epoch_s: 1_720_000_300,
        })
        .expect("sample should evaluate");

    assert_eq!(report.overall_health, ObservabilityHealth::Critical);
    assert!(report.alerts.iter().any(|alert| {
        alert.metric == ObservabilityMetric::LatencyP99
            && alert.severity == ObservabilitySeverity::Critical
    }));
    assert!(report.alerts.iter().any(|alert| {
        alert.metric == ObservabilityMetric::ErrorRate
            && alert.severity == ObservabilitySeverity::Critical
    }));
}

#[test]
fn integration_snapshot_rollup_is_deterministic() {
    let mut monitor = ObservabilityMonitor::new(ObservabilitySloProfile::baseline());
    monitor
        .evaluate(healthy_sample())
        .expect("first sample should evaluate");
    monitor
        .evaluate(ObservabilitySample {
            latency_p50_ms: 120,
            latency_p99_ms: 310,
            throughput_tps: 1_750,
            error_rate_pct: 1.2,
            availability_pct: 99.70,
            timestamp_epoch_s: 1_720_000_600,
        })
        .expect("second sample should evaluate");

    let snapshot = monitor.snapshot();
    assert_eq!(snapshot.total_samples, 2);
    assert_eq!(snapshot.healthy_samples, 1);
    assert_eq!(snapshot.degraded_samples, 1);
    assert_eq!(snapshot.critical_samples, 0);
    assert_eq!(snapshot.latest_health, ObservabilityHealth::Degraded);
}

#[test]
fn regression_availability_breach_triggers_critical_alert() {
    // Regression: #206
    let mut monitor = ObservabilityMonitor::new(ObservabilitySloProfile::baseline());
    let report = monitor
        .evaluate(ObservabilitySample {
            latency_p50_ms: 90,
            latency_p99_ms: 250,
            throughput_tps: 1_900,
            error_rate_pct: 0.3,
            availability_pct: 98.8,
            timestamp_epoch_s: 1_720_000_900,
        })
        .expect("sample should evaluate");

    assert!(report.alerts.iter().any(|alert| {
        alert.metric == ObservabilityMetric::Availability
            && alert.severity == ObservabilitySeverity::Critical
    }));
}
