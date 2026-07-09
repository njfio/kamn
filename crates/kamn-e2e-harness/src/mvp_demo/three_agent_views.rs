use std::path::Path;

use super::artifact_digest::{attach_json_digest, ArtifactJson, ThreeAgentViewDigests};
use super::devnet_settlement::DevnetSettlementEvidence;
use super::report::{escape_json, DemoReportInput};

const AGENT_A_VIEW_FILE: &str = "agent-a-view.json";
const AGENT_B_VIEW_FILE: &str = "agent-b-view.json";
const AGENT_C_VIEW_FILE: &str = "agent-c-verifier-view.json";

pub(crate) fn agent_a_view_path(input: &DemoReportInput<'_>) -> String {
    view_path(input, AGENT_A_VIEW_FILE)
}

pub(crate) fn agent_b_view_path(input: &DemoReportInput<'_>) -> String {
    view_path(input, AGENT_B_VIEW_FILE)
}

pub(crate) fn agent_c_verifier_view_path(input: &DemoReportInput<'_>) -> String {
    view_path(input, AGENT_C_VIEW_FILE)
}

pub(crate) fn public_view_digest(run_id: &str) -> String {
    format!("public-view-digest-{run_id}")
}

pub(crate) fn agent_a_private_view_digest(run_id: &str) -> String {
    format!("agent-a-private-digest-{run_id}")
}

pub(crate) fn agent_b_private_view_digest(run_id: &str) -> String {
    format!("agent-b-private-digest-{run_id}")
}

pub(crate) fn write_three_agent_views(
    run_id: &str,
    evidence: &DevnetSettlementEvidence,
    run_dir: &Path,
) -> Result<ThreeAgentViewDigests, String> {
    let agent_a = participant_view_json(
        run_id,
        evidence,
        "agent_a",
        agent_a_private_view_digest(run_id),
    )?;
    let agent_b = participant_view_json(
        run_id,
        evidence,
        "agent_b",
        agent_b_private_view_digest(run_id),
    )?;
    let agent_c = verifier_view_json(run_id, evidence)?;
    write_view(run_dir, AGENT_A_VIEW_FILE, agent_a.json.as_str())?;
    write_view(run_dir, AGENT_B_VIEW_FILE, agent_b.json.as_str())?;
    write_view(run_dir, AGENT_C_VIEW_FILE, agent_c.json.as_str())?;
    Ok(ThreeAgentViewDigests {
        agent_a: agent_a.digest,
        agent_b: agent_b.digest,
        agent_c_verifier: agent_c.digest,
    })
}

fn view_path(input: &DemoReportInput<'_>, file_name: &str) -> String {
    input
        .output_root
        .join(format!("{}/proof/{file_name}", input.run_id))
        .display()
        .to_string()
}

fn write_view(run_dir: &Path, file_name: &str, json: &str) -> Result<(), String> {
    let path = run_dir.join("proof").join(file_name);
    std::fs::write(path.as_path(), json).map_err(|error| {
        format!(
            "failed to write three-agent view artifact {}: {error}",
            path.display()
        )
    })
}

fn participant_view_json(
    run_id: &str,
    evidence: &DevnetSettlementEvidence,
    agent: &str,
    private_digest: String,
) -> Result<ArtifactJson, String> {
    attach_json_digest(
        format!(
            "{{\"schema_version\":\"kamn.mvp.three-agent-view.v1\",\"agent\":\"{}\",\"view_scope\":\"participant-private\",{},\"private_field_count\":3,\"participant_private_view_digest\":\"{}\",\"public_view_digest\":\"{}\",\"private_payload_redacted\":true,\"view_digest\":\"\"}}",
            escape_json(agent),
            shared_view_fields(run_id, evidence),
            escape_json(private_digest.as_str()),
            escape_json(public_view_digest(run_id).as_str()),
        ),
        "view_digest",
    )
}

fn verifier_view_json(
    run_id: &str,
    evidence: &DevnetSettlementEvidence,
) -> Result<ArtifactJson, String> {
    attach_json_digest(
        format!(
            "{{\"schema_version\":\"kamn.mvp.three-agent-view.v1\",\"agent\":\"agent_c_verifier\",\"view_scope\":\"restricted-public\",{},\"private_field_count\":0,\"public_view_digest\":\"{}\",\"private_payload_redacted\":true,\"view_digest\":\"\"}}",
            shared_view_fields(run_id, evidence),
            escape_json(public_view_digest(run_id).as_str()),
        ),
        "view_digest",
    )
}

fn shared_view_fields(run_id: &str, evidence: &DevnetSettlementEvidence) -> String {
    format!(
        "\"transaction_id\":\"mvp-three-agent-{}\",\"escrow_id\":\"escrow-three-agent-{}\",\"settlement_tx_signature\":\"{}\",\"amount_lamports\":{},\"payer_pubkey\":\"{}\",\"recipient_pubkey\":\"{}\",\"settlement_commitment\":\"{}\"",
        escape_json(run_id),
        escape_json(run_id),
        escape_json(evidence.settlement_tx_signature.as_str()),
        evidence.lamports,
        escape_json(evidence.payer_pubkey.as_str()),
        escape_json(evidence.recipient_pubkey.as_str()),
        escape_json(evidence.settlement_commitment.as_str())
    )
}
