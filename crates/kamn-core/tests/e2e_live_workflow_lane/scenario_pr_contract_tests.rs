use crate::support::constants::SDK_DIRECT_FULL_SCENARIOS_MARKER;
use crate::support::evaluation::evaluate_contract;
use crate::support::fixtures::{read_file_if_exists, repo_root};

#[test]
fn regression_e2e_live_workflow_lane_rejects_truncated_scenario_matrix() {
    assert_workflow_failure(
        SDK_DIRECT_FULL_SCENARIOS_MARKER,
        "SDK_DIRECT_FULL_SCENARIOS=\"S-01,S-02,S-03,S-04,S-05,S-06\"",
        "sdk_direct_scenarios_not_full_matrix",
    );
}

#[test]
fn regression_e2e_live_workflow_lane_rejects_missing_external_execution_flag() {
    assert_workflow_failure(
        "            --enable-external-execution \\\n",
        "",
        "sdk_direct_external_execution_flag_missing",
    );
}

#[test]
fn regression_e2e_live_workflow_lane_rejects_sdk_direct_pr_exclusion() {
    assert_workflow_failure(
        "  e2e-sdk-direct:\n    name: E2E SDK-Direct\n    runs-on: ubuntu-latest\n",
        "  e2e-sdk-direct:\n    name: E2E SDK-Direct\n    if: github.event_name != 'pull_request'\n    runs-on: ubuntu-latest\n",
        "sdk_direct_pr_scope_missing",
    );
}

#[test]
fn regression_e2e_live_workflow_lane_rejects_missing_mcp_pr_scope() {
    assert_workflow_failure(
        "if: github.event_name == 'pull_request' || github.event_name == 'schedule' || github.event_name == 'workflow_dispatch'",
        "if: github.event_name == 'schedule'",
        "mcp_agent_pr_scope_missing",
    );
}

#[test]
fn regression_e2e_live_workflow_lane_rejects_missing_pr_skip_reason_markers() {
    assert_workflow_failure(
        "echo \"e2e_cli_smoke_pr_skip_reason_code=none\"\n",
        "",
        "pr_skip_reason_markers_missing",
    );
}

fn assert_workflow_failure(target: &str, replacement: &str, reason: &str) {
    let root = repo_root();
    let workflow = read_file_if_exists(&root.join(".github/workflows/e2e-live.yml"))
        .expect("workflow fixture should exist");
    let strategy =
        read_file_if_exists(&root.join("docs/ci/strategy.md")).expect("strategy fixture exists");
    let mutated = workflow.replacen(target, replacement, 1);
    let decision = evaluate_contract(Some(mutated.as_str()), Some(strategy.as_str()));
    assert_eq!(decision.status, "fail");
    assert_eq!(decision.final_decision, "NO-GO");
    assert_eq!(decision.reason_codes_value, reason);
    assert_eq!(decision.contract_status, "violation");
}
