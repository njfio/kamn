use crate::support::constants::{REASON_CODES_CSV, REASON_TAXONOMY_VERSION};
use crate::support::evaluation::evaluate_contract;
use crate::support::fixtures::{read_file_if_exists, repo_root};

#[test]
fn unit_e2e_live_workflow_lane_reason_taxonomy_markers_remain_deterministic() {
    assert_eq!(
        REASON_TAXONOMY_VERSION,
        "kamn.ci.e2e-live-workflow-contract-reason-taxonomy.v1"
    );
    assert_eq!(REASON_CODES_CSV, "workflow_file_missing,strategy_doc_missing,push_trigger_missing,push_main_branch_scope_missing,pull_request_trigger_missing,centralized_service_auth_key_marker_missing,duplicated_service_auth_key_setup_present,live_job_timeout_missing,sdk_direct_job_missing,sdk_direct_pr_scope_missing,sdk_direct_pr_smoke_selector_missing,sdk_direct_live_toggle_missing,sdk_direct_external_execution_flag_missing,sdk_direct_scenarios_not_full_matrix,mcp_agent_job_missing,mcp_agent_pr_scope_missing,mcp_agent_pr_smoke_selector_missing,kolme_bootstrap_step_missing,kamn_runtime_bootstrap_missing,service_health_wait_marker_missing,cli_smoke_job_missing,cli_smoke_pr_scope_missing,cli_smoke_scenarios_not_smoke_slice,cli_smoke_retry_wrapper_missing,pr_skip_reason_markers_missing,ci_strategy_markers_missing");
}

#[test]
fn functional_e2e_live_workflow_lane_accepts_repository_baseline() {
    let root = repo_root();
    let workflow = read_file_if_exists(&root.join(".github/workflows/e2e-live.yml"));
    let strategy = read_file_if_exists(&root.join("docs/ci/strategy.md"));
    let decision = evaluate_contract(workflow.as_deref(), strategy.as_deref());
    assert_eq!(decision.status, "pass");
    assert_eq!(decision.final_decision, "GO");
    assert_eq!(decision.reason_taxonomy_version, REASON_TAXONOMY_VERSION);
    assert_eq!(decision.reason_codes_csv, REASON_CODES_CSV);
    assert_eq!(decision.reason_codes_value, "none");
    assert_eq!(decision.contract_status, "verified");
}
