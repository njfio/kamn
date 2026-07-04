use crate::support::constants::{
    CENTRALIZED_SERVICE_AUTH_KEY_MARKER, CLI_SMOKE_SCENARIOS,
    DUPLICATED_INLINE_SERVICE_AUTH_KEY_MARKER, SDK_DIRECT_FULL_SCENARIOS_MARKER,
};
use crate::support::evaluation::contract_decision::add_reason;
use crate::support::fixtures::count_occurrences;
use crate::support::sections::{cli_smoke_section, mcp_agent_section, sdk_direct_section};

pub(crate) fn evaluate_workflow_markers(workflow: &str, reasons: &mut Vec<&'static str>) {
    if workflow.is_empty() {
        return;
    }
    ensure_contains(workflow, "push:", reasons, "push_trigger_missing");
    ensure_main_branch_scope(workflow, reasons);
    ensure_contains(
        workflow,
        "pull_request:",
        reasons,
        "pull_request_trigger_missing",
    );
    ensure_contains(
        workflow,
        CENTRALIZED_SERVICE_AUTH_KEY_MARKER,
        reasons,
        "centralized_service_auth_key_marker_missing",
    );
    if count_occurrences(workflow, DUPLICATED_INLINE_SERVICE_AUTH_KEY_MARKER) > 0 {
        add_reason(reasons, "duplicated_service_auth_key_setup_present");
    }
}

pub(crate) fn evaluate_sdk_markers(workflow: &str, reasons: &mut Vec<&'static str>) {
    let Some(section) = required_section(
        workflow,
        sdk_direct_section(workflow),
        reasons,
        "sdk_direct_job_missing",
    ) else {
        return;
    };
    ensure_timeout(section, reasons);
    if section.contains("if: github.event_name != 'pull_request'") {
        add_reason(reasons, "sdk_direct_pr_scope_missing");
    }
    ensure_sdk_smoke_selector(section, reasons);
    ensure_sdk_runtime_markers(section, reasons);
}

pub(crate) fn evaluate_mcp_markers(workflow: &str, reasons: &mut Vec<&'static str>) {
    let Some(section) = required_section(
        workflow,
        mcp_agent_section(workflow),
        reasons,
        "mcp_agent_job_missing",
    ) else {
        return;
    };
    ensure_timeout(section, reasons);
    ensure_contains(
        section,
        "github.event_name == 'pull_request'",
        reasons,
        "mcp_agent_pr_scope_missing",
    );
    if !has_mcp_pr_safe_selector(section) {
        add_reason(reasons, "mcp_agent_pr_smoke_selector_missing");
    }
}

pub(crate) fn evaluate_cli_markers(workflow: &str, reasons: &mut Vec<&'static str>) {
    let Some(section) = required_section(
        workflow,
        cli_smoke_section(workflow),
        reasons,
        "cli_smoke_job_missing",
    ) else {
        return;
    };
    ensure_timeout(section, reasons);
    ensure_contains(
        section,
        "github.event_name == 'pull_request'",
        reasons,
        "cli_smoke_pr_scope_missing",
    );
    ensure_contains(
        section,
        CLI_SMOKE_SCENARIOS,
        reasons,
        "cli_smoke_scenarios_not_smoke_slice",
    );
    ensure_cli_retry_wrapper(section, reasons);
}

fn ensure_main_branch_scope(workflow: &str, reasons: &mut Vec<&'static str>) {
    if !workflow.contains("branches:") || !workflow.contains("- main") {
        add_reason(reasons, "push_main_branch_scope_missing");
    }
}

fn ensure_timeout(section: &str, reasons: &mut Vec<&'static str>) {
    ensure_contains(
        section,
        "timeout-minutes: 30",
        reasons,
        "live_job_timeout_missing",
    );
}

fn ensure_sdk_smoke_selector(section: &str, reasons: &mut Vec<&'static str>) {
    let has_smoke = section.contains("SDK_DIRECT_PR_SMOKE_SCENARIOS=\"S-01,S-02\"")
        && section.contains("SDK_DIRECT_SCENARIOS=\"$SDK_DIRECT_PR_SMOKE_SCENARIOS\"");
    if !has_smoke {
        add_reason(reasons, "sdk_direct_pr_smoke_selector_missing");
    }
}

