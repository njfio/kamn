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

#[test]
fn regression_requires_threshold_gate_commands() {
    // Regression: #595
    assert!(DOC.contains("## CI Threshold Gate Contract"));
    assert!(DOC.contains(".ci/performance-targets.env"));
    assert!(DOC.contains("generate_performance_smoke_report.sh --lane smoke"));
    assert!(DOC.contains("check_performance_thresholds.sh --lane deep"));
}

#[test]
fn regression_requires_runtime_invariant_fuzz_concurrency_budget_contract() {
    // Regression: #897
    assert!(DOC.contains("## Runtime Invariant/Fuzz/Concurrency Budget Contract"));
    assert!(DOC.contains("run_invariant_fuzz_concurrency_contract_lane.sh"));
    assert!(DOC.contains("check_invariant_fuzz_concurrency_policy.sh"));
    assert!(DOC.contains("KAMN_RUNTIME_INVARIANT_FUZZ_CONCURRENCY_MAX_SECONDS=180"));
    assert!(DOC.contains("kamn.runtime.invariant-fuzz-concurrency-contract-report.v1"));
}
