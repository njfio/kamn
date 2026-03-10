use crate::support::constants::{
    CENTRALIZED_SERVICE_AUTH_KEY_MARKER, CENTRALIZED_SERVICE_AUTH_PUBLIC_KEY_MARKER,
};
use crate::support::evaluation::evaluate_contract;
use crate::support::fixtures::{read_file_if_exists, repo_root};

#[test]
fn functional_e2e_live_workflow_lane_keeps_centralized_public_key_marker() {
    let root = repo_root();
    let workflow = read_file_if_exists(&root.join(".github/workflows/e2e-live.yml"))
        .expect("workflow fixture should exist");
    assert!(workflow.contains(CENTRALIZED_SERVICE_AUTH_PUBLIC_KEY_MARKER));
}

#[test]
fn regression_e2e_live_workflow_lane_rejects_missing_sdk_direct_live_toggle() {
    assert_single_reason_failure(
        "KAMN_E2E_SDK_DIRECT_LIVE: \"1\"\n",
        "",
        "sdk_direct_live_toggle_missing",
    );
}

#[test]
fn regression_e2e_live_workflow_lane_rejects_missing_centralized_service_auth_key_marker() {
    assert_single_reason_failure(
        CENTRALIZED_SERVICE_AUTH_KEY_MARKER,
        "",
        "centralized_service_auth_key_marker_missing",
    );
}

#[test]
fn regression_e2e_live_workflow_lane_rejects_missing_live_job_timeout() {
    assert_single_reason_failure(
        "    timeout-minutes: 30\n",
        "",
        "live_job_timeout_missing",
    );
}

fn assert_single_reason_failure(target: &str, replacement: &str, reason: &str) {
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
