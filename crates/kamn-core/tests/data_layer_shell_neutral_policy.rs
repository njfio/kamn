use kamn_core::{
    data_layer_evaluate_shell_neutral_policy, DataLayerPrdCriticalScenarioConformanceMatrix,
    DataLayerPrdCriticalScenarioMode, DataLayerPrdCriticalScenarioResultInput,
    DataLayerShellNeutralPolicyDecision, DataLayerShellNeutralPolicyError,
    DataLayerShellNeutralPolicyInput,
    DATA_LAYER_SHELL_NEUTRAL_POLICY_BLOCK_ORCHESTRATION_REASON_CODE,
    DATA_LAYER_SHELL_NEUTRAL_POLICY_BLOCK_RATIO_FAIL_REASON_CODE,
    DATA_LAYER_SHELL_NEUTRAL_POLICY_BLOCK_SHELL_DELTA_REASON_CODE,
    DATA_LAYER_SHELL_NEUTRAL_POLICY_VERIFIED_REASON_CODE,
    DATA_LAYER_SHELL_NEUTRAL_POLICY_WARN_RATIO_REASON_CODE,
};

fn build_critical_report(
    shell_violation_scenario_id: Option<u8>,
) -> kamn_core::DataLayerPrdCriticalScenarioConformanceReport {
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
fn spec_c01_verified_when_shell_neutral_and_ratio_within_budget() {
    let report = data_layer_evaluate_shell_neutral_policy(DataLayerShellNeutralPolicyInput {
        critical_scenario_report: build_critical_report(None),
        shell_loc_delta_actual: 0,
        rust_loc_delta_actual: 250,
        current_shell_to_rust_ratio: 0.90,
        warn_shell_to_rust_ratio_max: 0.95,
        fail_shell_to_rust_ratio_max: 1.00,
    })
    .expect("policy evaluation should succeed");
    assert_eq!(
        report.decision,
        DataLayerShellNeutralPolicyDecision::Verified
    );
    assert_eq!(
        report.reason_codes,
        vec![DATA_LAYER_SHELL_NEUTRAL_POLICY_VERIFIED_REASON_CODE]
    );
}

#[test]
fn spec_c02_orchestration_violations_block_policy() {
    let report = data_layer_evaluate_shell_neutral_policy(DataLayerShellNeutralPolicyInput {
        critical_scenario_report: build_critical_report(Some(68)),
        shell_loc_delta_actual: 0,
        rust_loc_delta_actual: 250,
        current_shell_to_rust_ratio: 0.90,
        warn_shell_to_rust_ratio_max: 0.95,
        fail_shell_to_rust_ratio_max: 1.00,
    })
    .expect("policy evaluation should succeed");
    assert_eq!(
        report.decision,
        DataLayerShellNeutralPolicyDecision::Blocked
    );
    assert_eq!(
        report.reason_codes,
        vec![DATA_LAYER_SHELL_NEUTRAL_POLICY_BLOCK_ORCHESTRATION_REASON_CODE]
    );
}

#[test]
fn spec_c03_positive_shell_delta_or_ratio_fail_blocks_policy() {
    let positive_shell_delta_report =
        data_layer_evaluate_shell_neutral_policy(DataLayerShellNeutralPolicyInput {
            critical_scenario_report: build_critical_report(None),
            shell_loc_delta_actual: 1,
            rust_loc_delta_actual: 0,
            current_shell_to_rust_ratio: 0.90,
            warn_shell_to_rust_ratio_max: 0.95,
            fail_shell_to_rust_ratio_max: 1.00,
        })
        .expect("policy evaluation should succeed");
    assert_eq!(
        positive_shell_delta_report.decision,
        DataLayerShellNeutralPolicyDecision::Blocked
    );
    assert_eq!(
        positive_shell_delta_report.reason_codes,
        vec![DATA_LAYER_SHELL_NEUTRAL_POLICY_BLOCK_SHELL_DELTA_REASON_CODE]
    );

    let ratio_fail_report =
        data_layer_evaluate_shell_neutral_policy(DataLayerShellNeutralPolicyInput {
            critical_scenario_report: build_critical_report(None),
            shell_loc_delta_actual: 0,
            rust_loc_delta_actual: 0,
            current_shell_to_rust_ratio: 1.01,
            warn_shell_to_rust_ratio_max: 0.95,
            fail_shell_to_rust_ratio_max: 1.00,
        })
        .expect("policy evaluation should succeed");
    assert_eq!(
        ratio_fail_report.decision,
        DataLayerShellNeutralPolicyDecision::Blocked
    );
    assert_eq!(
        ratio_fail_report.reason_codes,
        vec![DATA_LAYER_SHELL_NEUTRAL_POLICY_BLOCK_RATIO_FAIL_REASON_CODE]
    );
}

#[test]
fn spec_c04_warn_when_ratio_exceeds_warn_below_fail() {
    let report = data_layer_evaluate_shell_neutral_policy(DataLayerShellNeutralPolicyInput {
        critical_scenario_report: build_critical_report(None),
        shell_loc_delta_actual: 0,
        rust_loc_delta_actual: 100,
        current_shell_to_rust_ratio: 0.97,
        warn_shell_to_rust_ratio_max: 0.95,
        fail_shell_to_rust_ratio_max: 1.00,
    })
    .expect("policy evaluation should succeed");
    assert_eq!(
        report.decision,
        DataLayerShellNeutralPolicyDecision::Warning
    );
    assert_eq!(
        report.reason_codes,
        vec![DATA_LAYER_SHELL_NEUTRAL_POLICY_WARN_RATIO_REASON_CODE]
    );
}

#[test]
fn spec_c05_invalid_threshold_order_fails_closed() {
    let error = data_layer_evaluate_shell_neutral_policy(DataLayerShellNeutralPolicyInput {
        critical_scenario_report: build_critical_report(None),
        shell_loc_delta_actual: 0,
        rust_loc_delta_actual: 100,
        current_shell_to_rust_ratio: 0.90,
        warn_shell_to_rust_ratio_max: 1.00,
        fail_shell_to_rust_ratio_max: 0.95,
    });
    assert!(matches!(
        error,
        Err(DataLayerShellNeutralPolicyError::InvalidThresholdOrder)
    ));
}
