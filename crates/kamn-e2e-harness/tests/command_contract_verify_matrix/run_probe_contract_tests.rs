use super::support_helpers::*;

#[test]
fn spec_c107_external_execution_probe_failure_marks_runtime_orchestration_fail() {
    let kolme_binary = failing_stub_binary("kolme-node-probe-fail");
    let output = with_probe_binaries(|| {
        execute_run_contract(&probe_run_config(&kolme_binary)).expect("run output should render")
    });
    assert!(output.contains("\"runtime_orchestration\":"));
    assert!(output.contains("\"status\":\"FAIL\""));
    assert!(output.contains("probe failed"));
    cleanup_path(&kolme_binary);
}

#[test]
fn spec_c108_external_execution_probe_failure_marks_validation_fail() {
    let kolme_binary = failing_stub_binary("kolme-node-runtime-validation-fail");
    let output = with_probe_binaries(|| {
        execute_run_contract(&probe_run_config(&kolme_binary)).expect("run output should render")
    });
    assert!(output.contains("\"runtime_validation_execution\":"));
    assert!(output.contains("\"orchestration_contract\":\"FAIL\""));
    assert!(output.contains("\"lifecycle_contract\":\"FAIL\""));
    assert!(output.contains("\"overall\":\"FAIL\""));
    cleanup_path(&kolme_binary);
}

#[test]
fn spec_c109_run_output_contains_ordered_scenario_contract_projection() {
    let output = execute_run_contract(&ordered_run_config()).expect("run output should render");
    assert!(output.contains("\"scenario_contracts\":["));
    assert!(output.contains("\"steps\":["));
    assert!(output.contains("\"verifiable_outputs\":["));
    assert!(output.contains("\"pass_criteria\":["));
    assert_ordered_scenarios(&output);
}

fn assert_ordered_scenarios(output: &str) {
    let s03_index = output
        .find("\"id\":\"S-03\"")
        .expect("S-03 contract entry should be present");
    let s01_index = output
        .find("\"id\":\"S-01\"")
        .expect("S-01 contract entry should be present");
    assert!(
        s03_index < s01_index,
        "scenario contracts should preserve selected order"
    );
}
