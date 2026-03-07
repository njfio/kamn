use kamn_data_layer::{
    data_layer_evaluate_shell_neutral_policy, DataLayerPrdCriticalScenarioConformanceMatrix,
    DataLayerPrdCriticalScenarioMode, DataLayerPrdCriticalScenarioResultInput,
    DataLayerShellNeutralPolicyDecision, DataLayerShellNeutralPolicyError,
    DataLayerShellNeutralPolicyInput, DataLayerShellNeutralPolicyReasonCode,
};

fn build_critical_report(
    shell_violation_scenario_id: Option<u8>,
) -> kamn_data_layer::DataLayerPrdCriticalScenarioConformanceReport {
    let mut matrix = DataLayerPrdCriticalScenarioConformanceMatrix::new();
    for scenario_id in matrix.required_scenario_ids() {
        let mode = if shell_violation_scenario_id == Some(scenario_id) {
            DataLayerPrdCriticalScenarioMode::ShellHybrid
        } else {
            DataLayerPrdCriticalScenarioMode::RustOnly
        };
        matrix
            .record_result(DataLayerPrdCriticalScenarioResultInput {
                scenario_id,
                passed: true,
                orchestration_mode: mode,
                evidence_marker: format!("evidence:critical:{scenario_id}"),
            })
            .expect("critical scenario should record");
    }
    matrix
        .evaluate_conformance()
        .expect("critical report should evaluate")
}

#[test]
fn integration_shell_neutral_policy_covers_verified_warning_and_blocked_paths() {
    let verified = data_layer_evaluate_shell_neutral_policy(DataLayerShellNeutralPolicyInput {
        critical_scenario_report: build_critical_report(None),
        shell_loc_delta_actual: 0,
        rust_loc_delta_actual: 250,
        current_shell_to_rust_ratio: 0.90,
        warn_shell_to_rust_ratio_max: 0.95,
        fail_shell_to_rust_ratio_max: 1.00,
    })
    .expect("verified policy evaluation should succeed");
    assert_eq!(verified.decision, DataLayerShellNeutralPolicyDecision::Verified);
    assert_eq!(
        verified.reason_codes,
        vec![DataLayerShellNeutralPolicyReasonCode::Verified]
    );

    let warning = data_layer_evaluate_shell_neutral_policy(DataLayerShellNeutralPolicyInput {
        critical_scenario_report: build_critical_report(None),
        shell_loc_delta_actual: 0,
        rust_loc_delta_actual: 100,
        current_shell_to_rust_ratio: 0.97,
        warn_shell_to_rust_ratio_max: 0.95,
        fail_shell_to_rust_ratio_max: 1.00,
    })
    .expect("warning policy evaluation should succeed");
    assert_eq!(warning.decision, DataLayerShellNeutralPolicyDecision::Warning);
    assert_eq!(
        warning.reason_codes,
        vec![DataLayerShellNeutralPolicyReasonCode::WarnRatioThreshold]
    );

    let orchestration_block =
        data_layer_evaluate_shell_neutral_policy(DataLayerShellNeutralPolicyInput {
            critical_scenario_report: build_critical_report(Some(68)),
            shell_loc_delta_actual: 0,
            rust_loc_delta_actual: 250,
            current_shell_to_rust_ratio: 0.90,
            warn_shell_to_rust_ratio_max: 0.95,
            fail_shell_to_rust_ratio_max: 1.00,
        })
        .expect("orchestration blocked evaluation should succeed");
    assert_eq!(
        orchestration_block.decision,
        DataLayerShellNeutralPolicyDecision::Blocked
    );
    assert_eq!(
        orchestration_block.reason_codes,
        vec![DataLayerShellNeutralPolicyReasonCode::BlockOrchestrationViolation]
    );

    let shell_delta_block =
        data_layer_evaluate_shell_neutral_policy(DataLayerShellNeutralPolicyInput {
            critical_scenario_report: build_critical_report(None),
            shell_loc_delta_actual: 1,
            rust_loc_delta_actual: 0,
            current_shell_to_rust_ratio: 0.90,
            warn_shell_to_rust_ratio_max: 0.95,
            fail_shell_to_rust_ratio_max: 1.00,
        })
        .expect("shell delta blocked evaluation should succeed");
    assert_eq!(
        shell_delta_block.reason_codes,
        vec![DataLayerShellNeutralPolicyReasonCode::BlockPositiveShellDelta]
    );

    let ratio_block = data_layer_evaluate_shell_neutral_policy(DataLayerShellNeutralPolicyInput {
        critical_scenario_report: build_critical_report(None),
        shell_loc_delta_actual: 0,
        rust_loc_delta_actual: 0,
        current_shell_to_rust_ratio: 1.01,
        warn_shell_to_rust_ratio_max: 0.95,
        fail_shell_to_rust_ratio_max: 1.00,
    })
    .expect("ratio blocked evaluation should succeed");
    assert_eq!(
        ratio_block.reason_codes,
        vec![DataLayerShellNeutralPolicyReasonCode::BlockRatioFailThreshold]
    );
}

#[test]
fn integration_shell_neutral_policy_fails_closed_for_thresholds_and_unknown_markers() {
    let threshold_error =
        data_layer_evaluate_shell_neutral_policy(DataLayerShellNeutralPolicyInput {
            critical_scenario_report: build_critical_report(None),
            shell_loc_delta_actual: 0,
            rust_loc_delta_actual: 100,
            current_shell_to_rust_ratio: 0.90,
            warn_shell_to_rust_ratio_max: 1.00,
            fail_shell_to_rust_ratio_max: 0.95,
        });
    assert!(matches!(
        threshold_error,
        Err(DataLayerShellNeutralPolicyError::InvalidThresholdOrder)
    ));

    let parsed =
        "shell_neutral_policy_unknown_marker".parse::<DataLayerShellNeutralPolicyReasonCode>();
    assert!(parsed.is_err(), "unknown reason marker should fail closed");
}
