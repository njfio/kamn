pub(crate) const REASON_TAXONOMY_VERSION: &str =
    "kamn.ci.e2e-live-workflow-contract-reason-taxonomy.v1";
pub(crate) const REASON_CODES_CSV: &str = "workflow_file_missing,strategy_doc_missing,push_trigger_missing,push_main_branch_scope_missing,pull_request_trigger_missing,centralized_service_auth_key_marker_missing,duplicated_service_auth_key_setup_present,live_job_timeout_missing,sdk_direct_job_missing,sdk_direct_pr_scope_missing,sdk_direct_pr_smoke_selector_missing,sdk_direct_live_toggle_missing,sdk_direct_external_execution_flag_missing,sdk_direct_scenarios_not_full_matrix,mcp_agent_job_missing,mcp_agent_pr_scope_missing,mcp_agent_pr_smoke_selector_missing,kolme_bootstrap_step_missing,kamn_runtime_bootstrap_missing,service_health_wait_marker_missing,cli_smoke_job_missing,cli_smoke_pr_scope_missing,cli_smoke_scenarios_not_smoke_slice,cli_smoke_retry_wrapper_missing,pr_skip_reason_markers_missing,ci_strategy_markers_missing";
pub(crate) const REASON_CODES_ORDER: &[&str] = &[
    "workflow_file_missing",
    "strategy_doc_missing",
    "push_trigger_missing",
    "push_main_branch_scope_missing",
    "pull_request_trigger_missing",
    "centralized_service_auth_key_marker_missing",
    "duplicated_service_auth_key_setup_present",
    "live_job_timeout_missing",
    "sdk_direct_job_missing",
    "sdk_direct_pr_scope_missing",
    "sdk_direct_pr_smoke_selector_missing",
    "sdk_direct_live_toggle_missing",
    "sdk_direct_external_execution_flag_missing",
    "sdk_direct_scenarios_not_full_matrix",
    "mcp_agent_job_missing",
    "mcp_agent_pr_scope_missing",
    "mcp_agent_pr_smoke_selector_missing",
    "kolme_bootstrap_step_missing",
    "kamn_runtime_bootstrap_missing",
    "service_health_wait_marker_missing",
    "cli_smoke_job_missing",
    "cli_smoke_pr_scope_missing",
    "cli_smoke_scenarios_not_smoke_slice",
    "cli_smoke_retry_wrapper_missing",
    "pr_skip_reason_markers_missing",
    "ci_strategy_markers_missing",
];
pub(crate) const SDK_DIRECT_FULL_SCENARIOS_MARKER: &str =
    "SDK_DIRECT_FULL_SCENARIOS=\"S-01,S-02,S-03,S-04,S-05,S-06,S-07,S-08,S-09,S-10,S-12,S-13,S-14,S-15\"";
pub(crate) const CLI_SMOKE_SCENARIOS: &str = "--scenarios S-01,S-02";
pub(crate) const CENTRALIZED_SERVICE_AUTH_KEY_MARKER: &str =
    "  KAMN_E2E_SERVICE_AUTH_PRIVATE_KEY_HEX: \"658c3528422eb527b4c108b8f6d1e5f629543c304ea49cf608c67794424291c4\"";
pub(crate) const CENTRALIZED_SERVICE_AUTH_PUBLIC_KEY_MARKER: &str =
    "  KAMN_E2E_SERVICE_AUTH_PUBLIC_KEY_HEX: \"0264eb26609d15e709227b9ddc46c11a738b210bb237949aa86d7d490a35ae0f0a\"";
pub(crate) const DUPLICATED_INLINE_SERVICE_AUTH_KEY_MARKER: &str =
    "SERVICE_AUTH_PRIVATE_KEY_HEX=\"658c3528422eb527b4c108b8f6d1e5f629543c304ea49cf608c67794424291c4\"";
pub(crate) const STRATEGY_REQUIRED_MARKERS: &[&str] = &[
    "## E2E Live Workflow Contract",
    "cargo test -p kamn-core --test e2e_live_workflow_lane",
    "e2e_live_workflow_reason_taxonomy_version=kamn.ci.e2e-live-workflow-contract-reason-taxonomy.v1",
    "e2e_live_workflow_reason_codes_csv=workflow_file_missing,strategy_doc_missing,push_trigger_missing,push_main_branch_scope_missing,pull_request_trigger_missing,centralized_service_auth_key_marker_missing,duplicated_service_auth_key_setup_present,live_job_timeout_missing,sdk_direct_job_missing,sdk_direct_pr_scope_missing,sdk_direct_pr_smoke_selector_missing,sdk_direct_live_toggle_missing,sdk_direct_external_execution_flag_missing,sdk_direct_scenarios_not_full_matrix,mcp_agent_job_missing,mcp_agent_pr_scope_missing,mcp_agent_pr_smoke_selector_missing,kolme_bootstrap_step_missing,kamn_runtime_bootstrap_missing,service_health_wait_marker_missing,cli_smoke_job_missing,cli_smoke_pr_scope_missing,cli_smoke_scenarios_not_smoke_slice,cli_smoke_retry_wrapper_missing,pr_skip_reason_markers_missing,ci_strategy_markers_missing",
    "e2e_live_workflow_contract_status=verified|violation",
    "PR required lanes: e2e-sdk-direct, e2e-mcp-agent, e2e-cli-smoke",
    "e2e_live_workflow_service_auth_key_source=workflow_env_centralized",
    "e2e_live_workflow_job_timeout_minutes=30,30,30",
    "e2e_sdk_direct_pr_skip_reason_code=none",
    "e2e_mcp_agent_pr_skip_reason_code=none",
    "e2e_cli_smoke_pr_skip_reason_code=none",
];
