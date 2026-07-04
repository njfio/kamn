use std::path::{Path, PathBuf};
use std::process::Command;

const VERTICAL_SLICE_TEST: &str =
    "integration_service_api_endpoint_working_vertical_slice_proves_delivery_dispatch_and_audit_evidence";
const WEBSOCKET_TEST: &str =
    "integration_service_api_endpoint_websocket_upgrade_streams_state_transition_event";
const TARGET_DIR: &str = "target/mvp-demo-proof";

pub(crate) struct ServiceApiProofInput<'a> {
    pub(crate) vertical_slice_command: Option<&'a [String]>,
    pub(crate) websocket_command: Option<&'a [String]>,
    pub(crate) vertical_slice_log: &'a Path,
    pub(crate) websocket_log: &'a Path,
}

pub(crate) fn run_service_api_proofs(input: &ServiceApiProofInput<'_>) -> Result<(), String> {
    run_proof(
        input.vertical_slice_command,
        VERTICAL_SLICE_TEST,
        input.vertical_slice_log,
        "service API vertical slice",
    )?;
    run_proof(
        input.websocket_command,
        WEBSOCKET_TEST,
        input.websocket_log,
        "service API websocket event",
    )
}

fn run_proof(
    override_command: Option<&[String]>,
    test_name: &str,
    output_log: &Path,
    label: &str,
) -> Result<(), String> {
    let output = build_command(override_command, test_name)?
        .current_dir(repo_root())
        .env("CARGO_TARGET_DIR", TARGET_DIR)
        .output()
        .map_err(|error| format!("failed to run {label} proof: {error}"))?;
    write_command_log(output_log, &output)?;
    if !output.status.success() {
        return Err(format!("{label} proof command failed"));
    }
    validate_output(&output, test_name, label)
}

fn build_command(override_command: Option<&[String]>, test_name: &str) -> Result<Command, String> {
    if let Some(parts) = override_command {
        if parts.is_empty() {
            return Err("service API MVP proof command override is empty".to_owned());
        }
        let mut built = Command::new(parts[0].as_str());
        built.args(&parts[1..]);
        return Ok(built);
    }
    let mut built = Command::new("cargo");
    built.args(["test", "-p", "kamn-node", test_name, "--", "--nocapture"]);
    Ok(built)
}

fn write_command_log(path: &Path, output: &std::process::Output) -> Result<(), String> {
    let mut content = String::from("--- stdout ---\n");
    content.push_str(&String::from_utf8_lossy(output.stdout.as_slice()));
    content.push_str("\n--- stderr ---\n");
    content.push_str(&String::from_utf8_lossy(output.stderr.as_slice()));
    std::fs::write(path, content).map_err(|error| {
        format!(
            "failed to write service API MVP proof log {}: {error}",
            path.display()
        )
    })
}

fn validate_output(
    output: &std::process::Output,
    test_name: &str,
    label: &str,
) -> Result<(), String> {
    let combined = format!(
        "{}{}",
        String::from_utf8_lossy(output.stdout.as_slice()),
        String::from_utf8_lossy(output.stderr.as_slice())
    );
    if combined.contains(test_name) {
        return Ok(());
    }
    Err(format!(
        "{label} proof output missing test marker: {test_name}"
    ))
}

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}
