use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use super::local_artifacts::create_demo_artifacts;
use super::localhost_signed::{run_localhost_signed_demo, LocalhostSignedDemoInput};
use super::report::{escape_json, render_report_json, render_report_markdown, DemoReportInput};
use super::service_api_proof::{run_service_api_proofs, ServiceApiProofInput};
use super::verify::verify_mvp_demo_report_json;

#[derive(Debug, Clone, PartialEq, Eq)]
/// Parsed `demo-mvp` command configuration.
pub struct MvpDemoCommandConfig {
    /// Demo output root directory.
    pub output_root: String,
    /// Devnet execution mode (`optional` or `required`).
    pub devnet_mode: String,
    /// Optional Solana RPC URL for devnet-backed proof attempts.
    pub solana_rpc_url: Option<String>,
    /// Optional command override for tests; production uses the SDK localhost signed demo.
    pub localhost_signed_demo_command: Option<Vec<String>>,
    /// Optional command override for tests; production uses the service API vertical slice test.
    pub service_api_vertical_slice_command: Option<Vec<String>>,
    /// Optional command override for tests; production uses the service API websocket test.
    pub service_api_websocket_command: Option<Vec<String>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// Parsed `verify-mvp-demo` command configuration.
pub struct VerifyMvpDemoCommandConfig {
    /// Proof report JSON path.
    pub report: String,
}

/// Executes the MVP demo command and writes proof artifacts.
pub fn execute_mvp_demo_contract(config: &MvpDemoCommandConfig) -> Result<String, String> {
    validate_devnet_mode(config.devnet_mode.as_str())?;
    let output_root = Path::new(config.output_root.as_str());
    let run_id = build_run_id()?;
    let run_dir = output_root.join(run_id.as_str());
    create_demo_artifacts(&run_dir)?;
    create_localhost_signed_artifact(config, &run_dir)?;
    create_service_api_artifacts(config, &run_dir)?;
    let input = report_input(config, output_root, run_id.as_str());
    let report_json = render_report_json(&input);
    verify_mvp_demo_report_json(report_json.as_str())?;
    write_reports(output_root, run_id.as_str(), report_json.as_str(), &input)?;
    Ok(report_json)
}

/// Executes the MVP demo verifier command.
pub fn execute_verify_mvp_demo_contract(
    config: &VerifyMvpDemoCommandConfig,
) -> Result<String, String> {
    let report = std::fs::read_to_string(config.report.as_str())
        .map_err(|error| format!("failed to read MVP demo report {}: {error}", config.report))?;
    verify_mvp_demo_report_json(report.as_str())?;
    Ok(format!(
        "{{\"status\":\"PASS\",\"report\":\"{}\"}}",
        escape_json(config.report.as_str())
    ))
}

fn validate_devnet_mode(devnet_mode: &str) -> Result<(), String> {
    match devnet_mode {
        "optional" | "required" => Ok(()),
        other => Err(format!("unsupported KAMN_MVP_DEVNET_MODE: {other}")),
    }
}

fn build_run_id() -> Result<String, String> {
    let elapsed = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|error| format!("failed to build MVP demo run id: {error}"))?;
    Ok(format!(
        "run-{}-{}",
        std::process::id(),
        elapsed.as_millis()
    ))
}

fn report_input<'a>(
    config: &'a MvpDemoCommandConfig,
    output_root: &'a Path,
    run_id: &'a str,
) -> DemoReportInput<'a> {
    DemoReportInput {
        run_id,
        devnet_mode: config.devnet_mode.as_str(),
        solana_rpc_url: config.solana_rpc_url.as_deref(),
        output_root,
    }
}

fn create_localhost_signed_artifact(
    config: &MvpDemoCommandConfig,
    run_dir: &Path,
) -> Result<(), String> {
    run_localhost_signed_demo(&LocalhostSignedDemoInput {
        command: config.localhost_signed_demo_command.as_deref(),
        output_json: run_dir.join("proof/localhost-signed-demo.json").as_path(),
        output_log: run_dir
            .join("proof/localhost-signed-demo-output.txt")
            .as_path(),
    })
}

fn create_service_api_artifacts(
    config: &MvpDemoCommandConfig,
    run_dir: &Path,
) -> Result<(), String> {
    run_service_api_proofs(&ServiceApiProofInput {
        vertical_slice_command: config.service_api_vertical_slice_command.as_deref(),
        websocket_command: config.service_api_websocket_command.as_deref(),
        vertical_slice_log: run_dir
            .join("proof/service-api-vertical-slice-output.txt")
            .as_path(),
        websocket_log: run_dir
            .join("proof/service-api-websocket-output.txt")
            .as_path(),
    })
}

fn write_reports(
    output_root: &Path,
    run_id: &str,
    report_json: &str,
    input: &DemoReportInput<'_>,
) -> Result<(), String> {
    let report_md = render_report_markdown(input);
    write_report_pair(output_root.join(run_id), report_json, report_md.as_str())?;
    refresh_latest(output_root, run_id, report_json, report_md.as_str())
}

fn write_report_pair(root: PathBuf, report_json: &str, report_md: &str) -> Result<(), String> {
    let proof_dir = root.join("proof");
    create_dir(&proof_dir)?;
    write_file(proof_dir.join("report.json"), report_json)?;
    write_file(proof_dir.join("report.md"), report_md)
}

fn refresh_latest(
    output_root: &Path,
    run_id: &str,
    report_json: &str,
    report_md: &str,
) -> Result<(), String> {
    let latest = output_root.join("latest");
    remove_latest(&latest)?;
    write_report_pair(latest.clone(), report_json, report_md)?;
    write_file(latest.join("RUN_ID"), run_id)
}

fn remove_latest(latest: &Path) -> Result<(), String> {
    if latest.exists() {
        std::fs::remove_dir_all(latest)
            .map_err(|error| format!("failed to remove previous latest demo: {error}"))?;
    }
    Ok(())
}

fn create_dir(path: &Path) -> Result<(), String> {
    std::fs::create_dir_all(path).map_err(|error| {
        format!(
            "failed to create MVP demo directory {}: {error}",
            path.display()
        )
    })
}

fn write_file(path: PathBuf, content: &str) -> Result<(), String> {
    std::fs::write(&path, content).map_err(|error| {
        format!(
            "failed to write MVP demo artifact {}: {error}",
            path.display()
        )
    })
}
