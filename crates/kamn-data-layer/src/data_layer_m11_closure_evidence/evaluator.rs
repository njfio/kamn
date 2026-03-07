use crate::{
    DataLayerM11OperatorReadinessDecision, DataLayerPrdCriticalScenarioConformanceDecision,
};

use super::{
    DataLayerM11ClosureAcceptanceDecision, DataLayerM11ClosureEvidenceError,
    DataLayerM11ClosureEvidenceInput, DataLayerM11ClosureEvidenceReport,
    DATA_LAYER_M11_CLOSURE_ACCEPTED_REASON_CODE,
    DATA_LAYER_M11_CLOSURE_BLOCK_CRITICAL_SCENARIO_REASON_CODE,
    DATA_LAYER_M11_CLOSURE_BLOCK_EVIDENCE_GAP_REASON_CODE,
    DATA_LAYER_M11_CLOSURE_BLOCK_HARDENING_REASON_CODE,
};

/// Evaluates closure evidence and projects deterministic acceptance output.
pub fn data_layer_m11_evaluate_closure_evidence(
    input: DataLayerM11ClosureEvidenceInput,
) -> Result<DataLayerM11ClosureEvidenceReport, DataLayerM11ClosureEvidenceError> {
    if input.release_marker.trim().is_empty() {
        return Err(DataLayerM11ClosureEvidenceError::EmptyReleaseMarker);
    }

    let reason_codes = determine_reason_codes(&input);
    Ok(DataLayerM11ClosureEvidenceReport {
        release_marker: input.release_marker,
        decision: determine_decision(&reason_codes),
        reason_codes,
        hardening_decision: input.hardening_report.decision,
        critical_scenario_decision: input.critical_scenario_report.decision,
        performance_budget_met: input.performance_budget_met,
        security_signoff_complete: input.security_signoff_complete,
        chaos_signoff_complete: input.chaos_signoff_complete,
    })
}

fn determine_reason_codes(input: &DataLayerM11ClosureEvidenceInput) -> Vec<&'static str> {
    let mut reason_codes = Vec::new();

    if input.hardening_report.decision != DataLayerM11OperatorReadinessDecision::Go {
        reason_codes.push(DATA_LAYER_M11_CLOSURE_BLOCK_HARDENING_REASON_CODE);
    }
    if input.critical_scenario_report.decision
        != DataLayerPrdCriticalScenarioConformanceDecision::Conformant
    {
        reason_codes.push(DATA_LAYER_M11_CLOSURE_BLOCK_CRITICAL_SCENARIO_REASON_CODE);
    }
    if !input.performance_budget_met
        || !input.security_signoff_complete
        || !input.chaos_signoff_complete
    {
        reason_codes.push(DATA_LAYER_M11_CLOSURE_BLOCK_EVIDENCE_GAP_REASON_CODE);
    }
    if reason_codes.is_empty() {
        reason_codes.push(DATA_LAYER_M11_CLOSURE_ACCEPTED_REASON_CODE);
    }

    reason_codes
}

fn determine_decision(reason_codes: &[&'static str]) -> DataLayerM11ClosureAcceptanceDecision {
    if reason_codes == [DATA_LAYER_M11_CLOSURE_ACCEPTED_REASON_CODE] {
        DataLayerM11ClosureAcceptanceDecision::Accepted
    } else {
        DataLayerM11ClosureAcceptanceDecision::Rejected
    }
}
