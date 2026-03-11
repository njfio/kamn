use crate::{ExecutionMode, OrchestrationPhase, PhaseResultStatus};

use super::super::super::{is_mcp_mode, ScenarioExecutionResult};
use super::status_totals_from_iter;

pub(super) fn phase_details(
    phase: OrchestrationPhase,
    mode: ExecutionMode,
    status: PhaseResultStatus,
    scenario_results: &[ScenarioExecutionResult],
    evidence_status: PhaseResultStatus,
) -> String {
    match (phase, status) {
        (OrchestrationPhase::InfraUp, PhaseResultStatus::Fail) => {
            "deterministic fail-path marker for infra startup".to_owned()
        }
        (OrchestrationPhase::InfraUp, _) => {
            "deterministic placeholder for infra startup".to_owned()
        }
        (OrchestrationPhase::AgentDeploy, _) => {
            "deterministic placeholder for agent deploy".to_owned()
        }
        (OrchestrationPhase::ScenarioRun, _) => scenario_detail(scenario_results),
        (OrchestrationPhase::Evidence, _) => evidence_phase_detail(evidence_status),
        (OrchestrationPhase::Teardown, _) => teardown_phase_detail(mode, status),
    }
}

fn scenario_detail(scenario_results: &[ScenarioExecutionResult]) -> String {
    let totals = status_totals_from_iter(scenario_results.iter().map(|result| result.status));
    format!(
        "deterministic scenario execution summary: executed={} pass={} fail={} skip={}",
        totals.total, totals.pass, totals.fail, totals.skip
    )
}

fn evidence_phase_detail(status: PhaseResultStatus) -> String {
    match status {
        PhaseResultStatus::Fail => {
            "deterministic evidence phase summary: steps_total=6 pass=2 fail=4 skip=0 expected_artifacts=4 recorded_artifacts=3 status=FAIL".to_owned()
        }
        PhaseResultStatus::Pass => {
            "deterministic evidence phase summary: steps_total=6 pass=6 fail=0 skip=0 expected_artifacts=4 recorded_artifacts=4 status=PASS".to_owned()
        }
        PhaseResultStatus::Skip => {
            "deterministic evidence phase summary: steps_total=6 pass=0 fail=0 skip=6 expected_artifacts=4 recorded_artifacts=0 status=SKIP".to_owned()
        }
    }
}

fn teardown_phase_detail(mode: ExecutionMode, status: PhaseResultStatus) -> String {
    let mcp_step_status = if is_mcp_mode(mode) { "PASS" } else { "SKIP" };
    format!(
        "deterministic teardown summary: mcp_stop={} kamn_nodes=PASS kolme=PASS postgres=PASS archive=PASS status={}",
        mcp_step_status,
        status.as_str()
    )
}
