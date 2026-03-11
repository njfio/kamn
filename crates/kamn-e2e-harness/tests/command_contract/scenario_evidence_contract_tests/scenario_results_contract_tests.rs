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
fn spec_c72a_run_output_preserves_sdk_direct_s14_failure_detail() {
    let output = run_sdk_direct_s14_failure_output();
    assert_s14_failure_detail_present(output.as_str());
}

fn run_sdk_direct_s14_failure_output() -> String {
    let kolme_binary = temp_path("kolme-node-s14-detail");
    write_stub_binary(&kolme_binary);
    #[cfg(unix)]
    set_executable(&kolme_binary);
    let config = s14_failure_config(kolme_binary.display().to_string());
    let output = with_s14_failure_env(|| {
        with_external_component_binaries(|| {
            execute_run_contract(&config).expect("run output should render")
        })
    });
    let _ = std::fs::remove_file(kolme_binary);
    output
}

fn s14_failure_config(kolme_binary: String) -> RunCommandConfig {
    RunCommandConfig {
        mode: "sdk-direct".to_owned(),
        kolme_binary,
        agent_binary: None,
        external_execution: true,
        evidence_dir: "/tmp/evidence".to_owned(),
        scenario_ids: vec!["S-14".to_owned()],
    }
}

fn assert_s14_failure_detail_present(output: &str) {
    assert!(
        output.contains("\"scenario_results\":[{\"id\":\"S-14\",\"status\":\"FAIL\",\"detail\":\"sdk-direct live s14"),
        "scenario_results payload should retain failing S-14 detail: {output}"
    );
}

fn with_s14_failure_env<T>(f: impl FnOnce() -> T) -> T {
    let previous_block_height = std::env::var("KAMN_E2E_S14_BLOCK_HEIGHT").ok();
    let previous_live_toggle = std::env::var("KAMN_E2E_SDK_DIRECT_LIVE").ok();
    std::env::set_var("KAMN_E2E_SDK_DIRECT_LIVE", "1");
    std::env::set_var("KAMN_E2E_S14_BLOCK_HEIGHT", "0");
    let result = f();
    restore_env_var("KAMN_E2E_S14_BLOCK_HEIGHT", previous_block_height);
    restore_env_var("KAMN_E2E_SDK_DIRECT_LIVE", previous_live_toggle);
    result
}

fn restore_env_var(key: &str, value: Option<String>) {
    if let Some(value) = value {
        std::env::set_var(key, value);
    } else {
        std::env::remove_var(key);
    }
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
