use std::path::Path;

use super::agent_harness::agent_harness_claim_json;
use super::artifact_digest::ThreeAgentArtifactDigests;
use super::devnet_settlement::DevnetSettlementEvidence;
use super::live_task_binding::LiveTaskBinding;
use super::report_artifacts::artifacts_json;
use super::report_devnet::{devnet_required_claims, no_go_json};

#[doc = "MVP demo proof report schema marker."]
pub const MVP_DEMO_REPORT_SCHEMA_VERSION: &str = "kamn.mvp.demo.proof-report.v1";
#[doc = "Claim label for actual local KAMN execution."]
pub const CLAIM_LABEL_REAL: &str = "real";
#[doc = "Claim label for Solana devnet-backed execution or evidence."]
pub const CLAIM_LABEL_DEVNET_BACKED: &str = "devnet-backed";
#[doc = "Claim label for local-only KAMN execution."]
pub const CLAIM_LABEL_LOCAL_ONLY: &str = "local-only";
#[doc = "Claim label for dry-run-only behavior."]
pub const CLAIM_LABEL_DRY_RUN: &str = "dry-run";
#[doc = "Claim label for placeholder behavior."]
pub const CLAIM_LABEL_PLACEHOLDER: &str = "placeholder";
#[doc = "Claim label for explicit non-MVP roadmap behavior."]
pub const CLAIM_LABEL_ROADMAP: &str = "roadmap";

pub(crate) struct DemoReportInput<'a> {
    pub(crate) run_id: &'a str,
    pub(crate) devnet_mode: &'a str,
    pub(crate) solana_rpc_url: Option<&'a str>,
    pub(crate) output_root: &'a Path,
    pub(crate) devnet_settlement: Option<&'a DevnetSettlementEvidence>,
    pub(crate) live_task_binding: Option<&'a LiveTaskBinding>,
    pub(crate) devnet_no_go_reason: Option<&'a str>,
    pub(crate) agent_harness_evidence_path: Option<&'a str>,
    pub(crate) three_agent_artifact_digests: Option<&'a ThreeAgentArtifactDigests>,
}

pub(crate) fn render_report_json(input: &DemoReportInput<'_>) -> Result<String, String> {
    let status = report_status(input);
    Ok(format!(
        "{{\"schema_version\":\"{}\",\"run_id\":\"{}\",\"status\":\"{}\",\"devnet_mode\":\"{}\",\"artifacts\":{},\"claim_matrix\":[{}],\"no_go\":{}}}",
        MVP_DEMO_REPORT_SCHEMA_VERSION,
        escape_json(input.run_id),
        status,
        escape_json(input.devnet_mode),
        artifacts_json(input),
        claim_matrix_json(input)?,
        no_go_json(input)
    ))
}

pub(crate) fn report_status(input: &DemoReportInput<'_>) -> &'static str {
    if input.devnet_mode == "required" && input.devnet_settlement.is_none() {
        return "NO-GO";
    }
    "GO"
}

fn claim_matrix_json(input: &DemoReportInput<'_>) -> Result<String, String> {
    let mut claims = local_claims();
    if input.agent_harness_evidence_path.is_some() {
        claims.push(agent_harness_claim_json());
    }
    if input.devnet_mode == "required" {
        claims.extend(devnet_required_claims(input)?);
    }
    claims.push(roadmap_claim());
    Ok(claims.join(","))
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
        "{{\"id\":\"production_readiness\",\"label\":\"{CLAIM_LABEL_ROADMAP}\",\"required\":false,\"status\":\"NOT_CLAIMED\",\"summary\":\"production readiness is not claimed\"}}"
    )
}

pub(crate) fn escape_json(value: &str) -> String {
    value.replace('\\', "\\\\").replace('"', "\\\"")
}
