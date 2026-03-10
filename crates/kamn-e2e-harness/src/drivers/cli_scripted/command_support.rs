use super::{
    AGENT_LIB_DETERMINISTIC_IDENTITY_OPT_IN_ENV, AGENT_LIB_DETERMINISTIC_IDENTITY_OPT_IN_VALUE,
};
use std::process::{Command, Stdio};

pub(crate) fn run_cli_command_capture_stdout(
    cli_binary: &str,
    args: &[&str],
    step: &str,
) -> Result<String, String> {
    run_cli_command_capture_stdout_with_optional_agent_name(cli_binary, args, step, None)
}

pub(crate) fn run_cli_command_expect_failure_with_agent_name(
    cli_binary: &str,
    args: &[&str],
    step: &str,
    agent_name: &str,
) -> Result<String, String> {
    let mut command = configured_command(cli_binary, args, Some(agent_name));
    command.stdout(Stdio::piped()).stderr(Stdio::piped());
    let output = command
        .output()
        .map_err(|error| format!("{step} failed to spawn: {error}"))?;
    if output.status.success() {
        return Err(format!("{step} unexpectedly succeeded"));
    }
    let stderr = String::from_utf8_lossy(output.stderr.as_slice())
        .trim()
        .to_owned();
    if stderr.is_empty() {
        return Err(format!("{step} failed without stderr details"));
    }
    Ok(stderr)
}

pub(crate) fn run_cli_command_capture_stdout_with_agent_name(
    cli_binary: &str,
    args: &[&str],
    step: &str,
    agent_name: &str,
) -> Result<String, String> {
    run_cli_command_capture_stdout_with_optional_agent_name(
        cli_binary,
        args,
        step,
        Some(agent_name),
    )
}

pub(crate) fn run_cli_command_capture_stdout_with_optional_agent_name(
    cli_binary: &str,
    args: &[&str],
    step: &str,
    agent_name: Option<&str>,
) -> Result<String, String> {
    let mut command = configured_command(cli_binary, args, agent_name);
    command.stdout(Stdio::piped()).stderr(Stdio::null());
    let output = command
        .output()
        .map_err(|error| format!("{step} failed to spawn: {error}"))?;
    if !output.status.success() {
        let exit_status = output
            .status
            .code()
            .map(|value| value.to_string())
            .unwrap_or_else(|| "signal".to_owned());
        return Err(format!("{step} failed (exit_status={exit_status})"));
    }
    let stdout = String::from_utf8_lossy(output.stdout.as_slice())
        .trim()
        .to_owned();
    if stdout.is_empty() {
        return Err(format!("{step} returned empty stdout"));
    }
    Ok(stdout)
}

pub(crate) fn parse_text_output_field<'a>(output: &'a str, key: &str) -> Option<&'a str> {
    output.split_whitespace().find_map(|token| {
        let (field, value) = token.split_once('=')?;
        (field == key).then_some(value)
    })
}

fn configured_command(cli_binary: &str, args: &[&str], agent_name: Option<&str>) -> Command {
    let mut command = Command::new(cli_binary);
    command.args(args).stdin(Stdio::null()).env(
        AGENT_LIB_DETERMINISTIC_IDENTITY_OPT_IN_ENV,
        AGENT_LIB_DETERMINISTIC_IDENTITY_OPT_IN_VALUE,
    );
    if let Some(agent_name) = agent_name {
        command.env("KAMN_AGENT_NAME", agent_name);
    }
    command
}
