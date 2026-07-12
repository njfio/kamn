use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

use super::agent_harness::{
    validate_agent_harness_evidence_path, validate_embedded_agent_harness_evidence,
};
use super::artifact_digest::ThreeAgentArtifactDigests;
use super::command_config::{MvpDemoCommandConfig, VerifyMvpDemoCommandConfig};
use super::devnet_settlement::DevnetSettlementAttempt;
use super::live_task_binding::LiveTaskBinding;
use super::live_task_binding_verify::validate_live_task_binding_file;
use super::local_artifact_verify::validate_local_artifact_files;
use super::local_artifacts::create_demo_artifacts;
use super::localhost_signed::{run_localhost_signed_demo, LocalhostSignedDemoInput};
use super::report::{escape_json, render_report_json, DemoReportInput};
use super::report_writer::write_reports;
use super::runner_settlement::create_bound_settlement;
use super::service_api_proof::{run_service_api_proofs, ServiceApiProofInput};
use super::three_agent_transcript::validate_three_agent_transcript_file;
use super::verify::verify_mvp_demo_report_json;

/// Executes the MVP demo command and writes proof artifacts.
pub fn execute_mvp_demo_contract(config: &MvpDemoCommandConfig) -> Result<String, String> {
    validate_devnet_mode(config.devnet_mode.as_str())?;
    let output_root = Path::new(config.output_root.as_str());
    let run_id = build_run_id()?;
    let run_dir = output_root.join(run_id.as_str());
    create_demo_artifacts(&run_dir)?;
    create_localhost_signed_artifact(config, &run_dir)?;
    create_service_api_artifacts(config, &run_dir)?;
    let bound = create_bound_settlement(config, run_id.as_str(), &run_dir)?;
    let input = report_input(
        config,
        output_root,
        run_id.as_str(),
        &bound.settlement,
        bound.binding.as_ref(),
        bound.artifact_digests.as_ref(),
    );
    let report_json = render_report_json(&input)?;
    validate_generated_report(report_json.as_str(), output_root)?;
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
    validate_local_artifact_files(report.as_str())?;
    validate_embedded_agent_harness_evidence(report.as_str(), config.report.as_str())?;
    let pi_actor_summary = validate_direct_evidence(config, report.as_str())?;
    validate_three_agent_transcript_file(report.as_str())?;
    validate_live_task_binding_file(report.as_str())?;
    super::independent_verifier::validate_independent_bundle(
        report.as_str(),
        config.report.as_str(),
    )?;
    Ok(render_verify_output(config, pi_actor_summary))
}

fn validate_direct_evidence(
    config: &VerifyMvpDemoCommandConfig,
    report: &str,
) -> Result<Option<String>, String> {
    if let Some(path) = config.agent_harness_evidence_path.as_deref() {
        validate_agent_harness_evidence_path(report, config.report.as_str(), path)?;
    }
    let Some(paths) = config.pi_transaction_actor_paths.as_ref() else {
        return Ok(None);
    };
    let map = super::independent_verifier_errors::map_actor_verification_error;
    let expected =
        super::runtime_receipt_chain::build_runtime_receipt_chain_from_actor_paths(paths)
            .map_err(map)?;
    super::three_agent_transcript::require_runtime_chain_source(report, expected.as_str())
        .map_err(map)?;
    super::pi_transaction_actor_verify::verify_pi_transaction_actor_paths(paths)
        .map_err(map)
        .map(Some)
}

fn render_verify_output(config: &VerifyMvpDemoCommandConfig, summary: Option<String>) -> String {
    let evidence_path = config.agent_harness_evidence_path.as_deref().unwrap_or("");
    let output = format!(
        "{{\"status\":\"PASS\",\"report\":\"{}\",\"agent_harness_evidence\":\"{}\"}}",
        escape_json(config.report.as_str()),
        escape_json(evidence_path)
    );
    match summary {
        Some(summary) => format!(
            "{},\"pi_transaction_actors\":{summary}}}",
            &output[..output.len() - 1]
        ),
        None => output,
    }
}

fn validate_generated_report(report_json: &str, output_root: &Path) -> Result<(), String> {
    verify_mvp_demo_report_json(report_json)?;
    validate_local_artifact_files(report_json)?;
    validate_embedded_agent_harness_evidence(
        report_json,
        latest_report_path(output_root).as_str(),
    )?;
    validate_three_agent_transcript_file(report_json)?;
    validate_live_task_binding_file(report_json)
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
    settlement: &'a DevnetSettlementAttempt,
    binding: Option<&'a LiveTaskBinding>,
    three_agent_artifact_digests: Option<&'a ThreeAgentArtifactDigests>,
) -> DemoReportInput<'a> {
    DemoReportInput {
        run_id,
        devnet_mode: config.devnet_mode.as_str(),
        solana_rpc_url: config.solana_rpc_url.as_deref(),
        output_root,
        devnet_settlement: settlement.evidence.as_ref(),
        live_task_binding: binding,
        devnet_no_go_reason: settlement.no_go_reason.as_deref(),
        agent_harness_evidence_path: config.agent_harness_evidence_path.as_deref(),
        three_agent_artifact_digests,
    }
}

fn latest_report_path(output_root: &Path) -> String {
    output_root
        .join("latest/proof/report.json")
        .display()
        .to_string()
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
