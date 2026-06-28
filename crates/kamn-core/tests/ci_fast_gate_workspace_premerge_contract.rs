const CI_FAST_GATE_WORKFLOW: &str = include_str!("../../../.github/workflows/ci-fast-gate.yml");
const CI_DEEP_VALIDATE_WORKFLOW: &str =
    include_str!("../../../.github/workflows/ci-deep-validate.yml");
const CI_STRATEGY_DOC: &str = include_str!("../../../docs/ci/strategy.md");
const CARGO_AUDIT_ADR: &str = include_str!("../../../docs/architecture/adr-cargo-audit-ci-gate.md");

fn fast_gate_section() -> &'static str {
    CI_FAST_GATE_WORKFLOW
        .split("workspace-premerge-gate:")
        .next()
        .expect("fast gate section should exist")
}

fn workspace_premerge_section() -> &'static str {
    CI_FAST_GATE_WORKFLOW
        .split("workspace-premerge-gate:")
        .nth(1)
        .expect("workspace premerge section should exist")
}

fn workflow_job_section(job_id: &str) -> String {
    let marker = format!("  {job_id}:");
    let mut found = false;
    let mut section = String::new();
    for line in CI_FAST_GATE_WORKFLOW.lines() {
        if line == marker {
            found = true;
        } else if found && line.starts_with("  ") && !line.starts_with("    ") {
            break;
        }
        if found {
            section.push_str(line);
            section.push('\n');
        }
    }
    assert!(found, "workflow job section missing: {job_id}");
    section
}

fn assert_cargo_audit_capture_contract(workflow: &str, workflow_name: &str) {
    assert!(
        workflow.contains("cargo_audit_status=0"),
        "{workflow_name}_cargo_audit_status_initialization_missing",
    );
    assert!(
        workflow
            .contains("cargo audit --json > cargo-audit-report.json || cargo_audit_status=\"$?\""),
        "{workflow_name}_cargo_audit_nonzero_capture_missing",
    );
    assert!(
        workflow.contains("cargo_audit_exit_status=$cargo_audit_status"),
        "{workflow_name}_cargo_audit_exit_status_marker_missing",
    );
    assert!(
        workflow.contains("python3 scripts/ci/check_cargo_audit_policy.py"),
        "{workflow_name}_cargo_audit_policy_command_missing",
    );
}

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

#[test]
fn spec_c06_ci_fast_gate_runs_cargo_audit_in_fast_gate() {
    let fast_gate = fast_gate_section();
    assert!(
        fast_gate.contains("Fast Gate (PR)"),
        "ci_fast_gate_job_marker_missing",
    );
    assert!(
        fast_gate.contains("Install cargo-audit"),
        "ci_fast_gate_cargo_audit_install_step_missing",
    );
    assert!(
        fast_gate.contains("cargo audit --json > cargo-audit-report.json"),
        "ci_fast_gate_cargo_audit_command_missing",
    );
    assert!(
        fast_gate.contains("python3 scripts/ci/check_cargo_audit_policy.py"),
        "ci_fast_gate_cargo_audit_policy_command_missing",
    );
    assert!(
        fast_gate.contains("name: ci-cargo-audit-${{ github.run_id }}-${{ github.run_attempt }}"),
        "ci_fast_gate_cargo_audit_artifact_name_missing",
    );
    assert!(
        fast_gate.contains("cargo-audit-report.json"),
        "ci_fast_gate_cargo_audit_report_artifact_missing",
    );
    assert!(
        fast_gate.contains("ci-cargo-audit-policy.json"),
        "ci_fast_gate_cargo_audit_policy_artifact_missing",
    );
}

#[test]
fn spec_c07_ci_docs_record_fast_gate_cargo_audit_contract() {
    assert!(
        CI_STRATEGY_DOC.contains("fast_gate_cargo_audit_feedback=enabled"),
        "ci_strategy_fast_gate_cargo_audit_marker_missing",
    );
    assert!(
        CARGO_AUDIT_ADR.contains("cargo_audit_fast_gate_scope=run_rust"),
        "cargo_audit_adr_fast_gate_scope_marker_missing",
    );
    assert!(
        CARGO_AUDIT_ADR.contains("cargo_audit_fast_gate_artifact=ci-cargo-audit"),
        "cargo_audit_adr_fast_gate_artifact_marker_missing",
    );
}

#[test]
fn spec_c08_workspace_premerge_no_longer_duplicates_cargo_audit() {
    let workspace_premerge = workspace_premerge_section();
    assert!(
        !workspace_premerge.contains("Install cargo-audit"),
        "workspace_premerge_cargo_audit_install_step_should_be_absent",
    );
    assert!(
        !workspace_premerge.contains("cargo audit --json > cargo-audit-report.json"),
        "workspace_premerge_cargo_audit_command_should_be_absent",
    );
}

#[test]
fn spec_c09_cargo_audit_scan_preserves_report_for_policy_after_nonzero_exit() {
    assert_cargo_audit_capture_contract(fast_gate_section(), "ci_fast_gate");
    assert_cargo_audit_capture_contract(CI_DEEP_VALIDATE_WORKFLOW, "ci_deep_validate");
}

#[test]
fn spec_c10_ci_tool_regression_has_own_runtime_budget() {
    let fast_gate = workflow_job_section("fast-gate");
    assert!(
        !fast_gate.contains("Run CI tool regression tests"),
        "fast_gate_job_must_not_run_ci_tool_regression_bundle",
    );
    assert!(
        !fast_gate.contains("bash scripts/ci/test_ci_tools.sh"),
        "fast_gate_job_must_not_share_budget_with_ci_tool_regression_bundle",
    );
    let ci_tools = workflow_job_section("ci-tool-regression-gate");
    assert!(
        ci_tools.contains("CI Tool Regression Gate (PR)"),
        "ci_tool_regression_gate_name_missing",
    );
    assert!(ci_tools.contains("timeout-minutes: 20"));
    assert!(ci_tools.contains("KAMN_CI_TOOLS_FAST_MODE: 'true'"));
    assert!(ci_tools.contains("bash scripts/ci/test_ci_tools.sh"));
}
