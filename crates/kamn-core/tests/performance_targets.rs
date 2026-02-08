use kamn_core::observability::ObservabilitySample;
use kamn_core::{
    evaluate_performance_from_observability, evaluate_performance_run, PerformanceMetric,
    PerformanceRunError, PerformanceSample, PrdPerformanceTargets,
};

fn prd_compliant_samples() -> Vec<PerformanceSample> {
    vec![
        PerformanceSample {
            latency_p50_ms: 80,
            latency_p99_ms: 410,
            throughput_tps: 10_800,
            availability_pct: 99.95,
            timestamp_epoch_s: 1,
        },
        PerformanceSample {
            latency_p50_ms: 92,
            latency_p99_ms: 455,
            throughput_tps: 10_250,
            availability_pct: 99.92,
            timestamp_epoch_s: 2,
        },
        PerformanceSample {
            latency_p50_ms: 87,
            latency_p99_ms: 430,
            throughput_tps: 10_100,
            availability_pct: 99.90,
            timestamp_epoch_s: 3,
        },
    ]
}

#[test]
fn benchmark_run_meets_prd_targets_with_stable_aggregate() {
    let report = evaluate_performance_run(
        "bench-q1-2026",
        &prd_compliant_samples(),
        &PrdPerformanceTargets::v13_2(),
    )
    .expect("report should evaluate");

    assert!(report.meets_targets);
    assert_eq!(report.sample_count, 3);
    assert_eq!(report.aggregate.latency_p50_ms, 87);
    assert_eq!(report.aggregate.latency_p99_ms, 455);
    assert_eq!(report.aggregate.throughput_tps, 10_100);
    assert_eq!(report.aggregate.availability_pct, 99.90);

    let throughput = report
        .metric_result(PerformanceMetric::Throughput)
        .expect("throughput result present");
    assert!(throughput.met);
    assert_eq!(throughput.threshold, 10_000.0);

    assert!(report.bottlenecks().is_empty());
}

#[test]
fn benchmark_run_surfaces_bottlenecks_in_deviation_order() {
    let report = evaluate_performance_run(
        "bench-q1-2026-regression",
        &[
            PerformanceSample {
                latency_p50_ms: 130,
                latency_p99_ms: 680,
                throughput_tps: 7_000,
                availability_pct: 99.10,
                timestamp_epoch_s: 20,
            },
            PerformanceSample {
                latency_p50_ms: 122,
                latency_p99_ms: 650,
                throughput_tps: 7_200,
                availability_pct: 99.25,
                timestamp_epoch_s: 21,
            },
        ],
        &PrdPerformanceTargets::v13_2(),
    )
    .expect("report should evaluate");

    assert!(!report.meets_targets);
    assert_eq!(
        report.bottlenecks(),
        vec![
            PerformanceMetric::Throughput,
            PerformanceMetric::LatencyP99,
            PerformanceMetric::Availability,
            PerformanceMetric::LatencyP50,
        ]
    );

    let p99 = report
        .metric_result(PerformanceMetric::LatencyP99)
        .expect("p99 result present");
    assert!(!p99.met);
    assert!(p99.deviation_pct > 30.0);
    assert!(p99.remediation.contains("processor backlog"));
}

#[test]
fn integration_observability_samples_feed_benchmark_evidence() {
    let report = evaluate_performance_from_observability(
        "obs-bridge-run",
        &[
            ObservabilitySample {
                latency_p50_ms: 89,
                latency_p99_ms: 470,
                throughput_tps: 10_050,
                error_rate_pct: 0.8,
                availability_pct: 99.91,
                timestamp_epoch_s: 100,
            },
            ObservabilitySample {
                latency_p50_ms: 93,
                latency_p99_ms: 490,
                throughput_tps: 10_200,
                error_rate_pct: 0.7,
                availability_pct: 99.93,
                timestamp_epoch_s: 101,
            },
        ],
        &PrdPerformanceTargets::v13_2(),
    )
    .expect("conversion should evaluate");

    assert!(report.meets_targets);
    assert_eq!(report.aggregate.latency_p99_ms, 490);
}

#[test]
fn regression_availability_floor_breach_is_always_reported() {
    // Regression: #184
    let report = evaluate_performance_run(
        "bench-availability-regression",
        &[
            PerformanceSample {
                latency_p50_ms: 82,
                latency_p99_ms: 420,
                throughput_tps: 10_450,
                availability_pct: 99.89,
                timestamp_epoch_s: 30,
            },
            PerformanceSample {
                latency_p50_ms: 84,
                latency_p99_ms: 430,
                throughput_tps: 10_420,
                availability_pct: 99.97,
                timestamp_epoch_s: 31,
            },
        ],
        &PrdPerformanceTargets::v13_2(),
    )
    .expect("report should evaluate");

    assert!(!report.meets_targets);
    assert!(report
        .bottlenecks()
        .contains(&PerformanceMetric::Availability));
}

#[test]
fn rejects_empty_sample_set() {
    assert_eq!(
        evaluate_performance_run("empty-run", &[], &PrdPerformanceTargets::v13_2()),
        Err(PerformanceRunError::EmptySamples)
    );
}
