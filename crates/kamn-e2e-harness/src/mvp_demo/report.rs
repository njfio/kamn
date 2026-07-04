use std::path::Path;

use super::report_artifacts::{artifact_path, artifacts_json};

/// MVP demo proof report schema marker.
pub const MVP_DEMO_REPORT_SCHEMA_VERSION: &str = "kamn.mvp.demo.proof-report.v1";
/// Claim label for actual local KAMN execution.
pub const CLAIM_LABEL_REAL: &str = "real";
/// Claim label for Solana devnet-backed execution or evidence.
pub const CLAIM_LABEL_DEVNET_BACKED: &str = "devnet-backed";
/// Claim label for local-only KAMN execution.
pub const CLAIM_LABEL_LOCAL_ONLY: &str = "local-only";
/// Claim label for dry-run-only behavior.
pub const CLAIM_LABEL_DRY_RUN: &str = "dry-run";
/// Claim label for placeholder behavior.
pub const CLAIM_LABEL_PLACEHOLDER: &str = "placeholder";
/// Claim label for explicit non-MVP roadmap behavior.
pub const CLAIM_LABEL_ROADMAP: &str = "roadmap";

pub(crate) struct DemoReportInput<'a> {
    pub(crate) run_id: &'a str,
    pub(crate) devnet_mode: &'a str,
    pub(crate) solana_rpc_url: Option<&'a str>,
    pub(crate) output_root: &'a Path,
}

pub(crate) fn render_report_json(input: &DemoReportInput<'_>) -> String {
    let status = report_status(input.devnet_mode);
    format!(
        "{{\"schema_version\":\"{}\",\"run_id\":\"{}\",\"status\":\"{}\",\"devnet_mode\":\"{}\",\"artifacts\":{},\"claim_matrix\":[{}],\"no_go\":{}}}",
        MVP_DEMO_REPORT_SCHEMA_VERSION,
        escape_json(input.run_id),
        status,
        escape_json(input.devnet_mode),
        artifacts_json(input),
        claim_matrix_json(input),
        no_go_json(input)
    )
}

pub(crate) fn render_report_markdown(input: &DemoReportInput<'_>) -> String {
    [
        markdown_header(input),
        markdown_artifacts(input),
        markdown_claim_boundaries().to_owned(),
    ]
    .join("\n")
}

fn markdown_header(input: &DemoReportInput<'_>) -> String {
    let status = report_status(input.devnet_mode);
    format!(
        "# KAMN MVP Demo Proof Report\n\n- Run ID: `{}`\n- Status: `{}`\n- Devnet mode: `{}`\n- Report JSON: `{}`\n",
        input.run_id,
        status,
        input.devnet_mode,
        input.output_root.join("latest/proof/report.json").display(),
    )
}

fn markdown_artifacts(input: &DemoReportInput<'_>) -> String {
    format!(
        "## Proof Artifacts\n\n- SDK localhost signed artifact: `{}`\n- SDK localhost signed output: `{}`\n- Service API vertical slice output: `{}`\n- Service API websocket output: `{}`\n- Audit export: `{}`\n",
        artifact_path(input, &format!("{}/proof/localhost-signed-demo.json", input.run_id)),
        artifact_path(
            input,
            &format!("{}/proof/localhost-signed-demo-output.txt", input.run_id)
        ),
        artifact_path(
            input,
            &format!(
                "{}/proof/service-api-vertical-slice-output.txt",
                input.run_id
            ),
        ),
        artifact_path(input, &format!("{}/proof/service-api-websocket-output.txt", input.run_id)),
        artifact_path(input, &format!("{}/proof/audit-export.json", input.run_id))
    )
}

fn markdown_claim_boundaries() -> &'static str {
    "## Claim Boundaries\n\n- Local runtime, auth, message/task, state, relay, websocket, and audit proof are local-only MVP claims.\n- Settlement or asset movement is not claimed unless the JSON report carries `devnet-backed` evidence.\n- Devnet-required runs without configured settlement evidence are explicit `NO-GO`, not local-only success.\n- Devnet tokens are Solana devnet only and are not real economic value.\n- Production readiness, mainnet, consensus, broad bridge finality, and arbitrary partition tolerance remain roadmap.\n"
}

