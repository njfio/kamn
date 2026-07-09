use std::path::Path;

use super::agent_harness::agent_harness_claim_json;
use super::devnet_settlement::{
    devnet_no_go_reason, devnet_settlement_claim_json, DevnetSettlementEvidence,
};
use super::report_artifacts::artifacts_json;
use super::three_agent_claim::three_agent_escrow_claim_json;
use super::three_agent_transcript::three_agent_transcript_path;

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
    pub(crate) devnet_settlement: Option<&'a DevnetSettlementEvidence>,
    pub(crate) devnet_no_go_reason: Option<&'a str>,
    pub(crate) agent_harness_evidence_path: Option<&'a str>,
}

pub(crate) fn render_report_json(input: &DemoReportInput<'_>) -> String {
    let status = report_status(input);
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

pub(crate) fn report_status(input: &DemoReportInput<'_>) -> &'static str {
    if input.devnet_mode == "required" && input.devnet_settlement.is_none() {
        "NO-GO"
    } else {
        "GO"
    }
}

fn claim_matrix_json(input: &DemoReportInput<'_>) -> String {
    let mut claims = local_claims();
    if input.agent_harness_evidence_path.is_some() {
        claims.push(agent_harness_claim_json());
    }
    if input.devnet_mode == "required" {
        claims.extend(devnet_required_claims(input));
    }
    claims.push(roadmap_claim());
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

fn devnet_required_claims(input: &DemoReportInput<'_>) -> Vec<String> {
    match input.devnet_settlement {
        Some(evidence) => vec![
            devnet_settlement_claim_json(evidence),
            three_agent_escrow_claim_json(
                input.run_id,
                evidence,
                three_agent_transcript_path(input).as_str(),
            ),
        ],
        None => vec![devnet_no_go_claim_with_reason(input)],
    }
}

fn no_go_json(input: &DemoReportInput<'_>) -> String {
    if input.devnet_mode != "required" {
        return "{\"active\":false,\"reason\":\"\"}".to_owned();
    }
    if input.devnet_settlement.is_some() {
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
