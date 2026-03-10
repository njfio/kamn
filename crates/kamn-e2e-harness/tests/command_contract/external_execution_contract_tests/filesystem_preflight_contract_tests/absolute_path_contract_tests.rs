use super::*;

#[test]
fn spec_c65_external_execution_executable_regular_file_still_passes_preflight() {
    let kolme_binary = temp_path("kolme-node-exec-regular");
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
        execute_run_contract(&config).expect("executable regular file should pass preflight")
    });
    assert!(output.contains("\"runtime_external_execution\":{\"requested\":true"));
    let _ = std::fs::remove_file(kolme_binary);
}

#[test]
fn spec_c66_external_execution_relative_kolme_binary_returns_absolute_path_error() {
    let config = RunCommandConfig {
        mode: "sdk-direct".to_owned(),
        kolme_binary: "relative-kolme-node".to_owned(),
        agent_binary: None,
        external_execution: true,
        evidence_dir: "/tmp/evidence".to_owned(),
        scenario_ids: vec!["S-01".to_owned()],
    };
    let err = execute_run_contract(&config)
        .expect_err("relative kolme path should fail deterministic preflight");
    assert!(err.contains("external execution preflight failed"));
    assert!(err.contains("kolme binary path must be absolute"));
}

#[test]
fn spec_c67_external_execution_relative_agent_binary_returns_absolute_path_error() {
    let kolme_binary = temp_path("kolme-node-absolute");
    write_stub_binary(&kolme_binary);
    #[cfg(unix)]
    set_executable(&kolme_binary);

    let config = RunCommandConfig {
        mode: "mcp-tau".to_owned(),
        kolme_binary: kolme_binary.display().to_string(),
        agent_binary: Some("relative-agent-node".to_owned()),
        external_execution: true,
        evidence_dir: "/tmp/evidence".to_owned(),
        scenario_ids: vec!["S-01".to_owned()],
    };
    let err = execute_run_contract(&config)
        .expect_err("relative agent path should fail deterministic preflight");
    assert!(err.contains("external execution preflight failed"));
    assert!(err.contains("agent binary path must be absolute"));
    let _ = std::fs::remove_file(kolme_binary);
}

#[test]
fn spec_c68_external_execution_absolute_paths_still_pass_preflight() {
    let kolme_binary = temp_path("kolme-node-absolute-pass");
    write_stub_binary(&kolme_binary);
    #[cfg(unix)]
    set_executable(&kolme_binary);

    let agent_binary = temp_path("agent-node-absolute-pass");
    write_stub_binary(&agent_binary);
    #[cfg(unix)]
    set_executable(&agent_binary);

    let config = RunCommandConfig {
        mode: "mcp-tau".to_owned(),
        kolme_binary: kolme_binary.display().to_string(),
        agent_binary: Some(agent_binary.display().to_string()),
        external_execution: true,
        evidence_dir: "/tmp/evidence".to_owned(),
        scenario_ids: vec!["S-01".to_owned()],
    };
    let output = with_external_component_binaries(|| {
        execute_run_contract(&config).expect("absolute paths should pass preflight")
    });
    assert!(output.contains("\"runtime_external_execution\":{\"requested\":true"));
    let _ = std::fs::remove_file(kolme_binary);
    let _ = std::fs::remove_file(agent_binary);
}
