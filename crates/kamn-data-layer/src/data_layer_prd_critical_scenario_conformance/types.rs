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
/// Stable reason marker for invalid or mutating scenario record updates.
pub const DATA_LAYER_PRD_CRITICAL_SCENARIO_INVALID_MUTATION_REASON_CODE: &str =
    "prd_critical_scenario_invalid_mutation";

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
    /// Scenario pass or fail result.
    pub passed: bool,
    /// Scenario orchestration mode.
    pub orchestration_mode: DataLayerPrdCriticalScenarioMode,
    /// Deterministic evidence marker or path.
    pub evidence_marker: String,
}

/// Recorded scenario result row.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataLayerPrdCriticalScenarioResultRecord {
    /// PRD scenario identifier (`62..71`).
    pub scenario_id: u8,
    /// Scenario pass or fail result.
    pub passed: bool,
    /// Scenario orchestration mode.
    pub orchestration_mode: DataLayerPrdCriticalScenarioMode,
    /// Deterministic evidence marker or path.
    pub evidence_marker: String,
}

/// Conformance decision output.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DataLayerPrdCriticalScenarioConformanceDecision {
    /// Matrix satisfies completeness, pass, and policy constraints.
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
