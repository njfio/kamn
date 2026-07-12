use super::agent_harness::agent_harness_execution_surface;
use super::report::{report_status, DemoReportInput};
use super::report_artifacts::artifact_path;
use super::three_agent_transcript::three_agent_transcript_path;

pub(crate) fn render_report_markdown(input: &DemoReportInput<'_>) -> Result<String, String> {
    Ok([
        markdown_header(input),
        markdown_artifacts(input),
        markdown_agent_harness(input)?,
        markdown_three_agent_boundary(input),
        markdown_claim_boundaries().to_owned(),
    ]
    .into_iter()
    .filter(|section| !section.is_empty())
    .collect::<Vec<_>>()
    .join("\n"))
}

fn markdown_header(input: &DemoReportInput<'_>) -> String {
    let status = report_status(input);
    format!(
        "# KAMN MVP Demo Proof Report\n\n- Run ID: `{}`\n- Status: `{}`\n- Devnet mode: `{}`\n- Report JSON: `{}`\n",
        input.run_id,
        status,
        input.devnet_mode,
        input.output_root.join("latest/proof/report.json").display(),
    )
}

fn markdown_artifacts(input: &DemoReportInput<'_>) -> String {
    let run_id = input.run_id;
    format!(
        "## Proof Artifacts\n\n- SDK localhost signed artifact: `{}`\n- SDK localhost signed output: `{}`\n- Service API vertical slice output: `{}`\n- Service API websocket output: `{}`\n- Devnet settlement output: `{}`\n- Audit export: `{}`\n",
        artifact_path(input, &format!("{run_id}/proof/localhost-signed-demo.json")),
        artifact_path(input, &format!("{run_id}/proof/localhost-signed-demo-output.txt")),
        artifact_path(input, &format!("{run_id}/proof/service-api-vertical-slice-output.txt")),
        artifact_path(input, &format!("{run_id}/proof/service-api-websocket-output.txt")),
        artifact_path(input, &format!("{run_id}/proof/devnet-settlement-output.txt")),
        artifact_path(input, &format!("{run_id}/proof/audit-export.json"))
    )
}

fn markdown_agent_harness(input: &DemoReportInput<'_>) -> Result<String, String> {
    let Some(path) = input.agent_harness_evidence_path else {
        return Ok(String::new());
    };
    let execution_surface = agent_harness_execution_surface(path)?;
    Ok(format!(
        "## Agent Harness Evidence\n\n- Claim: `mcp_agent_harness_verification`\n- Agent harness evidence: `{}`\n- Execution surface: `{}`\n",
        path, execution_surface
    ))
}

fn markdown_three_agent_boundary(input: &DemoReportInput<'_>) -> String {
    let (Some(settlement), Some(binding)) = (input.devnet_settlement, input.live_task_binding)
    else {
        return String::new();
    };
    let transcript = three_agent_transcript_path(input);
    let explorer = format!(
        "https://explorer.solana.com/tx/{}?cluster=devnet",
        settlement.settlement_tx_signature
    );
    format!(
        "## Three-Agent View Boundary\n\n- Task ID: `{}`\n- Actual service escrow ID: `{}`\n- Settlement execution surface: `{}`\n- Solana devnet transaction: [{explorer}]({explorer})\n- Task binding digest: `{}`\n- Binding artifact: `{}`\n- Transcript artifact: `{transcript}`\n- Agent A and Agent B include participant-private proof digests and private field counts in the JSON report and transcript.\n- The third-party verifier uses a restricted public-view digest to validate shared task, escrow, and settlement commitments.\n- Raw private payloads are redacted, and no verifier private-view digest is emitted.\n",
        binding.task_id,
        settlement.escrow_id,
        settlement.execution_surface,
        binding.digest,
        binding.artifact_path,
    )
}

fn markdown_claim_boundaries() -> &'static str {
    "## Claim Boundaries\n\n- Local runtime, auth, message/task, state, relay, websocket, and audit proof are local-only MVP claims.\n- Settlement or asset movement is not claimed unless the JSON report carries `devnet-backed` evidence.\n- Devnet-required runs without configured settlement evidence are explicit `NO-GO`, not local-only success.\n- Devnet tokens are Solana devnet only and are not real economic value.\n- Production readiness, mainnet, consensus, broad bridge finality, and arbitrary partition tolerance remain roadmap.\n"
}
