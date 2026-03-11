mod infra_deploy;
mod scenario_evidence;
mod teardown;

use crate::{ExecutionMode, OrchestrationPhase, OrchestrationStepRecord, PhaseResultStatus};

use super::super::super::ScenarioExecutionResult;

pub(super) fn phase_step_records(
    phase: OrchestrationPhase,
    mode: ExecutionMode,
    fail_path_marker: bool,
    scenario_results: &[ScenarioExecutionResult],
    evidence_status: PhaseResultStatus,
) -> Vec<OrchestrationStepRecord> {
    match phase {
        OrchestrationPhase::InfraUp => infra_deploy::infra_steps(fail_path_marker),
        OrchestrationPhase::AgentDeploy => infra_deploy::deploy_steps(mode),
        OrchestrationPhase::ScenarioRun => scenario_evidence::scenario_steps(scenario_results),
        OrchestrationPhase::Evidence => scenario_evidence::evidence_steps(evidence_status),
        OrchestrationPhase::Teardown => teardown::teardown_steps(mode),
    }
}
