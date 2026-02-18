//! PRD critical-scenario conformance contracts (`18.2`, scenarios `62..71`).
//!
//! This module provides a deterministic, fail-closed contract for recording
//! scenario outcomes and evaluating whether the critical scenario matrix is
//! conformant under shell-neutral orchestration policy.

use std::collections::BTreeMap;
use std::fmt;

/// Stable reason marker for fully conformant critical scenario matrix.
pub const DATA_LAYER_PRD_CRITICAL_SCENARIO_CONFORMANT_REASON_CODE: &str =
    "prd_critical_scenario_matrix_conformant";
/// Stable reason marker when at least one required scenario failed.
pub const DATA_LAYER_PRD_CRITICAL_SCENARIO_FAILED_REASON_CODE: &str =
    "prd_critical_scenario_failed";
/// Stable reason marker when required scenario outcomes are missing.
pub const DATA_LAYER_PRD_CRITICAL_SCENARIO_MISSING_REASON_CODE: &str =
    "prd_critical_scenario_missing";
/// Stable reason marker when non-rust orchestration mode is detected.
pub const DATA_LAYER_PRD_CRITICAL_SCENARIO_SHELL_POLICY_REASON_CODE: &str =
    "prd_critical_scenario_shell_policy_violation";
/// Stable reason marker for invalid/mutating scenario record updates.
pub const DATA_LAYER_PRD_CRITICAL_SCENARIO_INVALID_MUTATION_REASON_CODE: &str =
    "prd_critical_scenario_invalid_mutation";

const DATA_LAYER_PRD_REQUIRED_CRITICAL_SCENARIO_IDS: [u8; 10] =
    [62, 63, 64, 65, 66, 67, 68, 69, 70, 71];

/// Orchestration mode used to execute a scenario.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DataLayerPrdCriticalScenarioMode {
    /// Rust-first orchestration policy-compliant mode.
    RustOnly,
    /// Shell-assisted mode; policy violation for critical scenarios.
    ShellHybrid,
}

/// Scenario result input payload.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataLayerPrdCriticalScenarioResultInput {
    /// PRD scenario identifier (`62..71`).
    pub scenario_id: u8,
    /// Scenario pass/fail result.
    pub passed: bool,
    /// Scenario orchestration mode.
    pub orchestration_mode: DataLayerPrdCriticalScenarioMode,
    /// Deterministic evidence marker/path.
    pub evidence_marker: String,
}

/// Recorded scenario result row.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataLayerPrdCriticalScenarioResultRecord {
    /// PRD scenario identifier (`62..71`).
    pub scenario_id: u8,
    /// Scenario pass/fail result.
    pub passed: bool,
    /// Scenario orchestration mode.
    pub orchestration_mode: DataLayerPrdCriticalScenarioMode,
    /// Deterministic evidence marker/path.
    pub evidence_marker: String,
}

/// Conformance decision output.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DataLayerPrdCriticalScenarioConformanceDecision {
    /// Matrix satisfies completeness/pass/policy constraints.
    Conformant,
    /// Matrix does not satisfy one or more constraints.
    NonConformant,
}

/// Conformance evaluation projection.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataLayerPrdCriticalScenarioConformanceReport {
    /// Final conformance decision.
    pub decision: DataLayerPrdCriticalScenarioConformanceDecision,
    /// Deterministic reason markers explaining the decision.
    pub reason_codes: Vec<&'static str>,
    /// Required scenario IDs with no recorded result.
    pub missing_scenario_ids: Vec<u8>,
    /// Required scenario IDs with `passed=false`.
    pub failed_scenario_ids: Vec<u8>,
    /// Required scenario IDs with non-rust orchestration mode.
    pub shell_policy_violation_scenario_ids: Vec<u8>,
    /// Number of required scenarios.
    pub total_required_scenarios: u8,
    /// Number of required scenarios with `passed=true`.
    pub passed_required_scenarios: u8,
}

/// Fail-closed error taxonomy for critical-scenario conformance contracts.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DataLayerPrdCriticalScenarioConformanceError {
    /// Required field was empty.
    EmptyField(&'static str),
    /// Scenario id is not within required set (`62..71`).
    InvalidScenarioId(u8),
    /// Attempted update mutates a previously recorded result.
    InvalidResultMutation {
        /// Scenario identifier being updated.
        scenario_id: u8,
        /// Existing `passed` value.
        existing_passed: bool,
        /// Requested `passed` value.
        requested_passed: bool,
        /// Existing mode.
        existing_mode: DataLayerPrdCriticalScenarioMode,
        /// Requested mode.
        requested_mode: DataLayerPrdCriticalScenarioMode,
        /// Stable reason marker.
        reason_code: &'static str,
    },
}

impl fmt::Display for DataLayerPrdCriticalScenarioConformanceError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyField(field_name) => write!(formatter, "{field_name} must not be empty"),
            Self::InvalidScenarioId(scenario_id) => {
                write!(formatter, "invalid PRD critical scenario id: {scenario_id}")
            }
            Self::InvalidResultMutation {
                scenario_id,
                existing_passed,
                requested_passed,
                existing_mode,
                requested_mode,
                reason_code,
            } => write!(
                formatter,
                "invalid result mutation for scenario {scenario_id}: passed({existing_passed}->{requested_passed}) mode({existing_mode:?}->{requested_mode:?}) ({reason_code})"
            ),
        }
    }
}

