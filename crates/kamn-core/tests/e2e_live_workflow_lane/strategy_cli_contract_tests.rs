use crate::support::evaluation::evaluate_contract;
use crate::support::fixtures::{strategy_fixture, workflow_fixture};

#[test]
fn regression_e2e_live_workflow_lane_rejects_missing_strategy_markers() {
    let workflow = workflow_fixture();
    let strategy = strategy_fixture();
    let mutated = strategy.replacen("## E2E Live Workflow Contract\n", "", 1);
    let decision = evaluate_contract(Some(workflow.as_str()), Some(mutated.as_str()));
    assert_eq!(decision.status, "fail");
    assert_eq!(decision.final_decision, "NO-GO");
    assert_eq!(decision.reason_codes_value, "ci_strategy_markers_missing");
    assert_eq!(decision.contract_status, "violation");
}

#[test]
fn regression_e2e_live_workflow_lane_rejects_missing_cli_smoke_retry_wrapper() {
    let workflow = workflow_fixture();
    let strategy = strategy_fixture();
    let mutated = workflow.replacen("          bash scripts/ci/run_with_retry.sh \\\n", "", 1);
    let decision = evaluate_contract(Some(mutated.as_str()), Some(strategy.as_str()));
    assert_eq!(decision.status, "fail");
    assert_eq!(decision.final_decision, "NO-GO");
    assert_eq!(
        decision.reason_codes_value,
        "cli_smoke_retry_wrapper_missing"
    );
    assert_eq!(decision.contract_status, "violation");
}
