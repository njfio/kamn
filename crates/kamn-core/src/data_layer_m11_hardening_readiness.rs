//! M11 hardening matrix contracts for security, chaos, performance, and operator readiness.
//!
//! This module provides deterministic contracts for scenario registration,
//! outcome recording, and fail-closed operator readiness decisions.

use std::collections::BTreeMap;
use std::fmt;

/// Stable reason marker when readiness evaluates to GO.
pub const DATA_LAYER_M11_READINESS_GO_REASON_CODE: &str = "m11_operator_readiness_go";
/// Stable reason marker when readiness is blocked by critical failures.
pub const DATA_LAYER_M11_BLOCK_CRITICAL_FAILURE_REASON_CODE: &str = "m11_blocking_critical_failure";
/// Stable reason marker when readiness is blocked by missing/incomplete required scenarios.
pub const DATA_LAYER_M11_BLOCK_REQUIRED_INCOMPLETE_REASON_CODE: &str =
    "m11_blocking_required_incomplete";
/// Stable reason marker for illegal scenario status transitions.
pub const DATA_LAYER_M11_INVALID_TRANSITION_REASON_CODE: &str = "m11_invalid_status_transition";

/// Hardening scenario domain categories for M11.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum DataLayerM11ScenarioDomain {
    /// Security and authorization/cryptography checks.
    Security,
    /// Chaos and resilience/fault-injection checks.
    Chaos,
    /// Performance and latency/throughput checks.
    Performance,
    /// Operator and runbook/process readiness checks.
    Operations,
}

/// Hardening scenario severity level.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum DataLayerM11ScenarioSeverity {
    /// Critical severity blocks production readiness on failure.
    Critical,
    /// High severity should be addressed before production rollout.
    High,
    /// Medium severity indicates bounded but material risk.
    Medium,
    /// Low severity indicates non-blocking risk.
    Low,
}

/// Execution status for one hardening scenario.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum DataLayerM11ScenarioStatus {
    /// Scenario completed successfully.
    Passed,
    /// Scenario failed and requires remediation.
    Failed,
    /// Scenario was skipped and must be treated as incomplete if required.
    Skipped,
}

/// Scenario registration input/definition.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataLayerM11ScenarioDefinition {
    /// Stable scenario identifier.
    pub scenario_id: String,
    /// Scenario domain.
    pub domain: DataLayerM11ScenarioDomain,
    /// Scenario severity.
    pub severity: DataLayerM11ScenarioSeverity,
    /// Whether this scenario is required for operator readiness.
    pub required: bool,
}

/// Scenario outcome recording input.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataLayerM11ScenarioOutcomeInput {
    /// Stable scenario identifier.
    pub scenario_id: String,
    /// Result status for the scenario execution.
    pub status: DataLayerM11ScenarioStatus,
    /// Deterministic evidence marker/path.
    pub evidence_marker: String,
}

/// Persisted scenario outcome record.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataLayerM11ScenarioOutcomeRecord {
    /// Stable scenario identifier.
    pub scenario_id: String,
    /// Result status for the scenario execution.
    pub status: DataLayerM11ScenarioStatus,
    /// Deterministic evidence marker/path.
    pub evidence_marker: String,
}

/// Operator readiness decision outcome.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DataLayerM11OperatorReadinessDecision {
    /// Ready for operator rollout.
    Go,
    /// Not ready for operator rollout.
    NoGo,
}

/// Readiness projection with deterministic reasons.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataLayerM11OperatorReadinessReport {
    /// Final readiness decision.
    pub decision: DataLayerM11OperatorReadinessDecision,
    /// Deterministic reason markers driving the decision.
    pub reason_codes: Vec<&'static str>,
    /// Required scenario ids with missing outcomes.
    pub missing_required_scenario_ids: Vec<String>,
    /// Critical-severity scenarios that reported failure.
    pub failing_critical_scenario_ids: Vec<String>,
    /// Number of required scenarios in the registry.
    pub total_required_scenarios: u16,
    /// Number of required scenarios with `Passed` outcomes.
    pub passed_required_scenarios: u16,
}

