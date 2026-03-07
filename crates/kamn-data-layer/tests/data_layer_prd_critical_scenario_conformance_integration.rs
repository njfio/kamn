use kamn_data_layer::{
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
    orchestration_mode: DataLayerPrdCriticalScenarioMode,
) -> DataLayerPrdCriticalScenarioResultInput {
    DataLayerPrdCriticalScenarioResultInput {
        scenario_id,
        passed,
        orchestration_mode,
        evidence_marker: format!("evidence:prd-critical:{scenario_id}"),
    }
}

#[test]
fn integration_catalog_is_deterministic_and_complete() {
    let matrix = DataLayerPrdCriticalScenarioConformanceMatrix::new();
    assert_eq!(
        matrix.required_scenario_ids(),
        vec![62, 63, 64, 65, 66, 67, 68, 69, 70, 71]
    );
}

#[test]
fn integration_evaluates_conformant_when_all_required_results_pass_in_rust_mode() {
    let mut matrix = DataLayerPrdCriticalScenarioConformanceMatrix::new();
    for scenario_id in matrix.required_scenario_ids() {
        matrix
            .record_result(result_input(
                scenario_id,
                true,
                DataLayerPrdCriticalScenarioMode::RustOnly,
            ))
            .expect("result should record");
    }

    let report = matrix
        .evaluate_conformance()
        .expect("report should evaluate");
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
fn integration_evaluates_non_conformance_for_failed_missing_and_shell_policy_inputs() {
    let mut failed = DataLayerPrdCriticalScenarioConformanceMatrix::new();
    for scenario_id in failed.required_scenario_ids() {
        failed
            .record_result(result_input(
                scenario_id,
                scenario_id != 66,
                DataLayerPrdCriticalScenarioMode::RustOnly,
            ))
            .expect("failed result should record");
    }
    let failed_report = failed
        .evaluate_conformance()
        .expect("failed report should evaluate");
    assert_eq!(
        failed_report.reason_codes,
        vec![DATA_LAYER_PRD_CRITICAL_SCENARIO_FAILED_REASON_CODE]
    );
    assert_eq!(failed_report.failed_scenario_ids, vec![66]);

    let mut missing = DataLayerPrdCriticalScenarioConformanceMatrix::new();
    for scenario_id in 62..=70 {
        missing
            .record_result(result_input(
                scenario_id,
                true,
                DataLayerPrdCriticalScenarioMode::RustOnly,
            ))
            .expect("missing result should record");
    }
    let missing_report = missing
        .evaluate_conformance()
        .expect("missing report should evaluate");
    assert_eq!(
        missing_report.reason_codes,
        vec![DATA_LAYER_PRD_CRITICAL_SCENARIO_MISSING_REASON_CODE]
    );
    assert_eq!(missing_report.missing_scenario_ids, vec![71]);

    let mut shell_policy = DataLayerPrdCriticalScenarioConformanceMatrix::new();
    for scenario_id in shell_policy.required_scenario_ids() {
        let mode = if scenario_id == 68 {
            DataLayerPrdCriticalScenarioMode::ShellHybrid
        } else {
            DataLayerPrdCriticalScenarioMode::RustOnly
        };
        shell_policy
            .record_result(result_input(scenario_id, true, mode))
            .expect("shell-policy result should record");
    }
    let shell_policy_report = shell_policy
        .evaluate_conformance()
        .expect("shell-policy report should evaluate");
    assert_eq!(
        shell_policy_report.reason_codes,
        vec![DATA_LAYER_PRD_CRITICAL_SCENARIO_SHELL_POLICY_REASON_CODE]
    );
    assert_eq!(shell_policy_report.shell_policy_violation_scenario_ids, vec![68]);
}

#[test]
fn integration_fails_closed_for_invalid_inputs_and_result_mutation() {
    let mut matrix = DataLayerPrdCriticalScenarioConformanceMatrix::new();

    let empty_evidence = matrix.record_result(DataLayerPrdCriticalScenarioResultInput {
        scenario_id: 62,
        passed: true,
        orchestration_mode: DataLayerPrdCriticalScenarioMode::RustOnly,
        evidence_marker: "   ".to_owned(),
    });
    assert_eq!(
        empty_evidence,
        Err(DataLayerPrdCriticalScenarioConformanceError::EmptyField(
            "evidence_marker",
        ))
    );

    let invalid = matrix.record_result(result_input(
        99,
        true,
        DataLayerPrdCriticalScenarioMode::RustOnly,
    ));
    assert_eq!(
        invalid,
        Err(DataLayerPrdCriticalScenarioConformanceError::InvalidScenarioId(99))
    );

    matrix
        .record_result(result_input(
            62,
            true,
            DataLayerPrdCriticalScenarioMode::RustOnly,
        ))
        .expect("baseline result should record");

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
