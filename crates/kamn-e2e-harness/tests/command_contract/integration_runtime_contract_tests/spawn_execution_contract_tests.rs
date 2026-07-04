use super::*;

#[test]
fn spec_c39_run_output_contains_spawn_plan_markers() {
    let config = RunCommandConfig {
        mode: "sdk-direct".to_owned(),
        kolme_binary: "/tmp/kolme-node".to_owned(),
        agent_binary: None,
        external_execution: false,
        evidence_dir: "/tmp/evidence".to_owned(),
        scenario_ids: vec!["S-01".to_owned()],
    };
    let output = execute_run_contract(&config).expect("run output should render");
    assert!(output.contains("\"spawn_plan\":"));
    assert!(output.contains("\"postgres_cmd\""));
    assert!(output.contains("\"kolme_cmd\""));
    assert!(output.contains("\"kamn_processor_cmd\""));
    assert!(output.contains("\"kamn_listener_cmd\""));
    assert!(output.contains("\"kamn_approver_cmd\""));
}

#[test]
fn spec_c40_spawn_plan_markers_are_deterministic_and_mode_coherent() {
    assert_spawn_plan_markers(
        &render_run(&run_config(
            "sdk-direct",
            None,
            false,
            "/tmp/evidence",
            &["S-01"],
        )),
        &[
            "\"postgres_cmd\":\"docker run --rm --name kamn-e2e-postgres postgres:15\"",
            "\"kamn_processor_cmd\":\"kamn-node --role processor --execution-mode sdk-direct\"",
        ],
    );
    assert_spawn_plan_markers(
        &render_run(&run_config(
            "mcp-tau",
            Some("/tmp/tau"),
            false,
            "/tmp/evidence",
            &["S-01"],
        )),
        &["\"kamn_processor_cmd\":\"kamn-node --role processor --execution-mode mcp-tau\""],
    );
}

fn assert_spawn_plan_markers(output: &str, markers: &[&str]) {
    assert_contains_all(output, markers);
}

#[test]
fn spec_c41_run_output_contains_spawn_execution_markers() {
    let config = RunCommandConfig {
        mode: "sdk-direct".to_owned(),
        kolme_binary: "/tmp/kolme-node".to_owned(),
        agent_binary: None,
        external_execution: false,
        evidence_dir: "/tmp/evidence".to_owned(),
        scenario_ids: vec!["S-01".to_owned()],
    };
    let output = execute_run_contract(&config).expect("run output should render");
    assert!(output.contains("\"spawn_execution\":"));
    assert!(output.contains("\"postgres\""));
    assert!(output.contains("\"kolme\""));
    assert!(output.contains("\"kamn_processor\""));
    assert!(output.contains("\"kamn_listener\""));
    assert!(output.contains("\"kamn_approver\""));
    assert!(output.contains("\"timeline_ref\""));
    assert!(output.contains("\"result\""));
}

#[test]
fn spec_c42_spawn_execution_markers_are_deterministic_and_status_coherent() {
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
        "\"spawn_execution\":{\"postgres\":{\"status\":\"PASS\",\"timeline_ref\":\"step-1\",\"result\":\"started\"},\"kolme\":{\"status\":\"PASS\",\"timeline_ref\":\"step-2\",\"result\":\"started\"},\"kamn_processor\":{\"status\":\"PASS\",\"timeline_ref\":\"step-3\",\"result\":\"started\"},\"kamn_listener\":{\"status\":\"PASS\",\"timeline_ref\":\"step-3\",\"result\":\"started\"},\"kamn_approver\":{\"status\":\"PASS\",\"timeline_ref\":\"step-3\",\"result\":\"started\"}}"
    ));
}

#[test]
fn spec_c43_run_output_contains_live_process_execution_markers() {
    let config = RunCommandConfig {
        mode: "sdk-direct".to_owned(),
        kolme_binary: "/tmp/kolme-node".to_owned(),
        agent_binary: None,
        external_execution: false,
        evidence_dir: "/tmp/evidence".to_owned(),
        scenario_ids: vec!["S-01".to_owned()],
    };
    let output = execute_run_contract(&config).expect("run output should render");
    assert!(output.contains("\"live_process_execution\":"));
    assert!(output.contains("\"postgres\""));
    assert!(output.contains("\"kolme\""));
    assert!(output.contains("\"kamn_processor\""));
    assert!(output.contains("\"kamn_listener\""));
    assert!(output.contains("\"kamn_approver\""));
    assert!(output.contains("\"state\""));
    assert!(output.contains("\"pid\""));
    assert!(output.contains("\"health\""));
}

#[test]
fn spec_c44_live_process_execution_markers_are_deterministic_and_coherent() {
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
        "\"live_process_execution\":{\"postgres\":{\"state\":\"running\",\"pid\":\"1001\",\"health\":\"PASS\"},\"kolme\":{\"state\":\"running\",\"pid\":\"1002\",\"health\":\"PASS\"},\"kamn_processor\":{\"state\":\"running\",\"pid\":\"2001\",\"health\":\"PASS\"},\"kamn_listener\":{\"state\":\"running\",\"pid\":\"2002\",\"health\":\"PASS\"},\"kamn_approver\":{\"state\":\"running\",\"pid\":\"2003\",\"health\":\"PASS\"}}"
    ));
}

#[test]
fn spec_c45_run_output_contains_live_execution_markers() {
    let config = RunCommandConfig {
        mode: "sdk-direct".to_owned(),
        kolme_binary: "/tmp/kolme-node".to_owned(),
        agent_binary: None,
        external_execution: false,
        evidence_dir: "/tmp/evidence".to_owned(),
        scenario_ids: vec!["S-01".to_owned()],
    };
    let output = execute_run_contract(&config).expect("run output should render");
    assert!(output.contains("\"live_execution\":"));
    assert!(output.contains("\"orchestration_status\""));
    assert!(output.contains("\"validation_status\""));
    assert!(output.contains("\"evidence_status\""));
    assert!(output.contains("\"overall_status\""));
}

#[test]
fn spec_c46_live_execution_markers_are_deterministic_and_coherent() {
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
        "\"live_execution\":{\"orchestration_status\":\"PASS\",\"validation_status\":\"PASS\",\"evidence_status\":\"PASS\",\"overall_status\":\"PASS\"}"
    ));
}
