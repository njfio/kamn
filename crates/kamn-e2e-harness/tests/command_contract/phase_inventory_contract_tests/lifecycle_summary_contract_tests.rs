use super::*;

#[test]
fn spec_c16_mcp_steps_are_skipped_in_sdk_direct_mode() {
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
        "{\"step\":\"[MCP modes] Spawn kamn-mcp-server per agent with identity\",\"status\":\"SKIP\""
    ));
    assert!(
        output.contains("{\"step\":\"[MCP modes] Verify MCP server health\",\"status\":\"SKIP\"")
    );
}

#[test]
fn spec_c17_mcp_steps_are_pass_in_mcp_tau_mode() {
    let config = RunCommandConfig {
        mode: "mcp-tau".to_owned(),
        kolme_binary: "/tmp/kolme-node".to_owned(),
        agent_binary: Some("/tmp/tau".to_owned()),
        external_execution: false,
        evidence_dir: "/tmp/evidence".to_owned(),
        scenario_ids: vec!["S-01".to_owned()],
    };
    let output = execute_run_contract(&config).expect("run output should render");
    assert!(output.contains(
        "{\"step\":\"[MCP modes] Spawn kamn-mcp-server per agent with identity\",\"status\":\"PASS\""
    ));
    assert!(
        output.contains("{\"step\":\"[MCP modes] Verify MCP server health\",\"status\":\"PASS\"")
    );
}

#[test]
fn spec_c18_fail_path_marks_infra_health_step_and_phase_fail() {
    let config = RunCommandConfig {
        mode: "sdk-direct".to_owned(),
        kolme_binary: "/tmp/kolme-node".to_owned(),
        agent_binary: None,
        external_execution: false,
        evidence_dir: "/tmp/fail-path".to_owned(),
        scenario_ids: vec!["S-01".to_owned()],
    };
    let output = execute_run_contract(&config).expect("run output should render");
    assert!(output
        .contains("{\"step\":\"Verify KAMN Service API health (/healthz)\",\"status\":\"FAIL\""));
    assert!(output.contains("{\"phase\":\"INFRA_UP\",\"status\":\"FAIL\""));
}

#[test]
fn spec_c19_run_output_contains_lifecycle_summary_totals() {
    let config = RunCommandConfig {
        mode: "sdk-direct".to_owned(),
        kolme_binary: "/tmp/kolme-node".to_owned(),
        agent_binary: None,
        external_execution: false,
        evidence_dir: "/tmp/evidence".to_owned(),
        scenario_ids: vec!["S-01".to_owned()],
    };
    let output = execute_run_contract(&config).expect("run output should render");
    assert!(output.contains("\"lifecycle_summary\":"));
    assert!(output.contains("\"phase_totals\":"));
    assert!(output.contains("\"step_totals\":"));
    assert!(output.contains("\"total\":5"));
}

#[test]
fn spec_c20_lifecycle_summary_is_deterministic_for_normal_and_fail_path_runs() {
    let normal = RunCommandConfig {
        mode: "sdk-direct".to_owned(),
        kolme_binary: "/tmp/kolme-node".to_owned(),
        agent_binary: None,
        external_execution: false,
        evidence_dir: "/tmp/evidence".to_owned(),
        scenario_ids: vec!["S-01".to_owned()],
    };
    let normal_output = execute_run_contract(&normal).expect("normal run should render");
    assert!(
        normal_output.contains("\"phase_totals\":{\"total\":5,\"pass\":5,\"fail\":0,\"skip\":0}")
    );
    assert!(
        normal_output.contains("\"step_totals\":{\"total\":27,\"pass\":24,\"fail\":0,\"skip\":3}")
    );

    let fail = RunCommandConfig {
        mode: "sdk-direct".to_owned(),
        kolme_binary: "/tmp/kolme-node".to_owned(),
        agent_binary: None,
        external_execution: false,
        evidence_dir: "/tmp/fail-path".to_owned(),
        scenario_ids: vec!["S-01".to_owned()],
    };
    let fail_output = execute_run_contract(&fail).expect("fail-path run should render");
    assert!(fail_output.contains("\"phase_totals\":{\"total\":5,\"pass\":4,\"fail\":1,\"skip\":0}"));
    assert!(
        fail_output.contains("\"step_totals\":{\"total\":27,\"pass\":23,\"fail\":1,\"skip\":3}")
    );
}
