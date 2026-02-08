const DOC: &str = include_str!("../../../docs/foundation/performance-target-benchmarking.md");

#[test]
fn doc_contains_prd_13_2_thresholds() {
    assert!(DOC.contains("## PRD 13.2 Target Profile"));
    assert!(DOC.contains("Message Latency (p50) | `< 100ms`"));
    assert!(DOC.contains("Message Latency (p99) | `< 500ms`"));
    assert!(DOC.contains("Throughput | `>= 10,000 msg/sec`"));
    assert!(DOC.contains("Availability | `>= 99.9%`"));
}

#[test]
fn doc_contains_deterministic_aggregation_rules() {
    assert!(DOC.contains("## Deterministic Aggregation Rules"));
    assert!(DOC.contains("`latency_p50_ms`: median across benchmark windows."));
    assert!(DOC.contains("`latency_p99_ms`: max across benchmark windows."));
    assert!(DOC.contains("`throughput_tps`: min across benchmark windows."));
    assert!(DOC.contains("`availability_pct`: min across benchmark windows."));
}

#[test]
fn regression_requires_cost_effective_fast_lane_policy() {
    // Regression: #184
    assert!(DOC.contains("## Fast and Cost-Effective Validation Strategy"));
    assert!(DOC.contains("PR gate (fast lane):"));
    assert!(DOC.contains("Deferred deep validation (slow lane):"));
}
