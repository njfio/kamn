use super::report::{escape_json, DemoReportInput};

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
    entries
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
    vec![
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
    ]
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
