use super::report::{report_status, DemoReportInput};
use super::report_artifacts::artifact_path;

pub(crate) fn render_report_markdown(input: &DemoReportInput<'_>) -> String {
    [
        markdown_header(input),
        markdown_artifacts(input),
        markdown_claim_boundaries().to_owned(),
    ]
    .join("\n")
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

fn markdown_claim_boundaries() -> &'static str {
    "## Claim Boundaries\n\n- Local runtime, auth, message/task, state, relay, websocket, and audit proof are local-only MVP claims.\n- Settlement or asset movement is not claimed unless the JSON report carries `devnet-backed` evidence.\n- Devnet-required runs without configured settlement evidence are explicit `NO-GO`, not local-only success.\n- Devnet tokens are Solana devnet only and are not real economic value.\n- Production readiness, mainnet, consensus, broad bridge finality, and arbitrary partition tolerance remain roadmap.\n"
}