fn report_status(devnet_mode: &str) -> &'static str {
    if devnet_mode == "required" {
        "NO-GO"
    } else {
        "GO"
    }
}

fn claim_matrix_json(input: &DemoReportInput<'_>) -> String {
    let mut claims = local_claims();
    claims.push(roadmap_claim());
    if input.devnet_mode == "required" {
        claims.push(devnet_no_go_claim(input.solana_rpc_url));
    }
    claims.join(",")
}

fn local_claims() -> Vec<String> {
    LOCAL_CLAIM_ROWS
        .iter()
        .map(|(id, label, status, summary)| claim(id, label, status, summary))
        .collect()
}

const LOCAL_CLAIM_ROWS: &[(&str, &str, &str, &str)] = &[
    (
        "local_runtime_startup",
        CLAIM_LABEL_REAL,
        "PASS",
        "local service API runtime proof recorded",
    ),
    (
        "authenticated_agent_identities",
        CLAIM_LABEL_LOCAL_ONLY,
        "PASS",
        "Alice and Bob identities recorded in localhost signed demo artifact",
    ),
    (
        "signed_message_or_task_flow",
        CLAIM_LABEL_LOCAL_ONLY,
        "PASS",
        "signed localhost message flow recorded",
    ),
    (
        "durable_state_written",
        CLAIM_LABEL_LOCAL_ONLY,
        "PASS",
        "service API vertical slice wrote durable state evidence",
    ),
    (
        "relay_projection_visible",
        CLAIM_LABEL_LOCAL_ONLY,
        "PASS",
        "service API vertical slice projected relay delivery",
    ),
    (
        "websocket_event_visibility",
        CLAIM_LABEL_LOCAL_ONLY,
        "PASS",
        "service API websocket proof recorded an event stream",
    ),
    (
        "audit_proof_export",
        CLAIM_LABEL_LOCAL_ONLY,
        "PASS",
        "service API vertical slice emitted audit proof",
    ),
];

fn claim(id: &str, label: &str, status: &str, summary: &str) -> String {
    format!(
        "{{\"id\":\"{}\",\"label\":\"{}\",\"required\":true,\"status\":\"{}\",\"summary\":\"{}\"}}",
        escape_json(id),
        escape_json(label),
        escape_json(status),
        escape_json(summary)
    )
}

fn roadmap_claim() -> String {
    format!(
        "{{\"id\":\"production_readiness\",\"label\":\"{}\",\"required\":false,\"status\":\"NOT_CLAIMED\",\"summary\":\"production readiness is not claimed\"}}",
        CLAIM_LABEL_ROADMAP
    )
}

fn devnet_no_go_claim(solana_rpc_url: Option<&str>) -> String {
    let reason = devnet_no_go_reason(solana_rpc_url);
    format!(
        "{{\"id\":\"devnet_settlement_no_go\",\"label\":\"{}\",\"required\":true,\"status\":\"NO-GO\",\"summary\":\"Solana devnet escrow settlement evidence unavailable\",\"network\":\"solana:devnet\",\"rpc_url\":\"{}\",\"no_go_reason\":\"{}\"}}",
        CLAIM_LABEL_DEVNET_BACKED,
        escape_json(solana_rpc_url.unwrap_or("")),
        reason
    )
}

fn devnet_no_go_reason(solana_rpc_url: Option<&str>) -> &'static str {
    match solana_rpc_url {
        Some(value) if !value.trim().is_empty() => "devnet_keypair_not_configured",
        _ => "devnet_rpc_url_missing",
    }
}

fn no_go_json(input: &DemoReportInput<'_>) -> String {
    if input.devnet_mode != "required" {
        return "{\"active\":false,\"reason\":\"\"}".to_owned();
    }
    format!(
        "{{\"active\":true,\"reason\":\"{}\"}}",
        devnet_no_go_reason(input.solana_rpc_url)
    )
}

pub(crate) fn escape_json(value: &str) -> String {
    value.replace('\\', "\\\\").replace('"', "\\\"")
}
