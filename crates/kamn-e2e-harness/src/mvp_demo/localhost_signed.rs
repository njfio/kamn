use std::path::{Path, PathBuf};
use std::process::Command;

const ARTIFACT_SCHEMA: &str = "kamn.sdk.localhost-signed.demo-receipt-artifact.v1";
const SUCCESS_MARKER: &str = "localhost signed message demo completed.";
const TIMEOUT_SECONDS: &str = "240";
const TARGET_DIR: &str = "target/mvp-demo-proof";

pub(crate) struct LocalhostSignedDemoInput<'a> {
    pub(crate) command: Option<&'a [String]>,
    pub(crate) output_json: &'a Path,
    pub(crate) output_log: &'a Path,
}

pub(crate) fn run_localhost_signed_demo(
    input: &LocalhostSignedDemoInput<'_>,
) -> Result<(), String> {
    let mut command = build_command(input.command)?;
    command.arg("--output-json").arg(input.output_json);
    command.arg("--timeout-seconds").arg(TIMEOUT_SECONDS);
    command.env("CARGO_TARGET_DIR", TARGET_DIR);
    let output = command
        .current_dir(repo_root())
        .output()
        .map_err(|error| format!("failed to run localhost signed MVP proof: {error}"))?;
    write_command_log(input.output_log, &output)?;
    if !output.status.success() {
        return Err("localhost signed MVP proof command failed".to_owned());
    }
    validate_command_output(&output)?;
    validate_artifact(input.output_json)
}

fn build_command(command: Option<&[String]>) -> Result<Command, String> {
    if let Some(parts) = command {
        if parts.is_empty() {
            return Err("localhost signed MVP proof command override is empty".to_owned());
        }
        let mut built = Command::new(parts[0].as_str());
        built.args(&parts[1..]);
        return Ok(built);
    }
    let mut built = Command::new("bash");
    built.arg("scripts/sdk/run_localhost_signed_demo.sh");
    Ok(built)
}

fn write_command_log(path: &Path, output: &std::process::Output) -> Result<(), String> {
    let mut content = String::from("--- stdout ---\n");
    content.push_str(&String::from_utf8_lossy(output.stdout.as_slice()));
    content.push_str("\n--- stderr ---\n");
    content.push_str(&String::from_utf8_lossy(output.stderr.as_slice()));
    std::fs::write(path, content).map_err(|error| {
        format!(
            "failed to write localhost signed MVP proof log {}: {error}",
            path.display()
        )
    })
}

fn validate_command_output(output: &std::process::Output) -> Result<(), String> {
    let stdout = String::from_utf8_lossy(output.stdout.as_slice());
    if stdout.contains(SUCCESS_MARKER) {
        return Ok(());
    }
    Err(format!(
        "localhost signed MVP proof missing success marker: {SUCCESS_MARKER}"
    ))
}

fn validate_artifact(path: &Path) -> Result<(), String> {
    let artifact = std::fs::read_to_string(path).map_err(|error| {
        format!(
            "failed to read localhost signed MVP proof artifact {}: {error}",
            path.display()
        )
    })?;
    require_artifact_marker(artifact.as_str(), ARTIFACT_SCHEMA, "schema_version")?;
    require_artifact_marker(artifact.as_str(), "\"status\": \"pass\"", "status")
}

fn require_artifact_marker(artifact: &str, marker: &str, context: &str) -> Result<(), String> {
    if artifact.contains(marker) {
        return Ok(());
    }
    Err(format!(
        "localhost signed MVP proof artifact missing {context}: {marker}"
    ))
}

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}
