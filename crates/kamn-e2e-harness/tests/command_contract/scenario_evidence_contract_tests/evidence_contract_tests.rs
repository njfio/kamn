use super::*;

#[test]
fn spec_c78_run_output_contains_evidence_contract_markers() {
    let config = RunCommandConfig {
        mode: "sdk-direct".to_owned(),
        kolme_binary: "/tmp/kolme-node".to_owned(),
        agent_binary: None,
        external_execution: false,
        evidence_dir: "/tmp/evidence".to_owned(),
        scenario_ids: vec!["S-01".to_owned(), "S-02".to_owned()],
    };
    let output = execute_run_contract(&config).expect("run output should render");
    assert!(output.contains("\"evidence_contract\":"));
    assert!(output.contains("\"expected_artifacts\""));
    assert!(output.contains("\"recorded_artifacts\""));
    assert!(output.contains("\"status\""));
}

#[test]
fn spec_c79_live_execution_evidence_status_matches_evidence_contract_status() {
    let config = RunCommandConfig {
        mode: "sdk-direct".to_owned(),
        kolme_binary: "/tmp/kolme-node".to_owned(),
        agent_binary: None,
        external_execution: false,
        evidence_dir: "/tmp/evidence".to_owned(),
        scenario_ids: vec!["S-01".to_owned(), "S-02".to_owned()],
    };
    let output = execute_run_contract(&config).expect("run output should render");
    assert!(output.contains(
        "\"evidence_contract\":{\"expected_artifacts\":4,\"recorded_artifacts\":4,\"status\":\"PASS\"}"
    ));
    assert!(output.contains(
        "\"live_execution\":{\"orchestration_status\":\"PASS\",\"validation_status\":\"PASS\",\"evidence_status\":\"PASS\",\"overall_status\":\"PASS\"}"
    ));
}

#[test]
fn spec_c80_evidence_fail_path_sets_evidence_status_and_overall_fail() {
    let config = RunCommandConfig {
        mode: "sdk-direct".to_owned(),
        kolme_binary: "/tmp/kolme-node".to_owned(),
        agent_binary: None,
        external_execution: false,
        evidence_dir: "/tmp/evidence-fail".to_owned(),
        scenario_ids: vec!["S-01".to_owned(), "S-02".to_owned()],
    };
    let output = execute_run_contract(&config).expect("run output should render");
    assert!(output.contains(
        "\"evidence_contract\":{\"expected_artifacts\":4,\"recorded_artifacts\":3,\"status\":\"FAIL\"}"
    ));
    assert!(output.contains(
        "\"live_execution\":{\"orchestration_status\":\"PASS\",\"validation_status\":\"PASS\",\"evidence_status\":\"FAIL\",\"overall_status\":\"FAIL\"}"
    ));
}

#[test]
fn spec_c81_run_output_contains_mode_execution_contract_markers() {
    let config = RunCommandConfig {
        mode: "sdk-direct".to_owned(),
        kolme_binary: "/tmp/kolme-node".to_owned(),
        agent_binary: None,
        external_execution: false,
        evidence_dir: "/tmp/evidence".to_owned(),
        scenario_ids: vec!["S-01".to_owned(), "S-02".to_owned()],
    };
    let output = execute_run_contract(&config).expect("run output should render");
    assert!(output.contains("\"mode_execution_contract\":"));
    assert!(output.contains("\"mode\""));
    assert!(output.contains("\"driver\""));
    assert!(output.contains("\"selected_scenarios\""));
    assert!(output.contains("\"executed_scenarios\""));
    assert!(output.contains("\"status\""));
}

#[test]
fn spec_c82_mode_execution_contract_driver_marker_is_mode_coherent() {
    assert_mode_driver("sdk-direct", None, "sdk-direct-driver");
    assert_mode_driver("cli-scripted", None, "cli-scripted-driver");
    assert_mode_driver("mcp-tau", Some("/tmp/tau"), "mcp-agent-driver");
}

fn assert_mode_driver(mode: &str, agent_binary: Option<&str>, driver: &str) {
    let output = render_run(&run_config(mode, agent_binary, false, "/tmp/evidence", &["S-01"]));
    assert!(output.contains(&format!(
        "\"mode_execution_contract\":{{\"mode\":\"{mode}\",\"driver\":\"{driver}\""
    )));
}

