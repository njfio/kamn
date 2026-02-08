const DOC: &str = include_str!("../../../docs/foundation/ci-caching-parallelism.md");

#[test]
fn doc_contains_selector_scope_and_cache_guidance() {
    assert!(DOC.contains("shared-key: kamn-rust-ci-v1"));
    assert!(DOC.contains("run_ci_tool_checks"));
    assert!(DOC.contains("scripts/deploy/test_preflight_topology.sh"));
}

#[test]
fn regression_requires_performance_threshold_gate_commands() {
    // Regression: #595
    assert!(DOC.contains("generate_performance_smoke_report.sh --lane smoke|deep"));
    assert!(DOC.contains("check_performance_thresholds.sh --lane smoke|deep"));
    assert!(DOC.contains(".ci/performance-targets.env"));
}
