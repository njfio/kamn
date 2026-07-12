use super::report::{escape_json, DemoReportInput};
use super::settlement_evidence_artifact::FILE_NAME as SETTLEMENT_EVIDENCE_FILE;
use super::three_agent_receipts::{
    agent_a_observation_receipt_path, agent_b_observation_receipt_path,
    agent_c_verifier_observation_receipt_path,
};
use super::three_agent_transcript::three_agent_transcript_path;
use super::three_agent_views::{agent_a_view_path, agent_b_view_path, agent_c_verifier_view_path};

pub(super) fn artifacts_json(input: &DemoReportInput<'_>) -> String {
    let entries = artifact_entries(input)
        .into_iter()
        .map(|(key, path)| artifact_json_entry(key, path.as_str()))
        .collect::<Vec<_>>();
    format!("{{{}}}", entries.join(","))
}

fn artifact_entries(input: &DemoReportInput<'_>) -> Vec<(&'static str, String)> {
    let mut entries = base_artifact_entries(input);
    entries.extend(proof_artifact_entries(input));
    if let Some(path) = input.agent_harness_evidence_path {
        entries.push(("agent_harness_evidence", path.to_owned()));
    }
    if let Some(binding) = input.live_task_binding {
        entries.push((
            "live_task_settlement_binding",
            binding.artifact_path.clone(),
        ));
    }
    let funding_request = run_proof_path(input, "devnet-escrow-funding-request.json");
    if std::path::Path::new(funding_request.as_str()).is_file() {
        entries.push(("devnet_escrow_funding_request", funding_request));
    }
    if input.devnet_settlement.is_some() && input.live_task_binding.is_some() {
        entries.extend(runtime_projection_entries(input));
        entries.push(("three_agent_transcript", three_agent_transcript_path(input)));
        entries.push(("agent_a_view", agent_a_view_path(input)));
        entries.push(("agent_b_view", agent_b_view_path(input)));
        entries.push(("agent_c_verifier_view", agent_c_verifier_view_path(input)));
        entries.push((
            "agent_a_observation_receipt",
            agent_a_observation_receipt_path(input),
        ));
        entries.push((
            "agent_b_observation_receipt",
            agent_b_observation_receipt_path(input),
        ));
        entries.push((
            "agent_c_verifier_observation_receipt",
            agent_c_verifier_observation_receipt_path(input),
        ));
    }
    entries
}

fn runtime_projection_entries(input: &DemoReportInput<'_>) -> Vec<(&'static str, String)> {
    [
        (
            "runtime_agent_a_participant_view",
            "runtime-agent-a-participant-view.json",
        ),
        (
            "runtime_agent_b_participant_view",
            "runtime-agent-b-participant-view.json",
        ),
        (
            "runtime_agent_c_verifier_view",
            "runtime-agent-c-verifier-view.json",
        ),
    ]
    .into_iter()
    .map(|(key, file)| (key, run_proof_path(input, file)))
    .filter(|(_, path)| std::path::Path::new(path).is_file())
    .collect()
}

fn base_artifact_entries(input: &DemoReportInput<'_>) -> Vec<(&'static str, String)> {
    vec![
        (
            "report_json",
            artifact_path(input, "latest/proof/report.json"),
        ),
        ("report_md", artifact_path(input, "latest/proof/report.md")),
        (
            "state_dir",
            artifact_path(input, &format!("{}/state", input.run_id)),
        ),
    ]
}

fn proof_artifact_entries(input: &DemoReportInput<'_>) -> Vec<(&'static str, String)> {
    let mut entries = vec![
        ("audit_export", run_proof_path(input, "audit-export.json")),
        (
            "localhost_signed_demo_artifact",
            run_proof_path(input, "localhost-signed-demo.json"),
        ),
        (
            "localhost_signed_demo_output",
            run_proof_path(input, "localhost-signed-demo-output.txt"),
        ),
        (
            "service_api_vertical_slice_output",
            run_proof_path(input, "service-api-vertical-slice-output.txt"),
        ),
        (
            "service_api_websocket_output",
            run_proof_path(input, "service-api-websocket-output.txt"),
        ),
        (
            "devnet_settlement_output",
            run_proof_path(input, "devnet-settlement-output.txt"),
        ),
    ];
    if input.devnet_settlement.is_some() {
        entries.push((
            "devnet_settlement_evidence",
            run_proof_path(input, SETTLEMENT_EVIDENCE_FILE),
        ));
    }
    entries
}

fn artifact_json_entry(key: &str, path: &str) -> String {
    format!("\"{}\":\"{}\"", escape_json(key), escape_json(path))
}

pub(super) fn artifact_path(input: &DemoReportInput<'_>, suffix: &str) -> String {
    input.output_root.join(suffix).display().to_string()
}

fn run_proof_path(input: &DemoReportInput<'_>, file_name: &str) -> String {
    artifact_path(input, &format!("{}/proof/{file_name}", input.run_id))
}
