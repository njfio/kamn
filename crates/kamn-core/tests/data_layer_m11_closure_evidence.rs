use kamn_core::{
    data_layer_m11_evaluate_closure_evidence, DataLayerM11ClosureAcceptanceDecision,
    DataLayerM11ClosureEvidenceError, DataLayerM11ClosureEvidenceInput,
    DataLayerM11HardeningMatrix, DataLayerM11ScenarioDefinition, DataLayerM11ScenarioDomain,
    DataLayerM11ScenarioOutcomeInput, DataLayerM11ScenarioSeverity, DataLayerM11ScenarioStatus,
    DataLayerPrdCriticalScenarioConformanceMatrix, DataLayerPrdCriticalScenarioMode,
    DataLayerPrdCriticalScenarioResultInput, DATA_LAYER_M11_CLOSURE_ACCEPTED_REASON_CODE,
    DATA_LAYER_M11_CLOSURE_BLOCK_CRITICAL_SCENARIO_REASON_CODE,
    DATA_LAYER_M11_CLOSURE_BLOCK_EVIDENCE_GAP_REASON_CODE,
    DATA_LAYER_M11_CLOSURE_BLOCK_HARDENING_REASON_CODE,
};

fn build_hardening_report(all_passed: bool) -> kamn_core::DataLayerM11OperatorReadinessReport {
    let mut matrix = DataLayerM11HardeningMatrix::new();
    matrix
        .register_scenario(DataLayerM11ScenarioDefinition {
            scenario_id: "security.authz-negative-matrix".to_owned(),
            domain: DataLayerM11ScenarioDomain::Security,
            severity: DataLayerM11ScenarioSeverity::Critical,
            required: true,
        })
        .expect("security scenario should register");
    matrix
        .register_scenario(DataLayerM11ScenarioDefinition {
            scenario_id: "chaos.partition-heal".to_owned(),
            domain: DataLayerM11ScenarioDomain::Chaos,
            severity: DataLayerM11ScenarioSeverity::High,
            required: true,
        })
        .expect("chaos scenario should register");
    matrix
        .register_scenario(DataLayerM11ScenarioDefinition {
            scenario_id: "operations.runbook-signoff".to_owned(),
            domain: DataLayerM11ScenarioDomain::Operations,
            severity: DataLayerM11ScenarioSeverity::Medium,
            required: true,
        })
        .expect("operations scenario should register");

    matrix
        .record_outcome(DataLayerM11ScenarioOutcomeInput {
            scenario_id: "security.authz-negative-matrix".to_owned(),
            status: if all_passed {
                DataLayerM11ScenarioStatus::Passed
            } else {
                DataLayerM11ScenarioStatus::Failed
            },
            evidence_marker: "evidence:security".to_owned(),
        })
        .expect("security result should record");
    matrix
        .record_outcome(DataLayerM11ScenarioOutcomeInput {
            scenario_id: "chaos.partition-heal".to_owned(),
            status: DataLayerM11ScenarioStatus::Passed,
            evidence_marker: "evidence:chaos".to_owned(),
        })
        .expect("chaos result should record");
    matrix
        .record_outcome(DataLayerM11ScenarioOutcomeInput {
            scenario_id: "operations.runbook-signoff".to_owned(),
            status: DataLayerM11ScenarioStatus::Passed,
            evidence_marker: "evidence:ops".to_owned(),
        })
        .expect("ops result should record");

    matrix
        .evaluate_operator_readiness()
        .expect("hardening report should evaluate")
}

fn build_critical_report(
    fail_scenario_id: Option<u8>,
) -> kamn_core::DataLayerPrdCriticalScenarioConformanceReport {
    let mut matrix = DataLayerPrdCriticalScenarioConformanceMatrix::new();
    for scenario_id in matrix.required_scenario_ids() {
        matrix
            .record_result(DataLayerPrdCriticalScenarioResultInput {
                scenario_id,
                passed: fail_scenario_id != Some(scenario_id),
                orchestration_mode: DataLayerPrdCriticalScenarioMode::RustOnly,
                evidence_marker: format!("evidence:critical:{scenario_id}"),
            })
            .expect("critical scenario should record");
    }
    matrix
        .evaluate_conformance()
        .expect("critical conformance should evaluate")
}

#[test]
fn spec_c01_conformant_closure_evidence_is_accepted() {
    let report = data_layer_m11_evaluate_closure_evidence(DataLayerM11ClosureEvidenceInput {
        release_marker: "release-2026-02-18".to_owned(),
        hardening_report: build_hardening_report(true),
        critical_scenario_report: build_critical_report(None),
        performance_budget_met: true,
        security_signoff_complete: true,
        chaos_signoff_complete: true,
    })
    .expect("closure evaluation should succeed");
    assert_eq!(
        report.decision,
        DataLayerM11ClosureAcceptanceDecision::Accepted
    );
    assert_eq!(
        report.reason_codes,
        vec![DATA_LAYER_M11_CLOSURE_ACCEPTED_REASON_CODE]
    );
}

#[test]
fn spec_c02_hardening_nogo_blocks_closure_acceptance() {
    let report = data_layer_m11_evaluate_closure_evidence(DataLayerM11ClosureEvidenceInput {
        release_marker: "release-2026-02-18".to_owned(),
        hardening_report: build_hardening_report(false),
        critical_scenario_report: build_critical_report(None),
        performance_budget_met: true,
        security_signoff_complete: true,
        chaos_signoff_complete: true,
    })
    .expect("closure evaluation should succeed");
    assert_eq!(
        report.decision,
        DataLayerM11ClosureAcceptanceDecision::Rejected
    );
    assert_eq!(
        report.reason_codes,
        vec![DATA_LAYER_M11_CLOSURE_BLOCK_HARDENING_REASON_CODE]
    );
}

#[test]
fn spec_c03_non_conformant_critical_scenario_blocks_closure_acceptance() {
    let report = data_layer_m11_evaluate_closure_evidence(DataLayerM11ClosureEvidenceInput {
        release_marker: "release-2026-02-18".to_owned(),
        hardening_report: build_hardening_report(true),
        critical_scenario_report: build_critical_report(Some(68)),
        performance_budget_met: true,
        security_signoff_complete: true,
        chaos_signoff_complete: true,
    })
    .expect("closure evaluation should succeed");
    assert_eq!(
        report.decision,
        DataLayerM11ClosureAcceptanceDecision::Rejected
    );
    assert_eq!(
        report.reason_codes,
        vec![DATA_LAYER_M11_CLOSURE_BLOCK_CRITICAL_SCENARIO_REASON_CODE]
    );
}

#[test]
fn spec_c04_missing_performance_or_signoff_evidence_blocks_closure_acceptance() {
    let report = data_layer_m11_evaluate_closure_evidence(DataLayerM11ClosureEvidenceInput {
        release_marker: "release-2026-02-18".to_owned(),
        hardening_report: build_hardening_report(true),
        critical_scenario_report: build_critical_report(None),
        performance_budget_met: false,
        security_signoff_complete: false,
        chaos_signoff_complete: true,
    })
    .expect("closure evaluation should succeed");
    assert_eq!(
        report.decision,
        DataLayerM11ClosureAcceptanceDecision::Rejected
    );
    assert_eq!(
        report.reason_codes,
        vec![DATA_LAYER_M11_CLOSURE_BLOCK_EVIDENCE_GAP_REASON_CODE]
    );
}

#[test]
fn spec_c05_empty_release_marker_fails_closed() {
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
