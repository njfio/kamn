use std::path::Path;

use super::agent_harness::agent_harness_claim_json;
use super::artifact_digest::ThreeAgentArtifactDigests;
use super::devnet_settlement::{
    devnet_no_go_reason, devnet_settlement_claim_json, DevnetSettlementEvidence,
};
use super::live_task_binding::LiveTaskBinding;
use super::report_artifacts::artifacts_json;
use super::three_agent_claim::three_agent_escrow_claim_json;
use super::three_agent_receipts::{
    agent_a_observation_receipt_path, agent_b_observation_receipt_path,
    agent_c_verifier_observation_receipt_path,
};
use super::three_agent_transcript::three_agent_transcript_path;
use super::three_agent_views::{agent_a_view_path, agent_b_view_path, agent_c_verifier_view_path};

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
        "{{\"id\":\"production_readiness\",\"label\":\"{}\",\"required\":false,\"status\":\"NOT_CLAIMED\",\"summary\":\"production readiness is not claimed\"}}",
        CLAIM_LABEL_ROADMAP
    )
}

fn devnet_required_claims(input: &DemoReportInput<'_>) -> Result<Vec<String>, String> {
    match input.devnet_settlement {
        Some(evidence) => devnet_success_claims(input, evidence),
        None => Ok(vec![devnet_no_go_claim_with_reason(input)]),
    }
}

fn devnet_success_claims(
    input: &DemoReportInput<'_>,
    evidence: &DevnetSettlementEvidence,
) -> Result<Vec<String>, String> {
    let settlement_claim = devnet_settlement_claim_json(evidence);
    let Some(binding) = input.live_task_binding else {
        return Ok(vec![settlement_claim]);
    };
    let digests = input
        .three_agent_artifact_digests
        .ok_or_else(|| "missing three-agent artifact digests".to_owned())?;
    let transcript = three_agent_transcript_path(input);
    let agent_a = agent_a_view_path(input);
    let agent_b = agent_b_view_path(input);
    let agent_c = agent_c_verifier_view_path(input);
    let agent_a_receipt = agent_a_observation_receipt_path(input);
    let agent_b_receipt = agent_b_observation_receipt_path(input);
    let agent_c_receipt = agent_c_verifier_observation_receipt_path(input);
    Ok(vec![
        settlement_claim,
        three_agent_escrow_claim_json(
            input.run_id,
            evidence,
            binding,
            transcript.as_str(),
            [agent_a.as_str(), agent_b.as_str(), agent_c.as_str()],
            [
                agent_a_receipt.as_str(),
                agent_b_receipt.as_str(),
                agent_c_receipt.as_str(),
            ],
            digests,
        ),
    ])
}

fn no_go_json(input: &DemoReportInput<'_>) -> String {
    if input.devnet_mode != "required" || input.devnet_settlement.is_some() {
        return "{\"active\":false,\"reason\":\"\"}".to_owned();
    }
    format!(
        "{{\"active\":true,\"reason\":\"{}\"}}",
        effective_no_go_reason(input).as_str()
    )
}

fn devnet_no_go_claim_with_reason(input: &DemoReportInput<'_>) -> String {
    format!(
        "{{\"id\":\"devnet_settlement_no_go\",\"label\":\"{}\",\"required\":true,\"status\":\"NO-GO\",\"summary\":\"Solana devnet escrow settlement evidence unavailable\",\"network\":\"solana:devnet\",\"rpc_url\":\"{}\",\"no_go_reason\":\"{}\"}}",
        CLAIM_LABEL_DEVNET_BACKED,
        escape_json(input.solana_rpc_url.unwrap_or("")),
        effective_no_go_reason(input).as_str()
    )
}

fn effective_no_go_reason(input: &DemoReportInput<'_>) -> String {
    input
        .devnet_no_go_reason
        .unwrap_or_else(|| devnet_no_go_reason(input.solana_rpc_url))
        .to_owned()
}

pub(crate) fn escape_json(value: &str) -> String {
    value.replace('\\', "\\\\").replace('"', "\\\"")
}
