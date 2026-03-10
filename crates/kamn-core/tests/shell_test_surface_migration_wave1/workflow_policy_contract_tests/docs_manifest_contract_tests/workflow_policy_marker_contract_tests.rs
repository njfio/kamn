use super::super::super::support::*;

#[test]
fn spec_c14_workflow_cache_policy_markers() {
    let (fast_workflow, deep_workflow) = workflow_texts();
    assert_shared_cache_markers(&fast_workflow, &deep_workflow);
    assert!(
        deep_workflow.contains("run_invariant_harness.sh --mode deep --parallelism 2"),
        "ci-deep-validate workflow missing bounded invariant harness parallelism marker"
    );
    assert!(
        fast_workflow.contains("if: steps.scope.outputs.run_ci_tool_checks == 'true'"),
        "ci-fast-gate workflow missing run_ci_tool_checks gate marker"
    );
    assert!(
        fast_workflow.contains("KAMN_CI_TOOLS_FAST_MODE: 'true'"),
        "ci-fast-gate workflow missing fast-mode env marker for CI tools"
    );
}

#[test]
fn spec_c15_workflow_performance_policy_markers() {
    let (fast_workflow, deep_workflow) = workflow_texts();
    assert_contains_all(
        &fast_workflow,
        fast_workflow_performance_markers(),
        "ci-fast-gate workflow performance policy markers",
    );
    assert_contains_all(
        &deep_workflow,
        deep_workflow_performance_markers(),
        "ci-deep-validate workflow performance policy markers",
    );
}

fn workflow_texts() -> (String, String) {
    (read_text(FAST_WORKFLOW), read_text(DEEP_WORKFLOW))
}

fn assert_shared_cache_markers(fast_workflow: &str, deep_workflow: &str) {
    for workflow in [fast_workflow, deep_workflow] {
        assert!(
            workflow.contains("shared-key: kamn-rust-ci-v1"),
            "workflow missing rust-cache shared key marker"
        );
        assert!(
            workflow.contains("save-if: ${{ github.ref == 'refs/heads/main' }}"),
            "workflow missing rust-cache save-if guard marker"
        );
    }
}

fn fast_workflow_performance_markers() -> &'static [&'static str] {
    &[
        "Generate performance smoke report",
        "generate_performance_smoke_report.sh --lane smoke --output-json performance-smoke-report.json",
        "Check performance thresholds (smoke)",
        "check_performance_thresholds.sh --lane smoke --report-json performance-smoke-report.json --profile-file .ci/performance-targets.env",
        "Generate fast-gate budget delta report",
        "generate_fast_gate_budget_delta_report.sh --current-json ci-budget-fast-gate.json --baseline-file .ci/fast-gate-budget-delta.env --output-json ci-budget-fast-gate-delta.json",
        "Check fast-gate budget delta thresholds",
        "check_fast_gate_budget_delta_threshold.sh --report-json ci-budget-fast-gate-delta.json --threshold-file .ci/fast-gate-budget-delta.env --waiver-file .ci/fast-gate-budget-delta-waiver.json",
        "Upload fast-gate budget delta telemetry",
        "ci-budget-fast-gate-delta-${{ github.run_id }}-${{ github.run_attempt }}",
    ]
}

fn deep_workflow_performance_markers() -> &'static [&'static str] {
    &[
        "Generate performance smoke report",
        "generate_performance_smoke_report.sh --lane deep --output-json performance-deep-report.json",
        "Check performance thresholds (deep)",
        "check_performance_thresholds.sh --lane deep --report-json performance-deep-report.json --profile-file .ci/performance-targets.env",
    ]
}
