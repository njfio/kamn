use super::*;
use std::path::PathBuf;

pub fn run_config(
    mode: &str,
    agent_binary: Option<&str>,
    external_execution: bool,
    evidence_dir: &str,
    scenario_ids: &[&str],
) -> RunCommandConfig {
    RunCommandConfig {
        mode: mode.to_owned(),
        kolme_binary: "/tmp/kolme-node".to_owned(),
        agent_binary: agent_binary.map(str::to_owned),
        external_execution,
        evidence_dir: evidence_dir.to_owned(),
        scenario_ids: scenario_ids.iter().map(|id| (*id).to_owned()).collect(),
    }
}

pub fn render_run(config: &RunCommandConfig) -> String {
    execute_run_contract(config).expect("run output should render")
}

pub fn assert_contains_all(output: &str, markers: &[&str]) {
    for marker in markers {
        assert!(output.contains(marker), "missing marker: {marker}");
    }
}

pub fn with_executable_binary<R>(name: &str, f: impl FnOnce(&PathBuf) -> R) -> R {
    let path = temp_path(name);
    write_stub_binary(&path);
    #[cfg(unix)]
    set_executable(&path);
    let result = f(&path);
    let _ = std::fs::remove_file(&path);
    result
}

pub fn with_two_executable_binaries<R>(
    first: &str,
    second: &str,
    f: impl FnOnce(&PathBuf, &PathBuf) -> R,
) -> R {
    with_executable_binary(first, |first_path| {
        with_executable_binary(second, |second_path| f(first_path, second_path))
    })
}
