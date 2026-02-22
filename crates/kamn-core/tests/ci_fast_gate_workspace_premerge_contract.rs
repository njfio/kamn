const CI_FAST_GATE_WORKFLOW: &str = include_str!("../../../.github/workflows/ci-fast-gate.yml");
const CI_STRATEGY_DOC: &str = include_str!("../../../docs/ci/strategy.md");

#[test]
fn spec_c01_ci_fast_gate_declares_workspace_premerge_job() {
    assert!(
        CI_FAST_GATE_WORKFLOW.contains("workspace-premerge-gate:"),
        "ci_fast_gate_workspace_premerge_job_missing",
    );
}

#[test]
fn spec_c02_ci_fast_gate_workspace_premerge_job_runs_on_pull_requests() {
    assert!(
        CI_FAST_GATE_WORKFLOW.contains("if: github.event_name == 'pull_request'"),
        "ci_fast_gate_workspace_premerge_pull_request_scope_missing",
    );
}

#[test]
fn spec_c03_ci_fast_gate_workspace_premerge_job_executes_workspace_test_command() {
    assert!(
        CI_FAST_GATE_WORKFLOW
            .contains("cargo test --workspace --locked --all-features --no-fail-fast"),
        "ci_fast_gate_workspace_premerge_workspace_test_command_missing",
    );
}

#[test]
fn spec_c04_ci_fast_gate_workspace_premerge_job_uses_bounded_retry_wrapper() {
    assert!(
        CI_FAST_GATE_WORKFLOW.contains("bash scripts/ci/run_with_retry.sh"),
        "ci_fast_gate_workspace_premerge_retry_wrapper_missing",
    );
    assert!(
        CI_FAST_GATE_WORKFLOW.contains("--label workspace-premerge-tests"),
        "ci_fast_gate_workspace_premerge_retry_label_missing",
    );
    assert!(
        CI_FAST_GATE_WORKFLOW.contains("--max-attempts 2"),
        "ci_fast_gate_workspace_premerge_retry_attempts_missing",
    );
}

#[test]
fn spec_c05_ci_strategy_docs_record_workspace_premerge_gate_contract() {
    assert!(
        CI_STRATEGY_DOC.contains("workspace-premerge-gate"),
        "ci_strategy_workspace_premerge_gate_marker_missing",
    );
    assert!(
        CI_STRATEGY_DOC.contains("bash scripts/ci/run_with_retry.sh --label workspace-premerge-tests --max-attempts 2 -- cargo test --workspace --locked --all-features --no-fail-fast"),
        "ci_strategy_workspace_premerge_command_marker_missing",
    );
}
