use crate::support::constants::STRATEGY_REQUIRED_MARKERS;
use crate::support::evaluation::contract_decision::{add_reason, build_decision, ContractDecision};
use crate::support::evaluation::section_logic::{
    evaluate_cli_markers, evaluate_mcp_markers, evaluate_sdk_markers, evaluate_workflow_markers,
};

pub(crate) fn evaluate_contract(
    workflow: Option<&str>,
    strategy: Option<&str>,
) -> ContractDecision {
    let mut reasons = Vec::new();
    let workflow_text = workflow_text(workflow, &mut reasons);
    let strategy_text = strategy_text(strategy, &mut reasons);
    evaluate_workflow_markers(workflow_text, &mut reasons);
    evaluate_sdk_markers(workflow_text, &mut reasons);
    evaluate_mcp_markers(workflow_text, &mut reasons);
    evaluate_cli_markers(workflow_text, &mut reasons);
    evaluate_pr_skip_markers(workflow_text, &mut reasons);
    evaluate_strategy_markers(strategy_text, &mut reasons);
    build_decision(reasons)
}

fn workflow_text<'a>(workflow: Option<&'a str>, reasons: &mut Vec<&'static str>) -> &'a str {
    match workflow {
        Some(text) => text,
        None => {
            add_reason(reasons, "workflow_file_missing");
            ""
        }
    }
}

fn strategy_text<'a>(strategy: Option<&'a str>, reasons: &mut Vec<&'static str>) -> &'a str {
    match strategy {
        Some(text) => text,
        None => {
            add_reason(reasons, "strategy_doc_missing");
            ""
        }
    }
}

fn evaluate_pr_skip_markers(workflow: &str, reasons: &mut Vec<&'static str>) {
    let markers = [
        "e2e_sdk_direct_pr_skip_reason_code=none",
        "e2e_mcp_agent_pr_skip_reason_code=none",
        "e2e_cli_smoke_pr_skip_reason_code=none",
    ];
    if markers.iter().any(|marker| !workflow.contains(marker)) {
        add_reason(reasons, "pr_skip_reason_markers_missing");
    }
}

fn evaluate_strategy_markers(strategy: &str, reasons: &mut Vec<&'static str>) {
    if strategy.is_empty() {
        return;
    }
    if STRATEGY_REQUIRED_MARKERS.iter().any(|marker| !strategy.contains(marker)) {
        add_reason(reasons, "ci_strategy_markers_missing");
    }
}
