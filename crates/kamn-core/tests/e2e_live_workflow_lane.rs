use std::path::{Path, PathBuf};

const REASON_TAXONOMY_VERSION: &str = "kamn.ci.e2e-live-workflow-contract-reason-taxonomy.v1";
const REASON_CODES_CSV: &str = "workflow_file_missing,strategy_doc_missing,push_trigger_missing,push_main_branch_scope_missing,pull_request_trigger_missing,sdk_direct_job_missing,sdk_direct_pr_scope_missing,sdk_direct_pr_smoke_selector_missing,sdk_direct_live_toggle_missing,sdk_direct_external_execution_flag_missing,sdk_direct_scenarios_not_full_matrix,mcp_agent_job_missing,mcp_agent_pr_scope_missing,mcp_agent_pr_smoke_selector_missing,kolme_bootstrap_step_missing,kamn_runtime_bootstrap_missing,service_health_wait_marker_missing,cli_smoke_job_missing,cli_smoke_pr_scope_missing,cli_smoke_scenarios_not_smoke_slice,cli_smoke_retry_wrapper_missing,pr_skip_reason_markers_missing,ci_strategy_markers_missing";
const REASON_CODES_ORDER: &[&str] = &[
    "workflow_file_missing",
    "strategy_doc_missing",
    "push_trigger_missing",
    "push_main_branch_scope_missing",
    "pull_request_trigger_missing",
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
// S-11 is intentionally excluded from the blocking live lane while it is stabilized.
const SDK_DIRECT_FULL_SCENARIOS_MARKER: &str =
    "SDK_DIRECT_FULL_SCENARIOS=\"S-01,S-02,S-03,S-04,S-05,S-06,S-07,S-08,S-09,S-10,S-12,S-13,S-14,S-15\"";
const CLI_SMOKE_SCENARIOS: &str = "--scenarios S-01,S-02";
const STRATEGY_REQUIRED_MARKERS: &[&str] = &[
    "## E2E Live Workflow Contract",
    "cargo test -p kamn-core --test e2e_live_workflow_lane",
    "e2e_live_workflow_reason_taxonomy_version=kamn.ci.e2e-live-workflow-contract-reason-taxonomy.v1",
    "e2e_live_workflow_reason_codes_csv=workflow_file_missing,strategy_doc_missing,push_trigger_missing,push_main_branch_scope_missing,pull_request_trigger_missing,sdk_direct_job_missing,sdk_direct_pr_scope_missing,sdk_direct_pr_smoke_selector_missing,sdk_direct_live_toggle_missing,sdk_direct_external_execution_flag_missing,sdk_direct_scenarios_not_full_matrix,mcp_agent_job_missing,mcp_agent_pr_scope_missing,mcp_agent_pr_smoke_selector_missing,kolme_bootstrap_step_missing,kamn_runtime_bootstrap_missing,service_health_wait_marker_missing,cli_smoke_job_missing,cli_smoke_pr_scope_missing,cli_smoke_scenarios_not_smoke_slice,cli_smoke_retry_wrapper_missing,pr_skip_reason_markers_missing,ci_strategy_markers_missing",
    "e2e_live_workflow_contract_status=verified|violation",
    "PR required lanes: e2e-sdk-direct, e2e-mcp-agent, e2e-cli-smoke",
    "e2e_sdk_direct_pr_skip_reason_code=none",
    "e2e_mcp_agent_pr_skip_reason_code=none",
    "e2e_cli_smoke_pr_skip_reason_code=none",
];

#[derive(Debug, Clone, PartialEq, Eq)]
struct ContractDecision {
    status: &'static str,
    final_decision: &'static str,
    reason_taxonomy_version: &'static str,
    reason_codes_csv: &'static str,
    reason_codes_value: String,
    contract_status: &'static str,
}

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("..").join("..")
}

fn read_file_if_exists(path: &Path) -> Option<String> {
    std::fs::read_to_string(path).ok()
}

fn add_reason(reasons: &mut Vec<&'static str>, reason: &'static str) {
    if !reasons.contains(&reason) {
        reasons.push(reason);
    }
}