#[test]
fn spec_c83_mode_execution_contract_executed_count_matches_scenario_count() {
    let config = RunCommandConfig {
        mode: "sdk-direct".to_owned(),
        kolme_binary: "/tmp/kolme-node".to_owned(),
        agent_binary: None,
        external_execution: false,
        evidence_dir: "/tmp/evidence".to_owned(),
        scenario_ids: vec!["S-03".to_owned(), "S-01".to_owned(), "S-15".to_owned()],
    };
    let output = execute_run_contract(&config).expect("run output should render");
    assert!(output.contains("\"scenario_count\":3"));
    assert!(output.contains(
        "\"mode_execution_contract\":{\"mode\":\"sdk-direct\",\"driver\":\"sdk-direct-driver\",\"selected_scenarios\":3,\"executed_scenarios\":3,\"status\":\"PASS\"}"
    ));
}

#[test]
fn spec_c84_evidence_phase_is_pass_on_normal_path() {
    let config = RunCommandConfig {
        mode: "sdk-direct".to_owned(),
        kolme_binary: "/tmp/kolme-node".to_owned(),
        agent_binary: None,
        external_execution: false,
        evidence_dir: "/tmp/evidence".to_owned(),
        scenario_ids: vec!["S-01".to_owned()],
    };
    let output = execute_run_contract(&config).expect("run output should render");
    assert!(output.contains("{\"phase\":\"EVIDENCE\",\"status\":\"PASS\""));
    assert!(output.contains("{\"step\":\"Dump Kolme chain state\",\"status\":\"PASS\""));
    assert!(output.contains("{\"step\":\"Dump KAMN node state snapshots\",\"status\":\"PASS\""));
    assert!(
        output.contains("{\"step\":\"Verify all proof anchors independently\",\"status\":\"PASS\"")
    );
    assert!(output.contains("{\"step\":\"Generate chain-of-custody report\",\"status\":\"PASS\""));
    assert!(output.contains("{\"step\":\"Compute evidence bundle hash\",\"status\":\"PASS\""));
    assert!(output.contains("{\"step\":\"Write manifest.json\",\"status\":\"PASS\""));
}

#[test]
fn spec_c85_evidence_phase_is_fail_on_evidence_fail_path() {
    let config = RunCommandConfig {
        mode: "sdk-direct".to_owned(),
        kolme_binary: "/tmp/kolme-node".to_owned(),
        agent_binary: None,
        external_execution: false,
        evidence_dir: "/tmp/evidence-fail".to_owned(),
        scenario_ids: vec!["S-01".to_owned()],
    };
    let output = execute_run_contract(&config).expect("run output should render");
    assert!(output.contains("{\"phase\":\"EVIDENCE\",\"status\":\"FAIL\""));
    assert!(
        output.contains("{\"step\":\"Verify all proof anchors independently\",\"status\":\"FAIL\"")
    );
    assert!(output.contains("{\"step\":\"Generate chain-of-custody report\",\"status\":\"FAIL\""));
    assert!(output.contains("{\"step\":\"Compute evidence bundle hash\",\"status\":\"FAIL\""));
    assert!(output.contains("{\"step\":\"Write manifest.json\",\"status\":\"FAIL\""));
}

#[test]
fn spec_c86_lifecycle_summary_reflects_evidence_phase_failure_transition() {
    let config = RunCommandConfig {
        mode: "sdk-direct".to_owned(),
        kolme_binary: "/tmp/kolme-node".to_owned(),
        agent_binary: None,
        external_execution: false,
        evidence_dir: "/tmp/evidence-fail".to_owned(),
        scenario_ids: vec!["S-01".to_owned()],
    };
    let output = execute_run_contract(&config).expect("run output should render");
    assert!(output.contains("\"phase_totals\":{\"total\":5,\"pass\":4,\"fail\":1,\"skip\":0}"));
    assert!(output.contains("\"step_totals\":{\"total\":27,\"pass\":20,\"fail\":4,\"skip\":3}"));
}
