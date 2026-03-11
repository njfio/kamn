use crate::{all_orchestration_phases, LifecycleStatusTotals, OrchestrationPhaseResult};

use super::super::super::escape_json;

pub(super) fn phase_labels_json() -> String {
    all_orchestration_phases()
        .iter()
        .map(|phase| format!("\"{}\"", phase.as_str()))
        .collect::<Vec<_>>()
        .join(",")
}

pub(super) fn phase_results_json(results: &[OrchestrationPhaseResult]) -> String {
    results
        .iter()
        .map(render_phase)
        .collect::<Vec<_>>()
        .join(",")
}

pub(super) fn totals_json(totals: &LifecycleStatusTotals) -> String {
    format!(
        "{{\"total\":{},\"pass\":{},\"fail\":{},\"skip\":{}}}",
        totals.total, totals.pass, totals.fail, totals.skip
    )
}

fn render_phase(result: &OrchestrationPhaseResult) -> String {
    format!(
        "{{\"phase\":\"{}\",\"status\":\"{}\",\"started_at\":\"{}\",\"completed_at\":\"{}\",\"details\":\"{}\",\"steps\":[{}]}}",
        result.phase.as_str(),
        result.status.as_str(),
        escape_json(result.started_at.as_str()),
        escape_json(result.completed_at.as_str()),
        escape_json(result.details.as_str()),
        render_steps(result)
    )
}

fn render_steps(result: &OrchestrationPhaseResult) -> String {
    result
        .steps
        .iter()
        .map(|step| {
            format!(
                "{{\"step\":\"{}\",\"status\":\"{}\",\"detail\":\"{}\"}}",
                escape_json(step.step.as_str()),
                step.status.as_str(),
                escape_json(step.detail.as_str())
            )
        })
        .collect::<Vec<_>>()
        .join(",")
}
