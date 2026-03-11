use crate::{drivers, scenarios, ExecutionMode, PhaseResultStatus};

use super::super::ScenarioExecutionResult;

pub(super) fn select_scenarios(
    ids: &[String],
) -> Result<Vec<scenarios::ScenarioDefinition>, String> {
    let inventory = scenarios::all_scenarios();
    ids.iter()
        .map(|id| {
            inventory
                .iter()
                .find(|item| item.id == id.as_str())
                .cloned()
                .ok_or_else(|| format!("unknown scenario id: {id}"))
        })
        .collect()
}

pub(super) fn execute_selected_scenarios(
    mode: ExecutionMode,
    selected: &[scenarios::ScenarioDefinition],
    force_first_fail: bool,
) -> Result<Vec<ScenarioExecutionResult>, String> {
    let _env_guard = crate::drivers::test_env_lock()
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let driver = driver_for_mode(mode)?;
    selected
        .iter()
        .enumerate()
        .map(|(index, scenario)| run_scenario(&*driver, scenario, index, force_first_fail))
        .collect()
}

pub(super) fn execute_selected_scenarios_contract_only(
    selected: &[scenarios::ScenarioDefinition],
    force_first_fail: bool,
) -> Vec<ScenarioExecutionResult> {
    selected
        .iter()
        .enumerate()
        .map(|(index, scenario)| ScenarioExecutionResult {
            id: scenario.id.to_owned(),
            status: if force_first_fail && index == 0 {
                PhaseResultStatus::Fail
            } else {
                PhaseResultStatus::Pass
            },
            detail: None,
        })
        .collect()
}

fn run_scenario(
    driver: &dyn drivers::HarnessDriver,
    scenario: &scenarios::ScenarioDefinition,
    index: usize,
    force_first_fail: bool,
) -> Result<ScenarioExecutionResult, String> {
    let driver_result = driver.execute(scenario.id);
    let status = if force_first_fail && index == 0 {
        PhaseResultStatus::Fail
    } else {
        normalize_driver_status(driver_result.status)?
    };
    Ok(ScenarioExecutionResult {
        id: scenario.id.to_owned(),
        status,
        detail: driver_result.detail,
    })
}

fn driver_for_mode(mode: ExecutionMode) -> Result<Box<dyn drivers::HarnessDriver>, String> {
    match mode {
        ExecutionMode::SdkDirect => Ok(Box::new(drivers::sdk_direct::SdkDirectDriver::from_env())),
        ExecutionMode::CliScripted => Ok(Box::new(
            drivers::cli_scripted::CliScriptedDriver::from_env(),
        )),
        ExecutionMode::McpTau | ExecutionMode::McpAny => Ok(Box::new(
            drivers::mcp_agent::McpAgentDriver::from_env(mode)?,
        )),
    }
}

fn normalize_driver_status(value: &str) -> Result<PhaseResultStatus, String> {
    match value.trim().to_ascii_uppercase().as_str() {
        "PASS" => Ok(PhaseResultStatus::Pass),
        "FAIL" => Ok(PhaseResultStatus::Fail),
        "SKIP" => Ok(PhaseResultStatus::Skip),
        other => Err(format!("unsupported driver execution status: {other}")),
    }
}
