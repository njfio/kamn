use kamn_core::{
    DataLayerPrdCriticalScenarioConformanceDecision, DataLayerPrdCriticalScenarioConformanceError,
    DataLayerPrdCriticalScenarioConformanceMatrix, DataLayerPrdCriticalScenarioMode,
    DataLayerPrdCriticalScenarioResultInput,
    DATA_LAYER_PRD_CRITICAL_SCENARIO_CONFORMANT_REASON_CODE,
    DATA_LAYER_PRD_CRITICAL_SCENARIO_FAILED_REASON_CODE,
    DATA_LAYER_PRD_CRITICAL_SCENARIO_INVALID_MUTATION_REASON_CODE,
    DATA_LAYER_PRD_CRITICAL_SCENARIO_MISSING_REASON_CODE,
    DATA_LAYER_PRD_CRITICAL_SCENARIO_SHELL_POLICY_REASON_CODE,
};

fn result_input(
    scenario_id: u8,
    passed: bool,
    mode: DataLayerPrdCriticalScenarioMode,
) -> DataLayerPrdCriticalScenarioResultInput {
    DataLayerPrdCriticalScenarioResultInput {
        scenario_id,
        passed,
        orchestration_mode: mode,
        evidence_marker: format!("evidence:prd-critical:{scenario_id}"),
    }
}

#[test]
fn spec_c01_required_scenario_catalog_is_deterministic() {
    let matrix = DataLayerPrdCriticalScenarioConformanceMatrix::new();
    assert_eq!(
        matrix.required_scenario_ids(),
        vec![62, 63, 64, 65, 66, 67, 68, 69, 70, 71]
    );
}

#[test]
fn spec_c02_all_required_scenarios_pass_with_rust_only_orchestration() {
    let mut matrix = DataLayerPrdCriticalScenarioConformanceMatrix::new();
    for scenario_id in matrix.required_scenario_ids() {
        matrix
            .record_result(result_input(
                scenario_id,
                true,
                DataLayerPrdCriticalScenarioMode::RustOnly,
            ))
            .expect("scenario result should record");
    }

    let report = matrix
        .evaluate_conformance()
        .expect("conformance evaluation should succeed");
    assert_eq!(
        report.decision,
        DataLayerPrdCriticalScenarioConformanceDecision::Conformant
    );
    assert_eq!(
        report.reason_codes,
        vec![DATA_LAYER_PRD_CRITICAL_SCENARIO_CONFORMANT_REASON_CODE]
    );
}

#[test]
fn spec_c03_failed_required_scenario_is_non_conformant() {
    let mut matrix = DataLayerPrdCriticalScenarioConformanceMatrix::new();
    for scenario_id in matrix.required_scenario_ids() {
        let passed = scenario_id != 66;
        matrix
            .record_result(result_input(
                scenario_id,
                passed,
                DataLayerPrdCriticalScenarioMode::RustOnly,
            ))
            .expect("scenario result should record");
    }

    let report = matrix
        .evaluate_conformance()
        .expect("conformance evaluation should succeed");
    assert_eq!(
        report.decision,
        DataLayerPrdCriticalScenarioConformanceDecision::NonConformant
    );
    assert_eq!(
        report.reason_codes,
        vec![DATA_LAYER_PRD_CRITICAL_SCENARIO_FAILED_REASON_CODE]
    );
    assert_eq!(report.failed_scenario_ids, vec![66]);
}

#[test]
fn spec_c04_missing_required_scenario_is_non_conformant() {
    let mut matrix = DataLayerPrdCriticalScenarioConformanceMatrix::new();
    for scenario_id in 62..=70 {
        matrix
            .record_result(result_input(
                scenario_id,
                true,
                DataLayerPrdCriticalScenarioMode::RustOnly,
            ))
            .expect("scenario result should record");
    }

    let report = matrix
        .evaluate_conformance()
        .expect("conformance evaluation should succeed");
    assert_eq!(
        report.decision,
        DataLayerPrdCriticalScenarioConformanceDecision::NonConformant
    );
    assert_eq!(
        report.reason_codes,
        vec![DATA_LAYER_PRD_CRITICAL_SCENARIO_MISSING_REASON_CODE]
    );
    assert_eq!(report.missing_scenario_ids, vec![71]);
}

#[test]
fn spec_c05_shell_hybrid_orchestration_is_policy_violation() {
    let mut matrix = DataLayerPrdCriticalScenarioConformanceMatrix::new();
    for scenario_id in matrix.required_scenario_ids() {
        let mode = if scenario_id == 68 {
            DataLayerPrdCriticalScenarioMode::ShellHybrid
        } else {
            DataLayerPrdCriticalScenarioMode::RustOnly
        };
        matrix
            .record_result(result_input(scenario_id, true, mode))
            .expect("scenario result should record");
    }

    let report = matrix
        .evaluate_conformance()
        .expect("conformance evaluation should succeed");
    assert_eq!(
        report.decision,
        DataLayerPrdCriticalScenarioConformanceDecision::NonConformant
    );
    assert_eq!(
        report.reason_codes,
        vec![DATA_LAYER_PRD_CRITICAL_SCENARIO_SHELL_POLICY_REASON_CODE]
    );
    assert_eq!(report.shell_policy_violation_scenario_ids, vec![68]);
}

#[test]
fn spec_c06_invalid_scenario_ids_and_mutating_records_fail_closed() {
    let mut matrix = DataLayerPrdCriticalScenarioConformanceMatrix::new();
    let invalid = matrix.record_result(result_input(
        72,
        true,
        DataLayerPrdCriticalScenarioMode::RustOnly,
    ));
    assert!(matches!(
        invalid,
        Err(DataLayerPrdCriticalScenarioConformanceError::InvalidScenarioId(72))
    ));

    matrix
        .record_result(result_input(
            62,
            true,
            DataLayerPrdCriticalScenarioMode::RustOnly,
        ))
        .expect("scenario result should record");

    let mutation = matrix.record_result(result_input(
        62,
        false,
        DataLayerPrdCriticalScenarioMode::RustOnly,
    ));
    assert!(matches!(
        mutation,
        Err(
            DataLayerPrdCriticalScenarioConformanceError::InvalidResultMutation {
                reason_code: DATA_LAYER_PRD_CRITICAL_SCENARIO_INVALID_MUTATION_REASON_CODE,
                ..
            }
        )
    ));
}
