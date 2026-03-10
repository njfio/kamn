use super::*;

#[test]
fn spec_c21_parser_accepts_sdk_direct_with_kolme_binary_only() {
    let parsed = parse_command_args([
        "run",
        "--mode",
        "sdk-direct",
        "--kolme-binary",
        "/tmp/kolme-node",
        "--evidence-dir",
        "/tmp/evidence",
        "--scenarios",
        "S-01,S-02",
    ])
    .expect("sdk-direct should parse with kolme binary only");
    let expected = HarnessCommand::Run(RunCommandConfig {
        mode: "sdk-direct".to_owned(),
        kolme_binary: "/tmp/kolme-node".to_owned(),
        agent_binary: None,
        external_execution: false,
        evidence_dir: "/tmp/evidence".to_owned(),
        scenario_ids: vec!["S-01".to_owned(), "S-02".to_owned()],
    });
    assert_eq!(parsed, expected);
}

#[test]
fn spec_c22_parser_accepts_mcp_tau_with_agent_binary() {
    let parsed = parse_command_args([
        "run",
        "--mode",
        "mcp-tau",
        "--kolme-binary",
        "/tmp/kolme-node",
        "--agent-binary",
        "/tmp/tau",
        "--evidence-dir",
        "/tmp/evidence",
        "--scenarios",
        "S-01",
    ])
    .expect("mcp-tau should parse with both runtime binaries");
    let expected = HarnessCommand::Run(RunCommandConfig {
        mode: "mcp-tau".to_owned(),
        kolme_binary: "/tmp/kolme-node".to_owned(),
        agent_binary: Some("/tmp/tau".to_owned()),
        external_execution: false,
        evidence_dir: "/tmp/evidence".to_owned(),
        scenario_ids: vec!["S-01".to_owned()],
    });
    assert_eq!(parsed, expected);
}

#[test]
fn spec_c23_parser_rejects_mcp_any_without_agent_binary() {
    let err = parse_command_args([
        "run",
        "--mode",
        "mcp-any",
        "--kolme-binary",
        "/tmp/kolme-node",
        "--evidence-dir",
        "/tmp/evidence",
        "--scenarios",
        "S-01",
    ])
    .expect_err("mcp-any without agent binary should fail");
    assert!(err.contains("missing required flag --agent-binary for MCP modes"));
}

#[test]
fn spec_c24_run_output_contains_integration_config_markers() {
    let config = RunCommandConfig {
        mode: "sdk-direct".to_owned(),
        kolme_binary: "/tmp/kolme-node".to_owned(),
        agent_binary: None,
        external_execution: false,
        evidence_dir: "/tmp/evidence".to_owned(),
        scenario_ids: vec!["S-01".to_owned()],
    };
    let output = execute_run_contract(&config).expect("run output should render");
    assert!(output.contains("\"integration_config\":"));
    assert!(output.contains("\"kolme_binary\""));
    assert!(output.contains("\"agent_binary\""));
    assert!(output.contains("\"agent_binary_required\":false"));
}

#[test]
fn spec_c25_run_output_contains_runtime_readiness_markers() {
    let config = RunCommandConfig {
        mode: "sdk-direct".to_owned(),
        kolme_binary: "/tmp/kolme-node".to_owned(),
        agent_binary: None,
        external_execution: false,
        evidence_dir: "/tmp/evidence".to_owned(),
        scenario_ids: vec!["S-01".to_owned()],
    };
    let output = execute_run_contract(&config).expect("run output should render");
    assert!(output.contains("\"runtime_readiness\":"));
    assert!(output.contains("\"kolme_binary\":\"PASS\""));
    assert!(output.contains("\"agent_binary\""));
    assert!(output.contains("\"scenario_selection\":\"PASS\""));
    assert!(output.contains("\"overall\":\"PASS\""));
}

#[test]
fn spec_c26_sdk_direct_runtime_readiness_marks_agent_binary_skip() {
    let config = RunCommandConfig {
        mode: "sdk-direct".to_owned(),
        kolme_binary: "/tmp/kolme-node".to_owned(),
        agent_binary: None,
        external_execution: false,
        evidence_dir: "/tmp/evidence".to_owned(),
        scenario_ids: vec!["S-01".to_owned()],
    };
    let output = execute_run_contract(&config).expect("run output should render");
    assert!(output
        .contains("\"runtime_readiness\":{\"kolme_binary\":\"PASS\",\"agent_binary\":\"SKIP\""));
    assert!(output.contains("\"overall\":\"PASS\""));
}

#[test]
fn spec_c27_mcp_tau_runtime_readiness_marks_agent_binary_pass() {
    let config = RunCommandConfig {
        mode: "mcp-tau".to_owned(),
        kolme_binary: "/tmp/kolme-node".to_owned(),
        agent_binary: Some("/tmp/tau".to_owned()),
        external_execution: false,
        evidence_dir: "/tmp/evidence".to_owned(),
        scenario_ids: vec!["S-01".to_owned()],
    };
    let output = execute_run_contract(&config).expect("run output should render");
    assert!(output
        .contains("\"runtime_readiness\":{\"kolme_binary\":\"PASS\",\"agent_binary\":\"PASS\""));
    assert!(output.contains("\"overall\":\"PASS\""));
}

#[test]
fn spec_c28_mcp_any_without_agent_binary_returns_deterministic_error() {
    let config = RunCommandConfig {
        mode: "mcp-any".to_owned(),
        kolme_binary: "/tmp/kolme-node".to_owned(),
        agent_binary: None,
        external_execution: false,
        evidence_dir: "/tmp/evidence".to_owned(),
        scenario_ids: vec!["S-01".to_owned()],
    };
    let err = execute_run_contract(&config).expect_err("mcp-any missing agent binary should fail");
    assert!(err.contains("missing required agent binary for MCP modes"));
}
