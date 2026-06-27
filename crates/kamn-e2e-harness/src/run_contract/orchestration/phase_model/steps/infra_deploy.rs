use crate::{ExecutionMode, OrchestrationStepRecord, PhaseResultStatus};

use super::super::super::super::is_mcp_mode;

pub(super) fn infra_steps(fail_path_marker: bool) -> Vec<OrchestrationStepRecord> {
    let mut steps = base_steps(INFRA_LABELS, PhaseResultStatus::Pass, INFRA_DETAILS);
    if let Some(last) = steps.last_mut() {
        last.status = if fail_path_marker {
            PhaseResultStatus::Fail
        } else {
            PhaseResultStatus::Pass
        };
        last.detail = if fail_path_marker {
            "deterministic fail-path marker: kamn health check failed".to_owned()
        } else {
            "deterministic placeholder: kamn health verified".to_owned()
        };
    }
    steps
}

pub(super) fn deploy_steps(mode: ExecutionMode) -> Vec<OrchestrationStepRecord> {
    let mut steps = base_steps(DEPLOY_LABELS, PhaseResultStatus::Pass, DEPLOY_DETAILS);
    apply_mcp_skip(
        mode,
        &mut steps[3],
        "deterministic placeholder: mcp server spawn skipped for non-mcp mode",
    );
    apply_mcp_skip(
        mode,
        &mut steps[4],
        "deterministic placeholder: mcp health skipped for non-mcp mode",
    );
    steps
}

fn apply_mcp_skip(mode: ExecutionMode, step: &mut OrchestrationStepRecord, skip_detail: &str) {
    if !is_mcp_mode(mode) {
        step.status = PhaseResultStatus::Skip;
        step.detail = skip_detail.to_owned();
    }
}

fn base_steps(
    labels: &[&str],
    status: PhaseResultStatus,
    details: &[&str],
) -> Vec<OrchestrationStepRecord> {
    labels
        .iter()
        .zip(details.iter())
        .map(|(step, detail)| OrchestrationStepRecord {
            step: (*step).to_owned(),
            status,
            detail: (*detail).to_owned(),
        })
        .collect()
}

const INFRA_LABELS: &[&str] = &[
    "Start PostgreSQL container (docker)",
    "Run Kolme migrations",
    "Start Kolme processor (in-memory or Fjall storage)",
    "Verify Kolme API health (/healthz)",
    "Start KAMN processor node",
    "Start KAMN listener node",
    "Start KAMN approver node",
    "Wait for peer discovery (3 connected peers)",
    "Verify KAMN Service API health (/healthz)",
];
const INFRA_DETAILS: &[&str] = &[
    "deterministic placeholder: postgres startup",
    "deterministic placeholder: migrations complete",
    "deterministic placeholder: kolme processor online",
    "deterministic placeholder: kolme health verified",
    "deterministic placeholder: processor node online",
    "deterministic placeholder: listener node online",
    "deterministic placeholder: approver node online",
    "deterministic placeholder: peer discovery complete",
    "deterministic placeholder: kamn health verified",
];
const DEPLOY_LABELS: &[&str] = &[
    "Generate ed25519 key pairs for Alice, Bob, Carol",
    "Write key files to temp directory",
    "Register agents via kamn-agent-lib (POST /v1/agents/bootstrap)",
    "[MCP modes] Spawn kamn-mcp-server per agent with identity",
    "[MCP modes] Verify MCP server health",
    "Record infrastructure evidence",
];
const DEPLOY_DETAILS: &[&str] = &[
    "deterministic placeholder: keys generated",
    "deterministic placeholder: key files materialized",
    "deterministic placeholder: agents registered",
    "deterministic placeholder: mcp servers spawned",
    "deterministic placeholder: mcp health verified",
    "deterministic placeholder: infra evidence recorded",
];
