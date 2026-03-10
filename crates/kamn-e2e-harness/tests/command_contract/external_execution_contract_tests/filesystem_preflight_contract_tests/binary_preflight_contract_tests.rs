use super::*;

#[test]
fn spec_c60_external_execution_non_executable_kolme_binary_returns_deterministic_error() {
    let kolme_binary = temp_path("kolme-node-non-exec");
    write_stub_binary(&kolme_binary);
    #[cfg(unix)]
    set_non_executable(&kolme_binary);
    let config = RunCommandConfig {
        mode: "sdk-direct".to_owned(),
        kolme_binary: kolme_binary.display().to_string(),
        agent_binary: None,
        external_execution: true,
        evidence_dir: "/tmp/evidence".to_owned(),
        scenario_ids: vec!["S-01".to_owned()],
    };
    let err = execute_run_contract(&config)
        .expect_err("non-executable kolme binary should fail deterministic preflight");
    assert!(err.contains("external execution preflight failed"));
    assert!(err.contains("kolme binary is not executable"));
    let _ = std::fs::remove_file(kolme_binary);
}

#[test]
fn spec_c61_external_execution_non_executable_agent_binary_returns_deterministic_error() {
    let kolme_binary = temp_path("kolme-node-exec");
    write_stub_binary(&kolme_binary);
    #[cfg(unix)]
    set_executable(&kolme_binary);

    let agent_binary = temp_path("agent-node-non-exec");
    write_stub_binary(&agent_binary);
    #[cfg(unix)]
    set_non_executable(&agent_binary);

    let config = RunCommandConfig {
        mode: "mcp-tau".to_owned(),
        kolme_binary: kolme_binary.display().to_string(),
        agent_binary: Some(agent_binary.display().to_string()),
        external_execution: true,
        evidence_dir: "/tmp/evidence".to_owned(),
        scenario_ids: vec!["S-01".to_owned()],
    };
    let err = execute_run_contract(&config)
        .expect_err("non-executable agent binary should fail deterministic preflight");
    assert!(err.contains("external execution preflight failed"));
    assert!(err.contains("agent binary is not executable"));
    let _ = std::fs::remove_file(kolme_binary);
    let _ = std::fs::remove_file(agent_binary);
}

#[test]
fn spec_c62_external_execution_executable_binaries_pass_preflight() {
    let kolme_binary = temp_path("kolme-node-exec-pass");
    write_stub_binary(&kolme_binary);
    #[cfg(unix)]
    set_executable(&kolme_binary);

    let agent_binary = temp_path("agent-node-exec-pass");
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
        execute_run_contract(&config).expect("executable binaries should pass preflight")
    });
    assert!(output.contains("\"runtime_external_execution\":{\"requested\":true"));
    let _ = std::fs::remove_file(kolme_binary);
    let _ = std::fs::remove_file(agent_binary);
}

#[test]
fn spec_c63_external_execution_directory_kolme_binary_returns_non_file_error() {
    let kolme_binary_dir = temp_path("kolme-dir");
    std::fs::create_dir_all(&kolme_binary_dir).expect("directory path should be created");
    let config = RunCommandConfig {
        mode: "sdk-direct".to_owned(),
        kolme_binary: kolme_binary_dir.display().to_string(),
        agent_binary: None,
        external_execution: true,
        evidence_dir: "/tmp/evidence".to_owned(),
        scenario_ids: vec!["S-01".to_owned()],
    };
    let err = execute_run_contract(&config)
        .expect_err("directory kolm path should fail deterministic preflight");
    assert!(err.contains("external execution preflight failed"));
    assert!(err.contains("kolme binary path is not a file"));
    let _ = std::fs::remove_dir_all(kolme_binary_dir);
}

#[test]
fn spec_c64_external_execution_directory_agent_binary_returns_non_file_error() {
    let kolme_binary = temp_path("kolme-node-exec-file");
    write_stub_binary(&kolme_binary);
    #[cfg(unix)]
    set_executable(&kolme_binary);

    let agent_binary_dir = temp_path("agent-dir");
    std::fs::create_dir_all(&agent_binary_dir).expect("directory path should be created");
    let config = RunCommandConfig {
        mode: "mcp-tau".to_owned(),
        kolme_binary: kolme_binary.display().to_string(),
        agent_binary: Some(agent_binary_dir.display().to_string()),
        external_execution: true,
        evidence_dir: "/tmp/evidence".to_owned(),
        scenario_ids: vec!["S-01".to_owned()],
    };
    let err = execute_run_contract(&config)
        .expect_err("directory agent path should fail deterministic preflight");
    assert!(err.contains("external execution preflight failed"));
    assert!(err.contains("agent binary path is not a file"));
    let _ = std::fs::remove_file(kolme_binary);
    let _ = std::fs::remove_dir_all(agent_binary_dir);
}
