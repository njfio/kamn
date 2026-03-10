use crate::support::evaluation::evaluate_contract;
use crate::support::fixtures::{strategy_fixture, workflow_fixture};

#[test]
fn regression_e2e_live_workflow_lane_rejects_missing_push_trigger() {
    assert_scope_failure("  push:\n", "", "push_trigger_missing");
}

#[test]
fn regression_e2e_live_workflow_lane_rejects_missing_push_main_branch_scope() {
    assert_scope_failure("      - main\n", "", "push_main_branch_scope_missing");
}

#[test]
fn regression_e2e_live_workflow_lane_rejects_missing_pull_request_trigger() {
    assert_scope_failure("  pull_request:\n", "", "pull_request_trigger_missing");
}

fn assert_scope_failure(target: &str, replacement: &str, reason: &str) {
    let workflow = workflow_fixture();
    let strategy = strategy_fixture();
    let mutated = workflow.replacen(target, replacement, 1);
    let decision = evaluate_contract(Some(mutated.as_str()), Some(strategy.as_str()));
    assert_eq!(decision.status, "fail");
    assert_eq!(decision.final_decision, "NO-GO");
    assert_eq!(decision.reason_codes_value, reason);
    assert_eq!(decision.contract_status, "violation");
}
