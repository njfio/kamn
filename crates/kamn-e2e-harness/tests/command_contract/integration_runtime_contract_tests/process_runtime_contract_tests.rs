use super::*;

#[test]
fn spec_c29_run_output_contains_process_runtime_markers() {
    let config = RunCommandConfig {
        mode: "sdk-direct".to_owned(),
        kolme_binary: "/tmp/kolme-node".to_owned(),
        agent_binary: None,
        external_execution: false,
        evidence_dir: "/tmp/evidence".to_owned(),
        scenario_ids: vec!["S-01".to_owned()],
    };
    let output = execute_run_contract(&config).expect("run output should render");
    assert!(output.contains("\"process_runtime\":"));
    assert!(output.contains("\"kolme_runtime\""));
    assert!(output.contains("\"kamn_nodes_runtime\""));
    assert!(output.contains("\"agent_runtime\""));
    assert!(output.contains("\"spawn_strategy\""));
}

#[test]
fn spec_c30_process_runtime_agent_runtime_is_sdk_direct_for_sdk_mode() {
    let config = RunCommandConfig {
        mode: "sdk-direct".to_owned(),
        kolme_binary: "/tmp/kolme-node".to_owned(),
        agent_binary: None,
        external_execution: false,
        evidence_dir: "/tmp/evidence".to_owned(),
        scenario_ids: vec!["S-01".to_owned()],
    };
    let output = execute_run_contract(&config).expect("run output should render");
    assert!(output.contains("\"process_runtime\":{\"kolme_runtime\":\"external-binary\",\"kamn_nodes_runtime\":\"managed-process-set\",\"agent_runtime\":\"sdk-direct\""));
}

#[test]
fn spec_c31_process_runtime_agent_runtime_is_cli_scripted_for_cli_mode() {
    let config = RunCommandConfig {
        mode: "cli-scripted".to_owned(),
        kolme_binary: "/tmp/kolme-node".to_owned(),
        agent_binary: None,
        external_execution: false,
        evidence_dir: "/tmp/evidence".to_owned(),
        scenario_ids: vec!["S-01".to_owned()],
    };
    let output = execute_run_contract(&config).expect("run output should render");
    assert!(output.contains("\"agent_runtime\":\"cli-scripted\""));
}

#[test]
fn spec_c32_process_runtime_agent_runtime_is_mcp_agent_for_mcp_mode() {
    let config = RunCommandConfig {
        mode: "mcp-tau".to_owned(),
        kolme_binary: "/tmp/kolme-node".to_owned(),
        agent_binary: Some("/tmp/tau".to_owned()),
        external_execution: false,
        evidence_dir: "/tmp/evidence".to_owned(),
        scenario_ids: vec!["S-01".to_owned()],
    };
    let output = execute_run_contract(&config).expect("run output should render");
    assert!(output.contains("\"agent_runtime\":\"mcp-agent\""));
}

#[test]
fn spec_c33_run_output_contains_process_lifecycle_service_markers() {
    let config = RunCommandConfig {
        mode: "sdk-direct".to_owned(),
        kolme_binary: "/tmp/kolme-node".to_owned(),
        agent_binary: None,
        external_execution: false,
        evidence_dir: "/tmp/evidence".to_owned(),
        scenario_ids: vec!["S-01".to_owned()],
    };
    let output = execute_run_contract(&config).expect("run output should render");
    assert!(output.contains("\"process_lifecycle\":"));
    assert!(output.contains("\"postgres\""));
    assert!(output.contains("\"kolme\""));
    assert!(output.contains("\"kamn_processor\""));
    assert!(output.contains("\"kamn_listener\""));
    assert!(output.contains("\"kamn_approver\""));
}

#[test]
fn spec_c34_process_lifecycle_service_markers_are_planned() {
    let config = RunCommandConfig {
        mode: "sdk-direct".to_owned(),
        kolme_binary: "/tmp/kolme-node".to_owned(),
        agent_binary: None,
        external_execution: false,
        evidence_dir: "/tmp/evidence".to_owned(),
        scenario_ids: vec!["S-01".to_owned()],
    };
    let output = execute_run_contract(&config).expect("run output should render");
    assert!(
        output.contains("\"process_lifecycle\":{\"postgres\":\"planned\",\"kolme\":\"planned\",\"kamn_processor\":\"planned\",\"kamn_listener\":\"planned\",\"kamn_approver\":\"planned\"}")
    );
}

#[test]
fn spec_c35_run_output_contains_spawn_timeline_markers() {
    let config = RunCommandConfig {
        mode: "sdk-direct".to_owned(),
        kolme_binary: "/tmp/kolme-node".to_owned(),
        agent_binary: None,
        external_execution: false,
        evidence_dir: "/tmp/evidence".to_owned(),
        scenario_ids: vec!["S-01".to_owned()],
    };
    let output = execute_run_contract(&config).expect("run output should render");
    assert!(output.contains("\"spawn_timeline\":"));
    assert!(output.contains("\"postgres_start\""));
    assert!(output.contains("\"kolme_start\""));
    assert!(output.contains("\"kamn_nodes_start\""));
    assert!(output.contains("\"agent_deploy_start\""));
}

#[test]
fn spec_c36_spawn_timeline_markers_follow_canonical_ordering() {
    let config = RunCommandConfig {
        mode: "sdk-direct".to_owned(),
        kolme_binary: "/tmp/kolme-node".to_owned(),
        agent_binary: None,
        external_execution: false,
        evidence_dir: "/tmp/evidence".to_owned(),
        scenario_ids: vec!["S-01".to_owned()],
    };
    let output = execute_run_contract(&config).expect("run output should render");
    assert!(
        output.contains("\"spawn_timeline\":{\"postgres_start\":\"step-1\",\"kolme_start\":\"step-2\",\"kamn_nodes_start\":\"step-3\",\"agent_deploy_start\":\"step-4\"}")
    );
}

#[test]
fn spec_c37_run_output_contains_live_validation_markers() {
    let config = RunCommandConfig {
        mode: "sdk-direct".to_owned(),
        kolme_binary: "/tmp/kolme-node".to_owned(),
        agent_binary: None,
        external_execution: false,
        evidence_dir: "/tmp/evidence".to_owned(),
        scenario_ids: vec!["S-01".to_owned()],
    };
    let output = execute_run_contract(&config).expect("run output should render");
    assert!(output.contains("\"live_validation\":"));
    assert!(output.contains("\"expected_checks\""));
    assert!(output.contains("\"completed_checks\""));
    assert!(output.contains("\"status\""));
}

#[test]
fn spec_c38_live_validation_markers_are_deterministic_and_coherent() {
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
        "\"live_validation\":{\"expected_checks\":4,\"completed_checks\":4,\"status\":\"PASS\"}"
    ));
}
