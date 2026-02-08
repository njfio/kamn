const DOC: &str = include_str!("../../../docs/foundation/ci-caching-parallelism.md");

#[test]
fn doc_contains_selector_scope_and_cache_guidance() {
    assert!(DOC.contains("shared-key: kamn-rust-ci-v1"));
    assert!(DOC.contains("run_ci_tool_checks"));
    assert!(DOC.contains("scripts/deploy/test_preflight_topology.sh"));
    assert!(DOC.contains("run_sdk_parity_matrix"));
}

#[test]
fn regression_requires_performance_threshold_gate_commands() {
    // Regression: #595
    assert!(DOC.contains("generate_performance_smoke_report.sh --lane smoke|deep"));
    assert!(DOC.contains("check_performance_thresholds.sh --lane smoke|deep"));
    assert!(DOC.contains(".ci/performance-targets.env"));
}

#[test]
fn regression_requires_sdk_parity_lane_guidance() {
    // Regression: #583
    assert!(DOC.contains("scripts/sdk/run_sdk_parity_matrix.sh"));
    assert!(DOC.contains("test_scope=sdk"));
    assert!(DOC.contains("Regression: #583"));
}
