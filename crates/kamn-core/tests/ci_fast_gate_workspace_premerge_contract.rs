const CI_FAST_GATE_WORKFLOW: &str = include_str!("../../../.github/workflows/ci-fast-gate.yml");

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
