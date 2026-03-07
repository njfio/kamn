use std::collections::BTreeMap;

use super::helpers::validate_result_input;
use super::{
    DataLayerPrdCriticalScenarioConformanceDecision, DataLayerPrdCriticalScenarioConformanceError,
    DataLayerPrdCriticalScenarioConformanceReport, DataLayerPrdCriticalScenarioMode,
    DataLayerPrdCriticalScenarioResultInput, DataLayerPrdCriticalScenarioResultRecord,
    DATA_LAYER_PRD_CRITICAL_SCENARIO_CONFORMANT_REASON_CODE,
    DATA_LAYER_PRD_CRITICAL_SCENARIO_FAILED_REASON_CODE,
    DATA_LAYER_PRD_CRITICAL_SCENARIO_INVALID_MUTATION_REASON_CODE,
    DATA_LAYER_PRD_CRITICAL_SCENARIO_MISSING_REASON_CODE,
    DATA_LAYER_PRD_CRITICAL_SCENARIO_SHELL_POLICY_REASON_CODE,
    DATA_LAYER_PRD_REQUIRED_CRITICAL_SCENARIO_IDS,
};

/// In-memory deterministic registry for PRD critical-scenario conformance.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct DataLayerPrdCriticalScenarioConformanceMatrix {
    scenario_results: BTreeMap<u8, DataLayerPrdCriticalScenarioResultRecord>,
}

impl DataLayerPrdCriticalScenarioConformanceMatrix {
    /// Creates an empty conformance matrix.
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns required PRD critical scenario IDs in deterministic order.
    pub fn required_scenario_ids(&self) -> Vec<u8> {
        DATA_LAYER_PRD_REQUIRED_CRITICAL_SCENARIO_IDS.to_vec()
    }

    /// Records one critical scenario result.
    pub fn record_result(
        &mut self,
        input: DataLayerPrdCriticalScenarioResultInput,
    ) -> Result<
        DataLayerPrdCriticalScenarioResultRecord,
        DataLayerPrdCriticalScenarioConformanceError,
    > {
        validate_result_input(&input)?;
        if let Some(existing) = self.scenario_results.get(&input.scenario_id) {
            return ensure_result_immutable(existing, &input);
        }

        let record = build_result_record(input);
        self.scenario_results
            .insert(record.scenario_id, record.clone());
        Ok(record)
    }

    /// Evaluates matrix conformance against required scenario completeness and policy.
    pub fn evaluate_conformance(
        &self,
    ) -> Result<
        DataLayerPrdCriticalScenarioConformanceReport,
        DataLayerPrdCriticalScenarioConformanceError,
    > {
        Ok(self.build_tally().into_report())
    }

    fn build_tally(&self) -> ConformanceTally {
        let mut tally = ConformanceTally::new();
        for scenario_id in DATA_LAYER_PRD_REQUIRED_CRITICAL_SCENARIO_IDS {
            tally.record_required_scenario(scenario_id, self.scenario_results.get(&scenario_id));
        }
        tally
    }
}

#[derive(Default)]
struct ConformanceTally {
    missing_scenario_ids: Vec<u8>,
    failed_scenario_ids: Vec<u8>,
    shell_policy_violation_scenario_ids: Vec<u8>,
    passed_required_scenarios: u8,
    total_required_scenarios: u8,
}

impl ConformanceTally {
    fn new() -> Self {
        Self {
            total_required_scenarios: DATA_LAYER_PRD_REQUIRED_CRITICAL_SCENARIO_IDS.len() as u8,
            ..Self::default()
        }
    }

    fn record_required_scenario(
        &mut self,
        scenario_id: u8,
        record: Option<&DataLayerPrdCriticalScenarioResultRecord>,
    ) {
        match record {
            Some(record) => self.record_existing_result(scenario_id, record),
            None => self.missing_scenario_ids.push(scenario_id),
        }
    }

    fn record_existing_result(
        &mut self,
        scenario_id: u8,
        record: &DataLayerPrdCriticalScenarioResultRecord,
    ) {
        if record.passed {
            self.passed_required_scenarios = self.passed_required_scenarios.saturating_add(1);
        } else {
            self.failed_scenario_ids.push(scenario_id);
        }

        if record.orchestration_mode != DataLayerPrdCriticalScenarioMode::RustOnly {
            self.shell_policy_violation_scenario_ids.push(scenario_id);
        }
    }

    fn into_report(self) -> DataLayerPrdCriticalScenarioConformanceReport {
        let (decision, reason_codes) = self.resolve_decision();
        DataLayerPrdCriticalScenarioConformanceReport {
            decision,
            reason_codes,
            missing_scenario_ids: self.missing_scenario_ids,
            failed_scenario_ids: self.failed_scenario_ids,
            shell_policy_violation_scenario_ids: self.shell_policy_violation_scenario_ids,
            total_required_scenarios: self.total_required_scenarios,
            passed_required_scenarios: self.passed_required_scenarios,
        }
    }

    fn resolve_decision(
        &self,
    ) -> (
        DataLayerPrdCriticalScenarioConformanceDecision,
        Vec<&'static str>,
    ) {
        if !self.shell_policy_violation_scenario_ids.is_empty() {
            return non_conformant(DATA_LAYER_PRD_CRITICAL_SCENARIO_SHELL_POLICY_REASON_CODE);
        }
        if !self.failed_scenario_ids.is_empty() {
            return non_conformant(DATA_LAYER_PRD_CRITICAL_SCENARIO_FAILED_REASON_CODE);
        }
        if !self.missing_scenario_ids.is_empty() {
            return non_conformant(DATA_LAYER_PRD_CRITICAL_SCENARIO_MISSING_REASON_CODE);
        }
        (
            DataLayerPrdCriticalScenarioConformanceDecision::Conformant,
            vec![DATA_LAYER_PRD_CRITICAL_SCENARIO_CONFORMANT_REASON_CODE],
        )
    }
}

fn ensure_result_immutable(
    existing: &DataLayerPrdCriticalScenarioResultRecord,
    input: &DataLayerPrdCriticalScenarioResultInput,
) -> Result<DataLayerPrdCriticalScenarioResultRecord, DataLayerPrdCriticalScenarioConformanceError>
{
    if existing.passed == input.passed && existing.orchestration_mode == input.orchestration_mode {
        return Ok(existing.clone());
    }
    Err(
        DataLayerPrdCriticalScenarioConformanceError::InvalidResultMutation {
            scenario_id: input.scenario_id,
            existing_passed: existing.passed,
            requested_passed: input.passed,
            existing_mode: existing.orchestration_mode,
            requested_mode: input.orchestration_mode,
            reason_code: DATA_LAYER_PRD_CRITICAL_SCENARIO_INVALID_MUTATION_REASON_CODE,
        },
    )
}

fn build_result_record(
    input: DataLayerPrdCriticalScenarioResultInput,
) -> DataLayerPrdCriticalScenarioResultRecord {
    DataLayerPrdCriticalScenarioResultRecord {
        scenario_id: input.scenario_id,
        passed: input.passed,
        orchestration_mode: input.orchestration_mode,
        evidence_marker: input.evidence_marker,
    }
}

fn non_conformant(
    reason_code: &'static str,
) -> (
    DataLayerPrdCriticalScenarioConformanceDecision,
    Vec<&'static str>,
) {
    (
        DataLayerPrdCriticalScenarioConformanceDecision::NonConformant,
        vec![reason_code],
    )
}
