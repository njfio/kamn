use kamn_data_layer::{
    data_layer_m11_evaluate_closure_evidence, DataLayerM11ClosureAcceptanceDecision,
    DataLayerM11ClosureEvidenceError, DataLayerM11ClosureEvidenceInput,
    DataLayerM11HardeningMatrix, DataLayerM11OperatorReadinessReport,
    DataLayerM11ScenarioDefinition, DataLayerM11ScenarioDomain, DataLayerM11ScenarioOutcomeInput,
    DataLayerM11ScenarioSeverity, DataLayerM11ScenarioStatus,
    DataLayerPrdCriticalScenarioConformanceMatrix, DataLayerPrdCriticalScenarioMode,
    DataLayerPrdCriticalScenarioResultInput, DATA_LAYER_M11_CLOSURE_ACCEPTED_REASON_CODE,
    DATA_LAYER_M11_CLOSURE_BLOCK_CRITICAL_SCENARIO_REASON_CODE,
    DATA_LAYER_M11_CLOSURE_BLOCK_EVIDENCE_GAP_REASON_CODE,
    DATA_LAYER_M11_CLOSURE_BLOCK_HARDENING_REASON_CODE,
};

const RELEASE_MARKER: &str = "release-2026-03-07";

fn build_hardening_report(all_passed: bool) -> DataLayerM11OperatorReadinessReport {
    let mut matrix = DataLayerM11HardeningMatrix::new();
    register_required_hardening_scenarios(&mut matrix);
    record_required_hardening_outcomes(&mut matrix, all_passed);
    matrix
        .evaluate_operator_readiness()
        .expect("hardening readiness should evaluate")
}

fn register_required_hardening_scenarios(matrix: &mut DataLayerM11HardeningMatrix) {
    register_required_scenario(
        matrix,
        "security.authz-negative-matrix",
        DataLayerM11ScenarioDomain::Security,
        DataLayerM11ScenarioSeverity::Critical,
    );
    register_required_scenario(
        matrix,
        "chaos.partition-heal",
        DataLayerM11ScenarioDomain::Chaos,
        DataLayerM11ScenarioSeverity::High,
    );
}

fn record_required_hardening_outcomes(matrix: &mut DataLayerM11HardeningMatrix, all_passed: bool) {
    let security_status = if all_passed {
        DataLayerM11ScenarioStatus::Passed
    } else {
        DataLayerM11ScenarioStatus::Failed
    };
    record_hardening_outcome(
        matrix,
        "security.authz-negative-matrix",
        security_status,
        "evidence:security",
    );
    record_hardening_outcome(
        matrix,
        "chaos.partition-heal",
        DataLayerM11ScenarioStatus::Passed,
        "evidence:chaos",
    );
}

fn register_required_scenario(
    matrix: &mut DataLayerM11HardeningMatrix,
    scenario_id: &str,
    domain: DataLayerM11ScenarioDomain,
    severity: DataLayerM11ScenarioSeverity,
) {
    matrix
        .register_scenario(DataLayerM11ScenarioDefinition {
            scenario_id: scenario_id.to_owned(),
            domain,
            severity,
            required: true,
        })
        .expect("required scenario should register");
}

fn record_hardening_outcome(
    matrix: &mut DataLayerM11HardeningMatrix,
    scenario_id: &str,
    status: DataLayerM11ScenarioStatus,
    evidence_marker: &str,
) {
    matrix
        .record_outcome(DataLayerM11ScenarioOutcomeInput {
            scenario_id: scenario_id.to_owned(),
            status,
            evidence_marker: evidence_marker.to_owned(),
        })
        .expect("hardening outcome should record");
}