fn ensure_sdk_runtime_markers(section: &str, reasons: &mut Vec<&'static str>) {
    ensure_contains(
        section,
        "KAMN_E2E_SDK_DIRECT_LIVE: \"1\"",
        reasons,
        "sdk_direct_live_toggle_missing",
    );
    ensure_contains(
        section,
        "--enable-external-execution",
        reasons,
        "sdk_direct_external_execution_flag_missing",
    );
    ensure_contains(
        section,
        SDK_DIRECT_FULL_SCENARIOS_MARKER,
        reasons,
        "sdk_direct_scenarios_not_full_matrix",
    );
    ensure_kolme_bootstrap(section, reasons);
    ensure_runtime_bootstrap(section, reasons);
    ensure_service_health(section, reasons);
}

fn ensure_kolme_bootstrap(section: &str, reasons: &mut Vec<&'static str>) {
    let has_bootstrap = section.contains("git clone https://github.com/fpco/kolme /tmp/kolme")
        && section.contains("/tmp/kolme/target/release/example-p2p")
        && section.contains("api-server");
    if !has_bootstrap {
        add_reason(reasons, "kolme_bootstrap_step_missing");
    }
}

fn ensure_runtime_bootstrap(section: &str, reasons: &mut Vec<&'static str>) {
    let has_runtime = section.contains("--role processor")
        && section.contains("--role listener")
        && section.contains("--role approver");
    if !has_runtime {
        add_reason(reasons, "kamn_runtime_bootstrap_missing");
    }
}

fn ensure_service_health(section: &str, reasons: &mut Vec<&'static str>) {
    let has_health = section.contains("http://127.0.0.1:8080/healthz")
        && section.contains("http://127.0.0.1:8081/healthz")
        && section.contains("http://127.0.0.1:8082/healthz")
        && section.contains("wait_for_port 127.0.0.1 3000")
        && section.contains("wait_for_http \"http://127.0.0.1:3000/healthz\"");
    if !has_health {
        add_reason(reasons, "service_health_wait_marker_missing");
    }
}

fn has_mcp_pr_safe_selector(section: &str) -> bool {
    let smoke_selector = section.contains("MCP_AGENT_PR_SMOKE_SCENARIOS=\"S-01,S-02\"")
        && section.contains("MCP_AGENT_SCENARIOS=\"$MCP_AGENT_PR_SMOKE_SCENARIOS\"");
    let safe_substitute = section
        .contains("MCP_AGENT_PR_SAFE_SUBSTITUTE=\"kamn-mcp-server-contract-smoke\"")
        && section.contains("e2e_mcp_agent_pr_safe_substitute=$MCP_AGENT_PR_SAFE_SUBSTITUTE")
        && section.contains("cargo test -p kamn-mcp-server --test stdio_protocol_contract")
        && section.contains("spec_c03_mcp_tools_call_health_dispatch_contract");
    smoke_selector || safe_substitute
}

fn ensure_cli_retry_wrapper(section: &str, reasons: &mut Vec<&'static str>) {
    let has_wrapper = section.contains("bash scripts/ci/run_with_retry.sh")
        && section.contains("--label e2e-cli-smoke-live")
        && section.contains("--max-attempts 2");
    if !has_wrapper {
        add_reason(reasons, "cli_smoke_retry_wrapper_missing");
    }
}

fn required_section<'a>(
    workflow: &'a str,
    section: Option<&'a str>,
    reasons: &mut Vec<&'static str>,
    missing: &'static str,
) -> Option<&'a str> {
    if workflow.is_empty() {
        return None;
    }
    if section.is_none() {
        add_reason(reasons, missing);
    }
    section
}

fn ensure_contains(
    section: &str,
    marker: &str,
    reasons: &mut Vec<&'static str>,
    reason: &'static str,
) {
    if !section.contains(marker) {
        add_reason(reasons, reason);
    }
}
