use kamn_data_layer::{
    data_layer_m11_evaluate_closure_evidence, DataLayerM11ClosureAcceptanceDecision,
    DataLayerM11ClosureEvidenceError, DataLayerM11ClosureEvidenceInput,
    DataLayerM11HardeningMatrix, DataLayerM11OperatorReadinessReport,
    DataLayerM11ScenarioDefinition, DataLayerM11ScenarioDomain, DataLayerM11ScenarioOutcomeInput,
    DataLayerM11ScenarioSeverity, DataLayerM11ScenarioStatus,
    DataLayerPrdCriticalScenarioConformanceMatrix, DataLayerPrdCriticalScenarioMode,
    DataLayerPrdCriticalScenarioResultInput,
    DATA_LAYER_M11_CLOSURE_ACCEPTED_REASON_CODE,
    DATA_LAYER_M11_CLOSURE_BLOCK_CRITICAL_SCENARIO_REASON_CODE,
    DATA_LAYER_M11_CLOSURE_BLOCK_EVIDENCE_GAP_REASON_CODE,
    DATA_LAYER_M11_CLOSURE_BLOCK_HARDENING_REASON_CODE,
};

fn build_hardening_report(all_passed: bool) -> DataLayerM11OperatorReadinessReport {
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
        .evaluate_operator_readiness()
        .expect("hardening readiness should evaluate")
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

#[test]
fn integration_m11_closure_evidence_covers_accept_and_reject_paths() {
    let accepted = data_layer_m11_evaluate_closure_evidence(DataLayerM11ClosureEvidenceInput {
        release_marker: "release-2026-03-07".to_owned(),
        hardening_report: build_hardening_report(true),
        critical_scenario_report: build_critical_report(None),
        performance_budget_met: true,
        security_signoff_complete: true,
        chaos_signoff_complete: true,
    })
    .expect("accepted closure evidence should evaluate");
    assert_eq!(
        accepted.decision,
        DataLayerM11ClosureAcceptanceDecision::Accepted
    );
    assert_eq!(
        accepted.reason_codes,
        vec![DATA_LAYER_M11_CLOSURE_ACCEPTED_REASON_CODE]
    );

    let hardening_blocked =
        data_layer_m11_evaluate_closure_evidence(DataLayerM11ClosureEvidenceInput {
            release_marker: "release-2026-03-07".to_owned(),
            hardening_report: build_hardening_report(false),
            critical_scenario_report: build_critical_report(None),
            performance_budget_met: true,
            security_signoff_complete: true,
            chaos_signoff_complete: true,
        })
        .expect("hardening-blocked closure evidence should evaluate");
    assert_eq!(
        hardening_blocked.reason_codes,
        vec![DATA_LAYER_M11_CLOSURE_BLOCK_HARDENING_REASON_CODE]
    );

    let critical_blocked =
        data_layer_m11_evaluate_closure_evidence(DataLayerM11ClosureEvidenceInput {
            release_marker: "release-2026-03-07".to_owned(),
            hardening_report: build_hardening_report(true),
            critical_scenario_report: build_critical_report(Some(67)),
            performance_budget_met: true,
            security_signoff_complete: true,
            chaos_signoff_complete: true,
        })
        .expect("critical-blocked closure evidence should evaluate");
    assert_eq!(
        critical_blocked.reason_codes,
        vec![DATA_LAYER_M11_CLOSURE_BLOCK_CRITICAL_SCENARIO_REASON_CODE]
    );

    let evidence_gap =
        data_layer_m11_evaluate_closure_evidence(DataLayerM11ClosureEvidenceInput {
            release_marker: "release-2026-03-07".to_owned(),
            hardening_report: build_hardening_report(true),
            critical_scenario_report: build_critical_report(None),
            performance_budget_met: false,
            security_signoff_complete: false,
            chaos_signoff_complete: true,
        })
        .expect("evidence-gap closure evidence should evaluate");
    assert_eq!(
        evidence_gap.reason_codes,
        vec![DATA_LAYER_M11_CLOSURE_BLOCK_EVIDENCE_GAP_REASON_CODE]
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
