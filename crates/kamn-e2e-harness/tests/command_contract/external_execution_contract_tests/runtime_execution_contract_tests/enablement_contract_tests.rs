use super::*;

#[test]
fn spec_c47_parser_accepts_external_execution_flag() {
    let parsed = parse_command_args([
        "run",
        "--mode",
        "sdk-direct",
        "--kolme-binary",
        "/tmp/kolme-node",
        "--enable-external-execution",
        "--evidence-dir",
        "/tmp/evidence",
        "--scenarios",
        "S-01",
    ])
    .expect("run command should parse with external execution flag");
    let HarnessCommand::Run(config) = parsed else {
        panic!("expected run command");
    };
    assert!(config.external_execution);
}

#[test]
fn spec_c48_run_output_contains_runtime_external_execution_markers() {
    let config = RunCommandConfig {
        mode: "sdk-direct".to_owned(),
        kolme_binary: "/tmp/kolme-node".to_owned(),
        agent_binary: None,
        external_execution: false,
        evidence_dir: "/tmp/evidence".to_owned(),
        scenario_ids: vec!["S-01".to_owned()],
    };
    let output = execute_run_contract(&config).expect("run output should render");
    assert!(output.contains("\"runtime_external_execution\":"));
    assert!(output.contains("\"requested\""));
    assert!(output.contains("\"guard_status\""));
    assert!(output.contains("\"execution_mode\""));
    assert!(output.contains("\"preflight\""));
}

#[test]
fn spec_c49_external_execution_missing_kolme_binary_returns_deterministic_error() {
    let config = RunCommandConfig {
        mode: "sdk-direct".to_owned(),
        kolme_binary: "/tmp/does-not-exist-kolme".to_owned(),
        agent_binary: None,
        external_execution: true,
        evidence_dir: "/tmp/evidence".to_owned(),
        scenario_ids: vec!["S-01".to_owned()],
    };
    let err =
        execute_run_contract(&config).expect_err("missing kolme binary should fail preflight");
    assert!(err.contains("external execution preflight failed"));
    assert!(err.contains("kolme binary not found"));
}

#[test]
fn spec_c50_external_execution_missing_agent_binary_in_mcp_mode_is_deterministic_error() {
    let config = RunCommandConfig {
        mode: "mcp-tau".to_owned(),
        kolme_binary: "/tmp/does-not-exist-kolme".to_owned(),
        agent_binary: Some("/tmp/does-not-exist-agent".to_owned()),
        external_execution: true,
        evidence_dir: "/tmp/evidence".to_owned(),
        scenario_ids: vec!["S-01".to_owned()],
    };
    let err =
        execute_run_contract(&config).expect_err("missing agent binary should fail preflight");
    assert!(err.contains("external execution preflight failed"));
}

#[test]
fn spec_c51_run_output_contains_runtime_orchestration_markers() {
    let config = RunCommandConfig {
        mode: "sdk-direct".to_owned(),
        kolme_binary: "/tmp/kolme-node".to_owned(),
        agent_binary: None,
        external_execution: false,
        evidence_dir: "/tmp/evidence".to_owned(),
        scenario_ids: vec!["S-01".to_owned()],
    };
    let output = execute_run_contract(&config).expect("run output should render");
    assert!(output.contains("\"runtime_orchestration\":"));
    assert!(output.contains("\"postgres\""));
    assert!(output.contains("\"kolme\""));
    assert!(output.contains("\"kamn_processor\""));
    assert!(output.contains("\"kamn_listener\""));
    assert!(output.contains("\"kamn_approver\""));
    assert!(output.contains("\"requested\""));
    assert!(output.contains("\"status\""));
    assert!(output.contains("\"detail\""));
}

#[test]
fn spec_c52_runtime_orchestration_markers_skip_when_external_disabled() {
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
fn spec_c53_runtime_orchestration_markers_pass_when_external_enabled() {
    let kolme_binary = temp_path("kolme-node");
    write_stub_binary(&kolme_binary);
    #[cfg(unix)]
    set_executable(&kolme_binary);
    let config = RunCommandConfig {
        mode: "sdk-direct".to_owned(),
        kolme_binary: kolme_binary.display().to_string(),
        agent_binary: None,
        external_execution: true,
        evidence_dir: "/tmp/evidence".to_owned(),
        scenario_ids: vec!["S-01".to_owned()],
    };
    let output = with_external_component_binaries(|| {
        execute_run_contract(&config).expect("run output should render")
    });
    assert!(output.contains("\"runtime_orchestration\":"));
    assert!(output.contains("\"postgres\":{\"requested\":true,\"status\":\"PASS\""));
    assert!(output.contains("\"kolme\":{\"requested\":true,\"status\":\"PASS\""));
    assert!(output.contains("\"kamn_processor\":{\"requested\":true,\"status\":\"PASS\""));
    assert!(output.contains("\"kamn_listener\":{\"requested\":true,\"status\":\"PASS\""));
    assert!(output.contains("\"kamn_approver\":{\"requested\":true,\"status\":\"PASS\""));

    let _ = std::fs::remove_file(kolme_binary);
}
