use crate::{ExecutionMode, OrchestrationStepRecord, PhaseResultStatus};

use super::super::super::super::is_mcp_mode;

pub(super) fn teardown_steps(mode: ExecutionMode) -> Vec<OrchestrationStepRecord> {
    let mut steps = base_teardown_steps();
    apply_mcp_teardown_mode(mode, &mut steps);
    steps
}

fn step_record(step: &str, status: PhaseResultStatus, detail: &str) -> OrchestrationStepRecord {
    OrchestrationStepRecord {
        step: step.to_owned(),
        status,
        detail: detail.to_owned(),
    }
}

fn base_teardown_steps() -> Vec<OrchestrationStepRecord> {
    vec![
        step_record(
            "[MCP modes] Stop kamn-mcp-server processes",
            PhaseResultStatus::Pass,
            "deterministic placeholder: mcp servers stopped",
        ),
        teardown_step("Stop KAMN nodes (graceful shutdown)", "kamn nodes stopped"),
        teardown_step("Stop Kolme devnet", "kolme devnet stopped"),
        teardown_step("Stop PostgreSQL container", "postgres container stopped"),
        teardown_step("Archive evidence bundle", "evidence bundle archived"),
    ]
}

fn teardown_step(step: &str, action: &str) -> OrchestrationStepRecord {
    step_record(
        step,
        PhaseResultStatus::Pass,
        format!("deterministic placeholder: {action}").as_str(),
    )
}

fn apply_mcp_teardown_mode(mode: ExecutionMode, steps: &mut [OrchestrationStepRecord]) {
    if is_mcp_mode(mode) {
        return;
    }
    steps[0].status = PhaseResultStatus::Skip;
    steps[0].detail = "deterministic placeholder: mcp teardown skipped for non-mcp mode".to_owned();
}
