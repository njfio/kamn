use std::fs;
use std::path::PathBuf;

const CLI_SCRIPTED_ROOT_SOURCE: &str = include_str!("../src/drivers/cli_scripted.rs");
const CLI_SCRIPTED_DRIVER_CORE_FILE: &str = "src/drivers/cli_scripted/driver_core.rs";
const CLI_SCRIPTED_COMMAND_SUPPORT_FILE: &str =
    "src/drivers/cli_scripted/command_support.rs";
const ROOT_MAX_LINES: usize = 200;
const EXTRACTED_MAX_LINES: usize = 200;

#[test]
fn regression_cli_scripted_root_declares_driver_core_and_command_support_modules() {
    for marker in ["mod driver_core;", "mod command_support;"] {
        assert!(
            CLI_SCRIPTED_ROOT_SOURCE.contains(marker),
            "cli_scripted.rs must declare extracted root module marker: {marker}"
        );
    }
}

#[test]
fn regression_cli_scripted_root_removes_residual_driver_and_command_helper_definitions() {
    for marker in [
        "pub struct CliScriptedDriver {",
        "fn run_cli_command_capture_stdout(",
        "fn run_cli_command_expect_failure_with_agent_name(",
        "fn run_cli_command_capture_stdout_with_agent_name(",
        "fn run_cli_command_capture_stdout_with_optional_agent_name(",
        "fn parse_text_output_field<'a>(",
    ] {
        assert!(
            !CLI_SCRIPTED_ROOT_SOURCE.contains(marker),
            "cli_scripted.rs must not keep residual root helper marker: {marker}"
        );
    }
}

#[test]
fn regression_cli_scripted_root_extracted_module_files_exist() {
    for relative_path in [CLI_SCRIPTED_DRIVER_CORE_FILE, CLI_SCRIPTED_COMMAND_SUPPORT_FILE] {
        let full_path = manifest_dir().join(relative_path);
        assert!(
            full_path.exists(),
            "expected extracted cli_scripted root module missing: {}",
            full_path.display()
        );
    }
}

#[test]
fn regression_cli_scripted_root_respects_full_file_budget() {
    let line_count = CLI_SCRIPTED_ROOT_SOURCE.lines().count();
    assert!(
        line_count <= ROOT_MAX_LINES,
        "cli_scripted.rs should stay within the root file budget: {line_count} > {ROOT_MAX_LINES}"
    );
}

#[test]
fn regression_cli_scripted_root_extracted_files_stay_within_line_budget() {
    let offenders = [CLI_SCRIPTED_DRIVER_CORE_FILE, CLI_SCRIPTED_COMMAND_SUPPORT_FILE]
        .into_iter()
        .filter_map(|relative_path| {
            let full_path = manifest_dir().join(relative_path);
            let line_count = fs::read_to_string(&full_path).ok()?.lines().count();
            (line_count > EXTRACTED_MAX_LINES)
                .then(|| format!("{} ({line_count})", full_path.display()))
        })
        .collect::<Vec<String>>();

    assert!(
        offenders.is_empty(),
        "extracted cli_scripted root files exceed {EXTRACTED_MAX_LINES} LOC: {}",
        offenders.join(", ")
    );
}

fn manifest_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}
