use crate::{
    DataLayerM11OperatorReadinessDecision, DataLayerM11OperatorReadinessReport,
    DataLayerPrdCriticalScenarioConformanceDecision, DataLayerPrdCriticalScenarioConformanceReport,
};

/// Stable reason marker when closure evidence satisfies all release gates.
pub const DATA_LAYER_M11_CLOSURE_ACCEPTED_REASON_CODE: &str = "m11_closure_accepted";
/// Stable reason marker when hardening readiness blocks closure.
pub const DATA_LAYER_M11_CLOSURE_BLOCK_HARDENING_REASON_CODE: &str = "m11_closure_block_hardening";
/// Stable reason marker when critical scenario conformance blocks closure.
pub const DATA_LAYER_M11_CLOSURE_BLOCK_CRITICAL_SCENARIO_REASON_CODE: &str =
    "m11_closure_block_critical_scenario";
/// Stable reason marker when performance/signoff evidence is incomplete.
pub const DATA_LAYER_M11_CLOSURE_BLOCK_EVIDENCE_GAP_REASON_CODE: &str =
    "m11_closure_block_evidence_gap";

/// Acceptance decision for M11 closure evidence.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DataLayerM11ClosureAcceptanceDecision {
    /// Release closure is accepted.
    Accepted,
    /// Release closure is rejected.
    Rejected,
}

/// Input envelope for closure evidence evaluation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataLayerM11ClosureEvidenceInput {
    /// Release marker for the closure report.
    pub release_marker: String,
    /// M11 hardening readiness report.
    pub hardening_report: DataLayerM11OperatorReadinessReport,
    /// PRD critical-scenario conformance report.
    pub critical_scenario_report: DataLayerPrdCriticalScenarioConformanceReport,
    /// Performance budget closure gate.
    pub performance_budget_met: bool,
    /// Security signoff closure gate.
    pub security_signoff_complete: bool,
    /// Chaos signoff closure gate.
    pub chaos_signoff_complete: bool,
}

/// Deterministic closure acceptance report.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataLayerM11ClosureEvidenceReport {
    /// Release marker for this report.
    pub release_marker: String,
    /// Final acceptance decision.
    pub decision: DataLayerM11ClosureAcceptanceDecision,
    /// Stable reason markers for the decision.
    pub reason_codes: Vec<&'static str>,
    /// Projected hardening decision.
    pub hardening_decision: DataLayerM11OperatorReadinessDecision,
    /// Projected critical scenario conformance decision.
    pub critical_scenario_decision: DataLayerPrdCriticalScenarioConformanceDecision,
    /// Performance gate marker.
    pub performance_budget_met: bool,
    /// Security signoff gate marker.
    pub security_signoff_complete: bool,
    /// Chaos signoff gate marker.
    pub chaos_signoff_complete: bool,
}
