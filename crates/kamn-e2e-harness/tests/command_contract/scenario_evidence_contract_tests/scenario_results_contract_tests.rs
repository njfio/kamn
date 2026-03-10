use super::*;

#[test]
fn spec_c72_run_output_contains_scenario_results_in_selected_order() {
    let config = RunCommandConfig {
        mode: "sdk-direct".to_owned(),
        kolme_binary: "/tmp/kolme-node".to_owned(),
        agent_binary: None,
        external_execution: false,
        evidence_dir: "/tmp/evidence".to_owned(),
        scenario_ids: vec!["S-03".to_owned(), "S-01".to_owned(), "S-15".to_owned()],
    };
    let output = execute_run_contract(&config).expect("run output should render");
    assert!(output.contains("\"scenario_results\":["));
    assert!(output.contains(
        "\"scenario_results\":[{\"id\":\"S-03\",\"status\":\"PASS\"},{\"id\":\"S-01\",\"status\":\"PASS\"},{\"id\":\"S-15\",\"status\":\"PASS\"}]"
    ));
}

#[test]
fn spec_c73_scenario_run_phase_is_pass_when_all_selected_scenarios_pass() {
    let config = RunCommandConfig {
        mode: "cli-scripted".to_owned(),
        kolme_binary: "/tmp/kolme-node".to_owned(),
        agent_binary: None,
        external_execution: false,
        evidence_dir: "/tmp/evidence".to_owned(),
        scenario_ids: vec!["S-01".to_owned(), "S-02".to_owned()],
    };
    let output = execute_run_contract(&config).expect("run output should render");
    assert!(output.contains("{\"phase\":\"SCENARIO_RUN\",\"status\":\"PASS\""));
    assert!(output.contains(
        "{\"step\":\"Execute selected scenarios via mode driver\",\"status\":\"PASS\",\"detail\":\"executed=2 pass=2 fail=0 skip=0\"}"
    ));
}

#[test]
fn spec_c74_scenario_run_phase_fails_when_scenario_execution_reports_fail() {
    let config = RunCommandConfig {
        mode: "sdk-direct".to_owned(),
        kolme_binary: "/tmp/kolme-node".to_owned(),
        agent_binary: None,
        external_execution: false,
        evidence_dir: "/tmp/scenario-fail".to_owned(),
        scenario_ids: vec!["S-01".to_owned(), "S-02".to_owned()],
    };
    let output = execute_run_contract(&config).expect("run output should render");
    assert!(output.contains("\"scenario_results\":[{\"id\":\"S-01\",\"status\":\"FAIL\"},{\"id\":\"S-02\",\"status\":\"PASS\"}]"));
    assert!(output.contains("{\"phase\":\"SCENARIO_RUN\",\"status\":\"FAIL\""));
    assert!(output.contains("\"phase_totals\":{\"total\":5,\"pass\":4,\"fail\":1,\"skip\":0}"));
}

#[test]
fn spec_c75_runtime_orchestration_contract_markers_remain_stable_with_scenario_results() {
    let config = RunCommandConfig {
        mode: "sdk-direct".to_owned(),
        kolme_binary: "/tmp/kolme-node".to_owned(),
        agent_binary: None,
        external_execution: false,
        evidence_dir: "/tmp/evidence".to_owned(),
        scenario_ids: vec!["S-01".to_owned()],
    };
    let output = execute_run_contract(&config).expect("run output should render");
    assert!(output.contains(
        "\"runtime_orchestration\":{\"postgres\":{\"requested\":false,\"status\":\"SKIP\",\"detail\":\"external execution disabled\"},\"kolme\":{\"requested\":false,\"status\":\"SKIP\",\"detail\":\"external execution disabled\"},\"kamn_processor\":{\"requested\":false,\"status\":\"SKIP\",\"detail\":\"external execution disabled\"},\"kamn_listener\":{\"requested\":false,\"status\":\"SKIP\",\"detail\":\"external execution disabled\"},\"kamn_approver\":{\"requested\":false,\"status\":\"SKIP\",\"detail\":\"external execution disabled\"}}"
    ));
}

#[test]
fn spec_c76_live_execution_overall_status_fails_when_scenario_fails() {
    let config = RunCommandConfig {
        mode: "sdk-direct".to_owned(),
        kolme_binary: "/tmp/kolme-node".to_owned(),
        agent_binary: None,
        external_execution: false,
        evidence_dir: "/tmp/scenario-fail".to_owned(),
        scenario_ids: vec!["S-01".to_owned(), "S-02".to_owned()],
    };
    let output = execute_run_contract(&config).expect("run output should render");
    assert!(output.contains("\"live_execution\":{\"orchestration_status\":\"PASS\",\"validation_status\":\"FAIL\",\"evidence_status\":\"PASS\",\"overall_status\":\"FAIL\"}"));
}

#[test]
fn spec_c77_live_validation_status_and_completed_checks_reflect_failure_path() {
    let config = RunCommandConfig {
        mode: "sdk-direct".to_owned(),
        kolme_binary: "/tmp/kolme-node".to_owned(),
        agent_binary: None,
        external_execution: false,
        evidence_dir: "/tmp/scenario-fail".to_owned(),
        scenario_ids: vec!["S-01".to_owned(), "S-02".to_owned()],
    };
    let output = execute_run_contract(&config).expect("run output should render");
    assert!(output.contains(
        "\"live_validation\":{\"expected_checks\":4,\"completed_checks\":3,\"status\":\"FAIL\"}"
    ));
}
