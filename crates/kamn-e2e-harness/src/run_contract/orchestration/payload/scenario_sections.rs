use crate::scenarios;

use super::super::super::{escape_json, ScenarioExecutionResult};

pub(super) fn scenario_ids_json(selected: &[scenarios::ScenarioDefinition]) -> String {
    selected
        .iter()
        .map(|item| format!("\"{}\"", item.id))
        .collect::<Vec<_>>()
        .join(",")
}

pub(super) fn scenario_results_json(results: &[ScenarioExecutionResult]) -> String {
    results
        .iter()
        .map(render_scenario_result)
        .collect::<Vec<_>>()
        .join(",")
}

pub(super) fn scenario_contracts_json(
    selected: &[scenarios::ScenarioDefinition],
    results: &[ScenarioExecutionResult],
) -> String {
    selected
        .iter()
        .zip(results.iter())
        .map(|(scenario, result)| render_scenario_contract(scenario, result))
        .collect::<Vec<_>>()
        .join(",")
}

fn render_scenario_result(result: &ScenarioExecutionResult) -> String {
    format!(
        "{{\"id\":\"{}\",\"status\":\"{}\"}}",
        escape_json(result.id.as_str()),
        result.status.as_str()
    )
}

fn render_scenario_contract(
    scenario: &scenarios::ScenarioDefinition,
    result: &ScenarioExecutionResult,
) -> String {
    format!(
        "{{\"id\":\"{}\",\"name\":\"{}\",\"priority\":\"{}\",\"status\":\"{}\",\"steps\":[{}],\"verifiable_outputs\":[{}],\"pass_criteria\":[{}]}}",
        scenario.id,
        escape_json(scenario.name),
        scenario.priority,
        result.status.as_str(),
        quoted_join(scenario.steps),
        quoted_join(scenario.verifiable_outputs),
        quoted_join(scenario.pass_criteria),
    )
}

fn quoted_join(values: &[&str]) -> String {
    values
        .iter()
        .map(|value| format!("\"{}\"", escape_json(value)))
        .collect::<Vec<_>>()
        .join(",")
}
