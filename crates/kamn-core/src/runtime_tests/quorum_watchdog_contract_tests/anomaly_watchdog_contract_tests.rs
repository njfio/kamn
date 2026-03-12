use super::super::*;

#[test]
fn functional_watchdog_anomaly_classifies_liveness_degradation_as_warning() {
    let evaluator = WatchdogAnomalyEvaluator;
    let input = WatchdogAnomalyWatchInput::new("sample-liveness", 100, 96, 7, 5, 30, 1)
        .expect("valid anomaly sample");
    let report = evaluator
        .evaluate(input)
        .expect("anomaly classification should succeed");
    assert_eq!(report.kind, WatchdogAnomalyKind::LivenessDegradation);
    assert_eq!(report.severity, WatchdogAnomalySeverity::Warning);
}

#[test]
fn unit_watchdog_anomaly_rejects_invalid_delivery_sample() {
    let error = WatchdogAnomalyWatchInput::new("sample-invalid", 10, 12, 5, 5, 30, 2)
        .expect_err("delivered count above expected must be rejected");
    assert_eq!(
        error,
        WatchdogAnomalyError::InvalidSampleCounts {
            expected_deliveries: 10,
            delivered_deliveries: 12
        }
    );
}

#[test]
fn integration_daemon_watchdog_anomaly_report_includes_summary_fields() {
    let evaluator = WatchdogAnomalyEvaluator;
    let input = WatchdogAnomalyWatchInput::new("sample-censorship", 100, 45, 8, 8, 60, 3)
        .expect("valid anomaly sample");
    let report = evaluate_daemon_watchdog_anomaly(&evaluator, input)
        .expect("daemon anomaly evaluation should succeed");
    assert_eq!(report.sample_id, "sample-censorship");
    assert_eq!(report.kind, WatchdogAnomalyKind::CensorshipSignal);
    assert_eq!(report.severity, WatchdogAnomalySeverity::Critical);
    assert_eq!(report.delivery_ratio_per_mille, 450);
    assert_eq!(report.targeted_peer_count, 3);
    assert_eq!(report.sample_window_secs, 60);
}

#[test]
fn regression_censorship_edge_signal_remains_detected_as_critical() {
    let evaluator = WatchdogAnomalyEvaluator;
    let input = WatchdogAnomalyWatchInput::new("sample-regression", 200, 98, 12, 12, 60, 2)
        .expect("valid anomaly sample");
    let report = evaluate_daemon_watchdog_anomaly(&evaluator, input)
        .expect("edge censorship signal should be classified");
    assert_eq!(report.kind, WatchdogAnomalyKind::CensorshipSignal);
    assert_eq!(report.severity, WatchdogAnomalySeverity::Critical);
}