/// Fail-closed error taxonomy for M11 hardening matrix contracts.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DataLayerM11HardeningMatrixError {
    /// Required field is empty.
    EmptyField(&'static str),
    /// Duplicate scenario identifier.
    DuplicateScenarioId(String),
    /// Scenario identifier does not exist in the matrix.
    ScenarioNotFound(String),
    /// Matrix has no registered scenarios.
    EmptyScenarioCatalog,
    /// Illegal status transition for an already-recorded scenario outcome.
    InvalidStatusTransition {
        /// Scenario identifier.
        scenario_id: String,
        /// Existing status.
        from_status: DataLayerM11ScenarioStatus,
        /// Requested status.
        to_status: DataLayerM11ScenarioStatus,
        /// Stable transition reason marker.
        reason_code: &'static str,
    },
}

impl fmt::Display for DataLayerM11HardeningMatrixError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyField(field_name) => write!(formatter, "{field_name} must not be empty"),
            Self::DuplicateScenarioId(scenario_id) => {
                write!(formatter, "duplicate scenario id {scenario_id}")
            }
            Self::ScenarioNotFound(scenario_id) => write!(formatter, "scenario not found: {scenario_id}"),
            Self::EmptyScenarioCatalog => write!(formatter, "scenario catalog must not be empty"),
            Self::InvalidStatusTransition {
                scenario_id,
                from_status,
                to_status,
                reason_code,
            } => write!(
                formatter,
                "invalid status transition for {scenario_id}: {from_status:?}->{to_status:?} ({reason_code})"
            ),
        }
    }
}

impl std::error::Error for DataLayerM11HardeningMatrixError {}

/// Deterministic in-memory registry for M11 hardening scenario contracts.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct DataLayerM11HardeningMatrix {
    scenario_catalog: BTreeMap<String, DataLayerM11ScenarioDefinition>,
    scenario_outcomes: BTreeMap<String, DataLayerM11ScenarioOutcomeRecord>,
}

impl DataLayerM11HardeningMatrix {
    /// Creates an empty hardening matrix.
    pub fn new() -> Self {
        Self::default()
    }

    /// Registers one hardening scenario definition.
    pub fn register_scenario(
        &mut self,
        definition: DataLayerM11ScenarioDefinition,
    ) -> Result<DataLayerM11ScenarioDefinition, DataLayerM11HardeningMatrixError> {
        validate_non_empty(definition.scenario_id.as_str(), "scenario_id")?;
        if self
            .scenario_catalog
            .contains_key(definition.scenario_id.as_str())
        {
            return Err(DataLayerM11HardeningMatrixError::DuplicateScenarioId(
                definition.scenario_id,
            ));
        }
        self.scenario_catalog
            .insert(definition.scenario_id.clone(), definition.clone());
        Ok(definition)
    }

    /// Lists registered scenarios in deterministic identifier order.
    pub fn list_scenarios(&self) -> Vec<DataLayerM11ScenarioDefinition> {
        self.scenario_catalog.values().cloned().collect()
    }

