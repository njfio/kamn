mod details;
mod steps;

use crate::{
    ExecutionMode, LifecycleStatusTotals, LifecycleSummary, OrchestrationPhase,
    OrchestrationPhaseResult, OrchestrationStepRecord, PhaseResultStatus,
};

use super::super::ScenarioExecutionResult;

pub(crate) fn aggregate_status(statuses: &[PhaseResultStatus]) -> PhaseResultStatus {
    if statuses.contains(&PhaseResultStatus::Fail) {
        return PhaseResultStatus::Fail;
    }
    if statuses
        .iter()
        .all(|status| *status == PhaseResultStatus::Skip)
    {
        return PhaseResultStatus::Skip;
    }
    PhaseResultStatus::Pass
}

pub(super) fn compute_lifecycle_summary(
    phase_results: &[OrchestrationPhaseResult],
) -> LifecycleSummary {
    LifecycleSummary {
        phase_totals: status_totals_from_iter(phase_results.iter().map(|result| result.status)),
        step_totals: status_totals_from_iter(
            phase_results
                .iter()
                .flat_map(|result| result.steps.iter().map(|step| step.status)),
        ),
    }
}

pub(super) fn status_totals_from_iter<I>(statuses: I) -> LifecycleStatusTotals
where
    I: IntoIterator<Item = PhaseResultStatus>,
{
    let mut totals = LifecycleStatusTotals {
        total: 0,
        pass: 0,
        fail: 0,
        skip: 0,
    };
    for status in statuses {
        totals.total += 1;
        match status {
            PhaseResultStatus::Pass => totals.pass += 1,
            PhaseResultStatus::Fail => totals.fail += 1,
            PhaseResultStatus::Skip => totals.skip += 1,
        }
    }
    totals
}

pub(super) fn build_phase_results(
    phases: &[OrchestrationPhase],
    mode: ExecutionMode,
    fail_path_marker: bool,
    scenario_results: &[ScenarioExecutionResult],
    evidence_status: PhaseResultStatus,
) -> Vec<OrchestrationPhaseResult> {
    phases
        .iter()
        .map(|phase| {
            build_phase_result(
                *phase,
                mode,
                fail_path_marker,
                scenario_results,
                evidence_status,
            )
        })
        .collect()
}

fn build_phase_result(
    phase: OrchestrationPhase,
    mode: ExecutionMode,
    fail_path_marker: bool,
    scenario_results: &[ScenarioExecutionResult],
    evidence_status: PhaseResultStatus,
) -> OrchestrationPhaseResult {
    let steps = steps::phase_step_records(
        phase,
        mode,
        fail_path_marker,
        scenario_results,
        evidence_status,
    );
    let status = phase_status_for_steps(steps.as_slice());
    OrchestrationPhaseResult {
        phase,
        status,
        started_at: "1970-01-01T00:00:00Z".to_owned(),
        completed_at: "1970-01-01T00:00:01Z".to_owned(),
        details: details::phase_details(phase, mode, status, scenario_results, evidence_status),
        steps,
    }
}

fn phase_status_for_steps(steps: &[OrchestrationStepRecord]) -> PhaseResultStatus {
    if steps
        .iter()
        .any(|step| step.status == PhaseResultStatus::Fail)
    {
        return PhaseResultStatus::Fail;
    }
    if steps
        .iter()
        .all(|step| step.status == PhaseResultStatus::Skip)
    {
        return PhaseResultStatus::Skip;
    }
    PhaseResultStatus::Pass
}