impl std::error::Error for DataLayerPrdCriticalScenarioConformanceError {}

/// In-memory deterministic registry for PRD critical scenario conformance.
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
        validate_required_scenario_id(input.scenario_id)?;
        validate_non_empty(input.evidence_marker.as_str(), "evidence_marker")?;

        if let Some(existing) = self.scenario_results.get(&input.scenario_id) {
            if existing.passed != input.passed
                || existing.orchestration_mode != input.orchestration_mode
            {
                return Err(
                    DataLayerPrdCriticalScenarioConformanceError::InvalidResultMutation {
                        scenario_id: input.scenario_id,
                        existing_passed: existing.passed,
                        requested_passed: input.passed,
                        existing_mode: existing.orchestration_mode,
                        requested_mode: input.orchestration_mode,
                        reason_code: DATA_LAYER_PRD_CRITICAL_SCENARIO_INVALID_MUTATION_REASON_CODE,
                    },
                );
            }
            return Ok(existing.clone());
        }

        let record = DataLayerPrdCriticalScenarioResultRecord {
            scenario_id: input.scenario_id,
            passed: input.passed,
            orchestration_mode: input.orchestration_mode,
            evidence_marker: input.evidence_marker,
        };
        self.scenario_results
            .insert(input.scenario_id, record.clone());
        Ok(record)
    }

    /// Evaluates matrix conformance against required scenario completeness + policy.
    pub fn evaluate_conformance(
        &self,
    ) -> Result<
        DataLayerPrdCriticalScenarioConformanceReport,
        DataLayerPrdCriticalScenarioConformanceError,
    > {
        let mut missing_scenario_ids = Vec::new();
        let mut failed_scenario_ids = Vec::new();
        let mut shell_policy_violation_scenario_ids = Vec::new();
        let mut passed_required_scenarios = 0u8;

        for scenario_id in self.required_scenario_ids() {
            match self.scenario_results.get(&scenario_id) {
                Some(record) => {
                    if record.passed {
                        passed_required_scenarios = passed_required_scenarios.saturating_add(1);
                    } else {
                        failed_scenario_ids.push(scenario_id);
                    }
                    if record.orchestration_mode != DataLayerPrdCriticalScenarioMode::RustOnly {
                        shell_policy_violation_scenario_ids.push(scenario_id);
                    }
                }
                None => missing_scenario_ids.push(scenario_id),
            }
        }

        let total_required_scenarios = DATA_LAYER_PRD_REQUIRED_CRITICAL_SCENARIO_IDS.len() as u8;

        if !shell_policy_violation_scenario_ids.is_empty() {
            return Ok(DataLayerPrdCriticalScenarioConformanceReport {
                decision: DataLayerPrdCriticalScenarioConformanceDecision::NonConformant,
                reason_codes: vec![DATA_LAYER_PRD_CRITICAL_SCENARIO_SHELL_POLICY_REASON_CODE],
                missing_scenario_ids,
                failed_scenario_ids,
                shell_policy_violation_scenario_ids,
                total_required_scenarios,
                passed_required_scenarios,
            });
        }

        if !failed_scenario_ids.is_empty() {
            return Ok(DataLayerPrdCriticalScenarioConformanceReport {
                decision: DataLayerPrdCriticalScenarioConformanceDecision::NonConformant,
                reason_codes: vec![DATA_LAYER_PRD_CRITICAL_SCENARIO_FAILED_REASON_CODE],
                missing_scenario_ids,
                failed_scenario_ids,
                shell_policy_violation_scenario_ids,
                total_required_scenarios,
                passed_required_scenarios,
            });
        }

        if !missing_scenario_ids.is_empty() {
            return Ok(DataLayerPrdCriticalScenarioConformanceReport {
                decision: DataLayerPrdCriticalScenarioConformanceDecision::NonConformant,
                reason_codes: vec![DATA_LAYER_PRD_CRITICAL_SCENARIO_MISSING_REASON_CODE],
                missing_scenario_ids,
                failed_scenario_ids,
                shell_policy_violation_scenario_ids,
                total_required_scenarios,
                passed_required_scenarios,
            });
        }

        Ok(DataLayerPrdCriticalScenarioConformanceReport {
            decision: DataLayerPrdCriticalScenarioConformanceDecision::Conformant,
            reason_codes: vec![DATA_LAYER_PRD_CRITICAL_SCENARIO_CONFORMANT_REASON_CODE],
            missing_scenario_ids,
            failed_scenario_ids,
            shell_policy_violation_scenario_ids,
            total_required_scenarios,
            passed_required_scenarios,
        })
    }
}

fn validate_required_scenario_id(
    scenario_id: u8,
) -> Result<(), DataLayerPrdCriticalScenarioConformanceError> {
    if !DATA_LAYER_PRD_REQUIRED_CRITICAL_SCENARIO_IDS.contains(&scenario_id) {
        return Err(DataLayerPrdCriticalScenarioConformanceError::InvalidScenarioId(scenario_id));
    }
    Ok(())
}

fn validate_non_empty(
    value: &str,
    field_name: &'static str,
) -> Result<(), DataLayerPrdCriticalScenarioConformanceError> {
    if value.trim().is_empty() {
        return Err(DataLayerPrdCriticalScenarioConformanceError::EmptyField(
            field_name,
        ));
    }
    Ok(())
}
