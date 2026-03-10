use crate::support::evaluation::evaluate_contract;
use crate::support::fixtures::{read_file_if_exists, repo_root};

#[test]
fn regression_e2e_live_workflow_lane_rejects_missing_strategy_markers() {
    let root = repo_root();
    let workflow = read_file_if_exists(&root.join(".github/workflows/e2e-live.yml"))
        .expect("workflow fixture should exist");
    let strategy =
        read_file_if_exists(&root.join("docs/ci/strategy.md")).expect("strategy fixture exists");
    let mutated = strategy.replacen("## E2E Live Workflow Contract\n", "", 1);
    let decision = evaluate_contract(Some(workflow.as_str()), Some(mutated.as_str()));
    assert_eq!(decision.status, "fail");
    assert_eq!(decision.final_decision, "NO-GO");
    assert_eq!(decision.reason_codes_value, "ci_strategy_markers_missing");
    assert_eq!(decision.contract_status, "violation");
}

#[test]
fn regression_e2e_live_workflow_lane_rejects_missing_cli_smoke_retry_wrapper() {
    let root = repo_root();
    let workflow = read_file_if_exists(&root.join(".github/workflows/e2e-live.yml"))
        .expect("workflow fixture should exist");
    let strategy =
        read_file_if_exists(&root.join("docs/ci/strategy.md")).expect("strategy fixture exists");
    let mutated = workflow.replacen("          bash scripts/ci/run_with_retry.sh \\\n", "", 1);
    let decision = evaluate_contract(Some(mutated.as_str()), Some(strategy.as_str()));
    assert_eq!(decision.status, "fail");
    assert_eq!(decision.final_decision, "NO-GO");
    assert_eq!(decision.reason_codes_value, "cli_smoke_retry_wrapper_missing");
    assert_eq!(decision.contract_status, "violation");
}