    /// Records one scenario outcome.
    pub fn record_outcome(
        &mut self,
        input: DataLayerM11ScenarioOutcomeInput,
    ) -> Result<DataLayerM11ScenarioOutcomeRecord, DataLayerM11HardeningMatrixError> {
        validate_non_empty(input.scenario_id.as_str(), "scenario_id")?;
        validate_non_empty(input.evidence_marker.as_str(), "evidence_marker")?;

        if !self
            .scenario_catalog
            .contains_key(input.scenario_id.as_str())
        {
            return Err(DataLayerM11HardeningMatrixError::ScenarioNotFound(
                input.scenario_id,
            ));
        }

        if let Some(existing) = self.scenario_outcomes.get(input.scenario_id.as_str()) {
            if existing.status != input.status {
                return Err(DataLayerM11HardeningMatrixError::InvalidStatusTransition {
                    scenario_id: input.scenario_id,
                    from_status: existing.status,
                    to_status: input.status,
                    reason_code: DATA_LAYER_M11_INVALID_TRANSITION_REASON_CODE,
                });
            }
        }

        let record = DataLayerM11ScenarioOutcomeRecord {
            scenario_id: input.scenario_id.clone(),
            status: input.status,
            evidence_marker: input.evidence_marker,
        };
        self.scenario_outcomes
            .insert(input.scenario_id, record.clone());
        Ok(record)
    }

    /// Evaluates overall operator readiness from required scenario outcomes.
    pub fn evaluate_operator_readiness(
        &self,
    ) -> Result<DataLayerM11OperatorReadinessReport, DataLayerM11HardeningMatrixError> {
        if self.scenario_catalog.is_empty() {
            return Err(DataLayerM11HardeningMatrixError::EmptyScenarioCatalog);
        }

        let required_scenarios: Vec<&DataLayerM11ScenarioDefinition> = self
            .scenario_catalog
            .values()
            .filter(|scenario| scenario.required)
            .collect();

        let mut missing_required_scenario_ids = Vec::new();
        let mut failing_critical_scenario_ids = Vec::new();
        let mut passed_required_scenarios = 0u16;

        for scenario in &required_scenarios {
            match self.scenario_outcomes.get(scenario.scenario_id.as_str()) {
                Some(outcome) => {
                    if outcome.status == DataLayerM11ScenarioStatus::Passed {
                        passed_required_scenarios = passed_required_scenarios.saturating_add(1);
                    }
                    if scenario.severity == DataLayerM11ScenarioSeverity::Critical
                        && outcome.status == DataLayerM11ScenarioStatus::Failed
                    {
                        failing_critical_scenario_ids.push(scenario.scenario_id.clone());
                    }
                }
                None => missing_required_scenario_ids.push(scenario.scenario_id.clone()),
            }
        }

        let total_required_scenarios = u16::try_from(required_scenarios.len()).unwrap_or(u16::MAX);

        if !failing_critical_scenario_ids.is_empty() {
            return Ok(DataLayerM11OperatorReadinessReport {
                decision: DataLayerM11OperatorReadinessDecision::NoGo,
                reason_codes: vec![DATA_LAYER_M11_BLOCK_CRITICAL_FAILURE_REASON_CODE],
                missing_required_scenario_ids,
                failing_critical_scenario_ids,
                total_required_scenarios,
                passed_required_scenarios,
            });
        }

        if !missing_required_scenario_ids.is_empty()
            || passed_required_scenarios != total_required_scenarios
        {
            return Ok(DataLayerM11OperatorReadinessReport {
                decision: DataLayerM11OperatorReadinessDecision::NoGo,
                reason_codes: vec![DATA_LAYER_M11_BLOCK_REQUIRED_INCOMPLETE_REASON_CODE],
                missing_required_scenario_ids,
                failing_critical_scenario_ids,
                total_required_scenarios,
                passed_required_scenarios,
            });
        }

        Ok(DataLayerM11OperatorReadinessReport {
            decision: DataLayerM11OperatorReadinessDecision::Go,
            reason_codes: vec![DATA_LAYER_M11_READINESS_GO_REASON_CODE],
            missing_required_scenario_ids,
            failing_critical_scenario_ids,
            total_required_scenarios,
            passed_required_scenarios,
        })
    }
}

fn validate_non_empty(
    value: &str,
    field_name: &'static str,
) -> Result<(), DataLayerM11HardeningMatrixError> {
    if value.trim().is_empty() {
        return Err(DataLayerM11HardeningMatrixError::EmptyField(field_name));
    }
    Ok(())
}