fn build_critical_report(
    failing_scenario_id: Option<u8>,
) -> kamn_data_layer::DataLayerPrdCriticalScenarioConformanceReport {
    let mut matrix = DataLayerPrdCriticalScenarioConformanceMatrix::new();
    for scenario_id in matrix.required_scenario_ids() {
        matrix
            .record_result(DataLayerPrdCriticalScenarioResultInput {
                scenario_id,
                passed: failing_scenario_id != Some(scenario_id),
                orchestration_mode: DataLayerPrdCriticalScenarioMode::RustOnly,
                evidence_marker: format!("evidence:critical:{scenario_id}"),
            })
            .expect("critical scenario should record");
    }
    matrix
        .evaluate_conformance()
        .expect("critical report should evaluate")
}

fn evaluate_closure(
    hardening_passed: bool,
    failing_scenario_id: Option<u8>,
    performance_budget_met: bool,
    security_signoff_complete: bool,
    chaos_signoff_complete: bool,
) -> kamn_data_layer::DataLayerM11ClosureEvidenceReport {
    data_layer_m11_evaluate_closure_evidence(DataLayerM11ClosureEvidenceInput {
        release_marker: RELEASE_MARKER.to_owned(),
        hardening_report: build_hardening_report(hardening_passed),
        critical_scenario_report: build_critical_report(failing_scenario_id),
        performance_budget_met,
        security_signoff_complete,
        chaos_signoff_complete,
    })
    .expect("closure evidence should evaluate")
}

#[test]
fn integration_m11_closure_evidence_accepts_when_all_gates_pass() {
    let accepted = evaluate_closure(true, None, true, true, true);
    assert_eq!(
        accepted.decision,
        DataLayerM11ClosureAcceptanceDecision::Accepted
    );
    assert_eq!(
        accepted.reason_codes,
        vec![DATA_LAYER_M11_CLOSURE_ACCEPTED_REASON_CODE]
    );
}

#[test]
fn integration_m11_closure_evidence_rejects_for_hardening_nogo() {
    let blocked = evaluate_closure(false, None, true, true, true);
    assert_eq!(
        blocked.reason_codes,
        vec![DATA_LAYER_M11_CLOSURE_BLOCK_HARDENING_REASON_CODE]
    );
}

#[test]
fn integration_m11_closure_evidence_rejects_for_critical_scenario_failure() {
    let blocked = evaluate_closure(true, Some(67), true, true, true);
    assert_eq!(
        blocked.reason_codes,
        vec![DATA_LAYER_M11_CLOSURE_BLOCK_CRITICAL_SCENARIO_REASON_CODE]
    );
}

#[test]
fn integration_m11_closure_evidence_rejects_for_evidence_gap() {
    let blocked = evaluate_closure(true, None, false, false, true);
    assert_eq!(
        blocked.reason_codes,
        vec![DATA_LAYER_M11_CLOSURE_BLOCK_EVIDENCE_GAP_REASON_CODE]
    );
}

#[test]
fn integration_m11_closure_evidence_projects_combined_blocking_reasons_in_order() {
    let blocked = evaluate_closure(false, Some(67), false, true, false);
    assert_eq!(
        blocked.reason_codes,
        vec![
            DATA_LAYER_M11_CLOSURE_BLOCK_HARDENING_REASON_CODE,
            DATA_LAYER_M11_CLOSURE_BLOCK_CRITICAL_SCENARIO_REASON_CODE,
            DATA_LAYER_M11_CLOSURE_BLOCK_EVIDENCE_GAP_REASON_CODE,
        ]
    );
}

#[test]
fn integration_m11_closure_evidence_fails_closed_for_empty_release_marker() {
    let error = data_layer_m11_evaluate_closure_evidence(DataLayerM11ClosureEvidenceInput {
        release_marker: " ".to_owned(),
        hardening_report: build_hardening_report(true),
        critical_scenario_report: build_critical_report(None),
        performance_budget_met: true,
        security_signoff_complete: true,
        chaos_signoff_complete: true,
    });
    assert!(matches!(
        error,
        Err(DataLayerM11ClosureEvidenceError::EmptyReleaseMarker)
    ));
}
