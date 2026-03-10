use super::*;

#[test]
fn spec_c69_integration_config_flags_map_for_sdk_direct_external_disabled() {
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
    assert!(output.contains("\"agent_binary_required\":false"));
    assert!(output.contains("\"external_execution_enabled\":false"));
}

#[test]
fn spec_c70_integration_config_flags_map_for_sdk_direct_external_enabled() {
    let kolme_binary = temp_path("kolme-node-config-map");
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
    assert!(output.contains("\"integration_config\":"));
    assert!(output.contains("\"agent_binary_required\":false"));
    assert!(output.contains("\"external_execution_enabled\":true"));
    let _ = std::fs::remove_file(kolme_binary);
}

#[test]
fn spec_c71_integration_config_flags_map_for_mcp_tau_external_enabled() {
    with_two_executable_binaries("kolme-node-mcp-map", "agent-node-mcp-map", |kolme, agent| {
        let config = RunCommandConfig {
            mode: "mcp-tau".to_owned(),
            kolme_binary: kolme.display().to_string(),
            agent_binary: Some(agent.display().to_string()),
            external_execution: true,
            evidence_dir: "/tmp/evidence".to_owned(),
            scenario_ids: vec!["S-01".to_owned()],
        };
        let output = with_external_component_binaries(|| render_run(&config));
        assert_contains_all(
            &output,
            &[
                "\"integration_config\":",
                "\"agent_binary_required\":true",
                "\"external_execution_enabled\":true",
            ],
        );
    });
}
