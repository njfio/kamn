use super::{
    DataLayerPrdCriticalScenarioConformanceError, DataLayerPrdCriticalScenarioResultInput,
    DATA_LAYER_PRD_REQUIRED_CRITICAL_SCENARIO_IDS,
};

pub(crate) fn validate_result_input(
    input: &DataLayerPrdCriticalScenarioResultInput,
) -> Result<(), DataLayerPrdCriticalScenarioConformanceError> {
    validate_required_scenario_id(input.scenario_id)?;
    validate_non_empty(input.evidence_marker.as_str(), "evidence_marker")
}

fn validate_required_scenario_id(
    scenario_id: u8,
) -> Result<(), DataLayerPrdCriticalScenarioConformanceError> {
    if DATA_LAYER_PRD_REQUIRED_CRITICAL_SCENARIO_IDS.contains(&scenario_id) {
        return Ok(());
    }
    Err(DataLayerPrdCriticalScenarioConformanceError::InvalidScenarioId(scenario_id))
}

fn validate_non_empty(
    value: &str,
    field_name: &'static str,
) -> Result<(), DataLayerPrdCriticalScenarioConformanceError> {
    if !value.trim().is_empty() {
        return Ok(());
    }
    Err(DataLayerPrdCriticalScenarioConformanceError::EmptyField(
        field_name,
    ))
}