fn normalize_reasons(observed: Vec<&'static str>) -> Vec<&'static str> {
    REASON_CODES_ORDER
        .iter()
        .copied()
        .filter(|candidate| observed.contains(candidate))
        .collect()
}

fn reason_codes_value(reasons: &[&str]) -> String {
    if reasons.is_empty() {
        return "none".to_owned();
    }
    reasons.join(",")
}

fn sdk_direct_section(workflow: &str) -> Option<&str> {
    let start = workflow.find("  e2e-sdk-direct:")?;
    let mcp_start = workflow[start..]
        .find("  e2e-mcp-agent:")
        .map(|idx| start + idx);
    Some(&workflow[start..mcp_start.unwrap_or(workflow.len())])
}

fn cli_smoke_section(workflow: &str) -> Option<&str> {
    let start = workflow.find("  e2e-cli-smoke:")?;
    Some(&workflow[start..])
}

fn mcp_agent_section(workflow: &str) -> Option<&str> {
    let start = workflow.find("  e2e-mcp-agent:")?;
    let cli_start = workflow[start..]
        .find("  e2e-cli-smoke:")
        .map(|idx| start + idx);
    Some(&workflow[start..cli_start.unwrap_or(workflow.len())])
}

fn evaluate_contract(workflow: Option<&str>, strategy: Option<&str>) -> ContractDecision {
    let mut raw_reasons = Vec::new();

    let workflow_text = match workflow {
        Some(text) => text,
        None => {
            add_reason(&mut raw_reasons, "workflow_file_missing");
            ""
        }
    };
    let strategy_text = match strategy {
        Some(text) => text,
        None => {
            add_reason(&mut raw_reasons, "strategy_doc_missing");
            ""
        }
    };

    let section = if workflow_text.is_empty() {
        None
    } else {
        if !workflow_text.contains("push:") {
            add_reason(&mut raw_reasons, "push_trigger_missing");
        }

        if !workflow_text.contains("branches:") || !workflow_text.contains("- main") {
            add_reason(&mut raw_reasons, "push_main_branch_scope_missing");
        }

        if !workflow_text.contains("pull_request:") {
            add_reason(&mut raw_reasons, "pull_request_trigger_missing");
        }

        let sdk = sdk_direct_section(workflow_text);
        if sdk.is_none() {
            add_reason(&mut raw_reasons, "sdk_direct_job_missing");
        }
        sdk
    };

    if let Some(sdk) = section {
        if sdk.contains("if: github.event_name != 'pull_request'") {
            add_reason(&mut raw_reasons, "sdk_direct_pr_scope_missing");
        }

        if !sdk.contains("SDK_DIRECT_PR_SMOKE_SCENARIOS=\"S-01,S-02\"")
            || !sdk.contains("SDK_DIRECT_SCENARIOS=\"$SDK_DIRECT_PR_SMOKE_SCENARIOS\"")
        {
            add_reason(&mut raw_reasons, "sdk_direct_pr_smoke_selector_missing");
        }

        if !sdk.contains("KAMN_E2E_SDK_DIRECT_LIVE: \"1\"") {
            add_reason(&mut raw_reasons, "sdk_direct_live_toggle_missing");
        }

        if !sdk.contains("--enable-external-execution") {
            add_reason(
                &mut raw_reasons,
                "sdk_direct_external_execution_flag_missing",
            );
        }

        if !sdk.contains(SDK_DIRECT_FULL_SCENARIOS_MARKER) {
            add_reason(&mut raw_reasons, "sdk_direct_scenarios_not_full_matrix");
        }

        if !sdk.contains("git clone https://github.com/fpco/kolme /tmp/kolme")
            || !sdk.contains("/tmp/kolme/target/release/example-p2p")
            || !sdk.contains("api-server")
        {
            add_reason(&mut raw_reasons, "kolme_bootstrap_step_missing");
        }

        if !sdk.contains("--role processor")
            || !sdk.contains("--role listener")
            || !sdk.contains("--role approver")
        {
            add_reason(&mut raw_reasons, "kamn_runtime_bootstrap_missing");
        }

        if !sdk.contains("http://127.0.0.1:8080/healthz")
            || !sdk.contains("http://127.0.0.1:8081/healthz")
            || !sdk.contains("http://127.0.0.1:8082/healthz")
            || !sdk.contains("wait_for_port 127.0.0.1 3000")
            || !sdk.contains("wait_for_http \"http://127.0.0.1:3000/healthz\"")
        {
            add_reason(&mut raw_reasons, "service_health_wait_marker_missing");
        }
    }

    let mcp_section = if workflow_text.is_empty() {
        None
    } else {
        let mcp = mcp_agent_section(workflow_text);
        if mcp.is_none() {
            add_reason(&mut raw_reasons, "mcp_agent_job_missing");
        }
        mcp
    };

    if let Some(mcp) = mcp_section {
        if !mcp.contains("github.event_name == 'pull_request'") {
            add_reason(&mut raw_reasons, "mcp_agent_pr_scope_missing");
        }

        let has_pr_smoke_selector = mcp.contains("MCP_AGENT_PR_SMOKE_SCENARIOS=\"S-01,S-02\"")
            && mcp.contains("MCP_AGENT_SCENARIOS=\"$MCP_AGENT_PR_SMOKE_SCENARIOS\"");
        let has_pr_safe_substitute = mcp
            .contains("MCP_AGENT_PR_SAFE_SUBSTITUTE=\"kamn-mcp-server-contract-smoke\"")
            && mcp.contains("e2e_mcp_agent_pr_safe_substitute=$MCP_AGENT_PR_SAFE_SUBSTITUTE")
            && mcp.contains("cargo test -p kamn-mcp-server --test stdio_protocol_contract")
            && mcp.contains("spec_c03_mcp_tools_call_health_dispatch_contract");

        if !has_pr_smoke_selector && !has_pr_safe_substitute {
            add_reason(&mut raw_reasons, "mcp_agent_pr_smoke_selector_missing");
        }
    }

    let cli_section = if workflow_text.is_empty() {
        None
    } else {
        let cli = cli_smoke_section(workflow_text);
        if cli.is_none() {
            add_reason(&mut raw_reasons, "cli_smoke_job_missing");
        }
        cli
    };

    if let Some(cli) = cli_section {
        if !cli.contains("github.event_name == 'pull_request'") {
            add_reason(&mut raw_reasons, "cli_smoke_pr_scope_missing");
        }

        if !cli.contains(CLI_SMOKE_SCENARIOS) {
            add_reason(&mut raw_reasons, "cli_smoke_scenarios_not_smoke_slice");
        }

        if !cli.contains("bash scripts/ci/run_with_retry.sh")
            || !cli.contains("--label e2e-cli-smoke-live")
            || !cli.contains("--max-attempts 2")
        {
            add_reason(&mut raw_reasons, "cli_smoke_retry_wrapper_missing");
        }
    }

    let pr_skip_reason_markers = [
        "e2e_sdk_direct_pr_skip_reason_code=none",
        "e2e_mcp_agent_pr_skip_reason_code=none",
        "e2e_cli_smoke_pr_skip_reason_code=none",
    ];
    if pr_skip_reason_markers
        .iter()
        .any(|marker| !workflow_text.contains(marker))
    {
        add_reason(&mut raw_reasons, "pr_skip_reason_markers_missing");
    }

    if !strategy_text.is_empty()
        && STRATEGY_REQUIRED_MARKERS
            .iter()
            .any(|marker| !strategy_text.contains(marker))
    {
        add_reason(&mut raw_reasons, "ci_strategy_markers_missing");
    }

    let reasons = normalize_reasons(raw_reasons);
    let status = if reasons.is_empty() { "pass" } else { "fail" };
    let final_decision = if status == "pass" { "GO" } else { "NO-GO" };
    let contract_status = if status == "pass" {
        "verified"
    } else {
        "violation"
    };

    ContractDecision {
        status,
        final_decision,
        reason_taxonomy_version: REASON_TAXONOMY_VERSION,
        reason_codes_csv: REASON_CODES_CSV,
        reason_codes_value: reason_codes_value(&reasons),
        contract_status,
    }
}

#[test]
fn unit_e2e_live_workflow_lane_reason_taxonomy_markers_remain_deterministic() {
    assert_eq!(
        REASON_TAXONOMY_VERSION,
        "kamn.ci.e2e-live-workflow-contract-reason-taxonomy.v1"
    );
    assert_eq!(
        REASON_CODES_CSV,
        "workflow_file_missing,strategy_doc_missing,push_trigger_missing,push_main_branch_scope_missing,pull_request_trigger_missing,sdk_direct_job_missing,sdk_direct_pr_scope_missing,sdk_direct_pr_smoke_selector_missing,sdk_direct_live_toggle_missing,sdk_direct_external_execution_flag_missing,sdk_direct_scenarios_not_full_matrix,mcp_agent_job_missing,mcp_agent_pr_scope_missing,mcp_agent_pr_smoke_selector_missing,kolme_bootstrap_step_missing,kamn_runtime_bootstrap_missing,service_health_wait_marker_missing,cli_smoke_job_missing,cli_smoke_pr_scope_missing,cli_smoke_scenarios_not_smoke_slice,cli_smoke_retry_wrapper_missing,pr_skip_reason_markers_missing,ci_strategy_markers_missing"
    );
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

#[test]
fn regression_e2e_live_workflow_lane_rejects_missing_sdk_direct_live_toggle() {
    let root = repo_root();
    let workflow = read_file_if_exists(&root.join(".github/workflows/e2e-live.yml"))
        .expect("workflow fixture should exist");
    let strategy =
        read_file_if_exists(&root.join("docs/ci/strategy.md")).expect("strategy fixture exists");
    let mutated = workflow.replacen("KAMN_E2E_SDK_DIRECT_LIVE: \"1\"\n", "", 1);
    let decision = evaluate_contract(Some(mutated.as_str()), Some(strategy.as_str()));

    assert_eq!(decision.status, "fail");
    assert_eq!(decision.final_decision, "NO-GO");
    assert_eq!(
        decision.reason_codes_value,
        "sdk_direct_live_toggle_missing"
    );
    assert_eq!(decision.contract_status, "violation");
}

#[test]
fn regression_e2e_live_workflow_lane_rejects_missing_push_trigger() {
    let root = repo_root();
    let workflow = read_file_if_exists(&root.join(".github/workflows/e2e-live.yml"))
        .expect("workflow fixture should exist");
    let strategy =
        read_file_if_exists(&root.join("docs/ci/strategy.md")).expect("strategy fixture exists");
    let mutated = workflow.replacen("  push:\n", "", 1);
    let decision = evaluate_contract(Some(mutated.as_str()), Some(strategy.as_str()));

    assert_eq!(decision.status, "fail");
    assert_eq!(decision.final_decision, "NO-GO");
    assert_eq!(decision.reason_codes_value, "push_trigger_missing");
    assert_eq!(decision.contract_status, "violation");
}

#[test]
fn regression_e2e_live_workflow_lane_rejects_missing_push_main_branch_scope() {
    let root = repo_root();
    let workflow = read_file_if_exists(&root.join(".github/workflows/e2e-live.yml"))
        .expect("workflow fixture should exist");
    let strategy =
        read_file_if_exists(&root.join("docs/ci/strategy.md")).expect("strategy fixture exists");
    let mutated = workflow.replacen("      - main\n", "", 1);
    let decision = evaluate_contract(Some(mutated.as_str()), Some(strategy.as_str()));

    assert_eq!(decision.status, "fail");
    assert_eq!(decision.final_decision, "NO-GO");
    assert_eq!(
        decision.reason_codes_value,
        "push_main_branch_scope_missing"
    );
    assert_eq!(decision.contract_status, "violation");
}

#[test]
fn regression_e2e_live_workflow_lane_rejects_missing_pull_request_trigger() {
    let root = repo_root();
    let workflow = read_file_if_exists(&root.join(".github/workflows/e2e-live.yml"))
        .expect("workflow fixture should exist");
    let strategy =
        read_file_if_exists(&root.join("docs/ci/strategy.md")).expect("strategy fixture exists");
    let mutated = workflow.replacen("  pull_request:\n", "", 1);
    let decision = evaluate_contract(Some(mutated.as_str()), Some(strategy.as_str()));

    assert_eq!(decision.status, "fail");
    assert_eq!(decision.final_decision, "NO-GO");
    assert_eq!(decision.reason_codes_value, "pull_request_trigger_missing");
    assert_eq!(decision.contract_status, "violation");
}

#[test]
fn regression_e2e_live_workflow_lane_rejects_truncated_scenario_matrix() {
    let root = repo_root();
    let workflow = read_file_if_exists(&root.join(".github/workflows/e2e-live.yml"))
        .expect("workflow fixture should exist");
    let strategy =
        read_file_if_exists(&root.join("docs/ci/strategy.md")).expect("strategy fixture exists");
    let mutated = workflow.replacen(
        SDK_DIRECT_FULL_SCENARIOS_MARKER,
        "SDK_DIRECT_FULL_SCENARIOS=\"S-01,S-02,S-03,S-04,S-05,S-06\"",
        1,
    );
    let decision = evaluate_contract(Some(mutated.as_str()), Some(strategy.as_str()));

    assert_eq!(decision.status, "fail");
    assert_eq!(decision.final_decision, "NO-GO");
    assert_eq!(
        decision.reason_codes_value,
        "sdk_direct_scenarios_not_full_matrix"
    );
    assert_eq!(decision.contract_status, "violation");
}

#[test]
fn regression_e2e_live_workflow_lane_rejects_missing_external_execution_flag() {
    let root = repo_root();
    let workflow = read_file_if_exists(&root.join(".github/workflows/e2e-live.yml"))
        .expect("workflow fixture should exist");
    let strategy =
        read_file_if_exists(&root.join("docs/ci/strategy.md")).expect("strategy fixture exists");
    let mutated = workflow.replacen("            --enable-external-execution \\\n", "", 1);
    let decision = evaluate_contract(Some(mutated.as_str()), Some(strategy.as_str()));

    assert_eq!(decision.status, "fail");
    assert_eq!(decision.final_decision, "NO-GO");
    assert_eq!(
        decision.reason_codes_value,
        "sdk_direct_external_execution_flag_missing"
    );
    assert_eq!(decision.contract_status, "violation");
}

#[test]
fn regression_e2e_live_workflow_lane_rejects_sdk_direct_pr_exclusion() {
    let root = repo_root();
    let workflow = read_file_if_exists(&root.join(".github/workflows/e2e-live.yml"))
        .expect("workflow fixture should exist");
    let strategy =
        read_file_if_exists(&root.join("docs/ci/strategy.md")).expect("strategy fixture exists");
    let mutated = workflow.replacen(
        "  e2e-sdk-direct:\n    name: E2E SDK-Direct\n    runs-on: ubuntu-latest\n",
        "  e2e-sdk-direct:\n    name: E2E SDK-Direct\n    if: github.event_name != 'pull_request'\n    runs-on: ubuntu-latest\n",
        1,
    );
    let decision = evaluate_contract(Some(mutated.as_str()), Some(strategy.as_str()));

    assert_eq!(decision.status, "fail");
    assert_eq!(decision.final_decision, "NO-GO");
    assert_eq!(decision.reason_codes_value, "sdk_direct_pr_scope_missing");
    assert_eq!(decision.contract_status, "violation");
}

#[test]
fn regression_e2e_live_workflow_lane_rejects_missing_mcp_pr_scope() {
    let root = repo_root();
    let workflow = read_file_if_exists(&root.join(".github/workflows/e2e-live.yml"))
        .expect("workflow fixture should exist");
    let strategy =
        read_file_if_exists(&root.join("docs/ci/strategy.md")).expect("strategy fixture exists");
    let mutated = workflow.replacen(
        "if: github.event_name == 'pull_request' || github.event_name == 'schedule' || github.event_name == 'workflow_dispatch'",
        "if: github.event_name == 'schedule'",
        1,
    );
    let decision = evaluate_contract(Some(mutated.as_str()), Some(strategy.as_str()));

    assert_eq!(decision.status, "fail");
    assert_eq!(decision.final_decision, "NO-GO");
    assert_eq!(decision.reason_codes_value, "mcp_agent_pr_scope_missing");
    assert_eq!(decision.contract_status, "violation");
}

#[test]
fn regression_e2e_live_workflow_lane_rejects_missing_pr_skip_reason_markers() {
    let root = repo_root();
    let workflow = read_file_if_exists(&root.join(".github/workflows/e2e-live.yml"))
        .expect("workflow fixture should exist");
    let strategy =
        read_file_if_exists(&root.join("docs/ci/strategy.md")).expect("strategy fixture exists");
    let mutated = workflow.replacen("echo \"e2e_cli_smoke_pr_skip_reason_code=none\"\n", "", 1);
    let decision = evaluate_contract(Some(mutated.as_str()), Some(strategy.as_str()));

    assert_eq!(decision.status, "fail");
    assert_eq!(decision.final_decision, "NO-GO");
    assert_eq!(
        decision.reason_codes_value,
        "pr_skip_reason_markers_missing"
    );
    assert_eq!(decision.contract_status, "violation");
}

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
    assert_eq!(
        decision.reason_codes_value,
        "cli_smoke_retry_wrapper_missing"
    );
    assert_eq!(decision.contract_status, "violation");
}
