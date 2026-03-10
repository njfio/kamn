use super::*;

#[test]
fn spec_c87_teardown_phase_is_pass_with_prd_step_inventory_in_sdk_mode() {
    let config = RunCommandConfig {
        mode: "sdk-direct".to_owned(),
        kolme_binary: "/tmp/kolme-node".to_owned(),
        agent_binary: None,
        external_execution: false,
        evidence_dir: "/tmp/evidence".to_owned(),
        scenario_ids: vec!["S-01".to_owned()],
    };
    let output = execute_run_contract(&config).expect("run output should render");
    assert!(output.contains("{\"phase\":\"TEARDOWN\",\"status\":\"PASS\""));
    assert!(output
        .contains("{\"step\":\"[MCP modes] Stop kamn-mcp-server processes\",\"status\":\"SKIP\""));
    assert!(
        output.contains("{\"step\":\"Stop KAMN nodes (graceful shutdown)\",\"status\":\"PASS\"")
    );
    assert!(output.contains("{\"step\":\"Stop Kolme devnet\",\"status\":\"PASS\""));
    assert!(output.contains("{\"step\":\"Stop PostgreSQL container\",\"status\":\"PASS\""));
    assert!(output.contains("{\"step\":\"Archive evidence bundle\",\"status\":\"PASS\""));
}

#[test]
fn spec_c88_teardown_phase_marks_mcp_stop_step_pass_in_mcp_mode() {
    let config = RunCommandConfig {
        mode: "mcp-tau".to_owned(),
        kolme_binary: "/tmp/kolme-node".to_owned(),
        agent_binary: Some("/tmp/tau".to_owned()),
        external_execution: false,
        evidence_dir: "/tmp/evidence".to_owned(),
        scenario_ids: vec!["S-01".to_owned()],
    };
    let output = execute_run_contract(&config).expect("run output should render");
    assert!(output.contains("{\"phase\":\"TEARDOWN\",\"status\":\"PASS\""));
    assert!(output
        .contains("{\"step\":\"[MCP modes] Stop kamn-mcp-server processes\",\"status\":\"PASS\""));
}

#[test]
fn spec_c89_lifecycle_summary_reflects_teardown_phase_activation_on_normal_path() {
    let config = RunCommandConfig {
        mode: "sdk-direct".to_owned(),
        kolme_binary: "/tmp/kolme-node".to_owned(),
        agent_binary: None,
        external_execution: false,
        evidence_dir: "/tmp/evidence".to_owned(),
        scenario_ids: vec!["S-01".to_owned()],
    };
    let output = execute_run_contract(&config).expect("run output should render");
    assert!(output.contains("\"phase_totals\":{\"total\":5,\"pass\":5,\"fail\":0,\"skip\":0}"));
    assert!(output.contains("\"step_totals\":{\"total\":27,\"pass\":24,\"fail\":0,\"skip\":3}"));
}

#[test]
fn spec_c90_evidence_phase_step_inventory_includes_all_prd_labels() {
    let config = RunCommandConfig {
        mode: "sdk-direct".to_owned(),
        kolme_binary: "/tmp/kolme-node".to_owned(),
        agent_binary: None,
        external_execution: false,
        evidence_dir: "/tmp/evidence".to_owned(),
        scenario_ids: vec!["S-01".to_owned()],
    };
    let output = execute_run_contract(&config).expect("run output should render");
    assert!(output.contains("\"step\":\"Dump Kolme chain state\""));
    assert!(output.contains("\"step\":\"Dump KAMN node state snapshots\""));
    assert!(output.contains("\"step\":\"Verify all proof anchors independently\""));
    assert!(output.contains("\"step\":\"Generate chain-of-custody report\""));
    assert!(output.contains("\"step\":\"Compute evidence bundle hash\""));
    assert!(output.contains("\"step\":\"Write manifest.json\""));
}
