use std::path::Path;

use super::artifact_digest::{
    attach_json_digest, ArtifactJson, ThreeAgentReceiptDigests, ThreeAgentViewDigests,
};
use super::devnet_settlement::DevnetSettlementEvidence;
use super::report::escape_json;
use super::three_agent_receipt_spec::{agent_a_spec, agent_b_spec, ReceiptSpec};
use super::three_agent_receipts::{
    AGENT_A_RECEIPT_FILE, AGENT_B_RECEIPT_FILE, AGENT_C_RECEIPT_FILE,
};
use super::three_agent_views::public_view_digest;

const RECEIPT_DIGEST_FIELD: &str = "receipt_digest";

pub(crate) fn write_three_agent_receipts(
    run_id: &str,
    evidence: &DevnetSettlementEvidence,
    run_dir: &Path,
    views: &ThreeAgentViewDigests,
) -> Result<ThreeAgentReceiptDigests, String> {
    let agent_a = participant_receipt(run_id, evidence, run_dir, views, agent_a_spec())?;
    let agent_b = participant_receipt(run_id, evidence, run_dir, views, agent_b_spec())?;
    let agent_c = verifier_receipt(run_id, evidence, run_dir, views)?;
    write_receipt(run_dir, AGENT_A_RECEIPT_FILE, agent_a.json.as_str())?;
    write_receipt(run_dir, AGENT_B_RECEIPT_FILE, agent_b.json.as_str())?;
    write_receipt(run_dir, AGENT_C_RECEIPT_FILE, agent_c.json.as_str())?;
    Ok(ThreeAgentReceiptDigests {
        agent_a: agent_a.digest,
        agent_b: agent_b.digest,
        agent_c_verifier: agent_c.digest,
    })
}

fn write_receipt(run_dir: &Path, file_name: &str, json: &str) -> Result<(), String> {
    let path = run_dir.join("proof").join(file_name);
    std::fs::write(path.as_path(), json).map_err(|error| {
        format!(
            "failed to write three-agent observation receipt {}: {error}",
            path.display()
        )
    })
}

fn participant_receipt(
    run_id: &str,
    evidence: &DevnetSettlementEvidence,
    run_dir: &Path,
    views: &ThreeAgentViewDigests,
    spec: ReceiptSpec,
) -> Result<ArtifactJson, String> {
    attach_json_digest(
        format!(
            "{{\"schema_version\":\"kamn.mvp.three-agent-observation-receipt.v1\",\"agent\":\"{}\",\"action\":\"{}\",\"view_scope\":\"participant-private\",{},\"view_artifact\":\"{}\",\"view_digest\":\"{}\",\"participant_private_view_digest\":\"{}\",\"public_view_digest\":\"{}\",\"private_payload_redacted\":true,\"receipt_digest\":\"\"}}",
            spec.agent,
            spec.action,
            shared_fields(run_id, evidence),
            receipt_view_path(run_dir, spec.view_file),
            spec.view_digest(views),
            spec.private_digest(run_id),
            public_view_digest(run_id),
        ),
        RECEIPT_DIGEST_FIELD,
    )
}

fn verifier_receipt(
    run_id: &str,
    evidence: &DevnetSettlementEvidence,
    run_dir: &Path,
    views: &ThreeAgentViewDigests,
) -> Result<ArtifactJson, String> {
    attach_json_digest(
        format!(
            "{{\"schema_version\":\"kamn.mvp.three-agent-observation-receipt.v1\",\"agent\":\"agent_c_verifier\",\"action\":\"verify_three_agent_proof\",\"view_scope\":\"restricted-public\",{},\"view_artifact\":\"{}\",\"view_digest\":\"{}\",\"public_view_digest\":\"{}\",\"private_payload_redacted\":true,\"receipt_digest\":\"\"}}",
            shared_fields(run_id, evidence),
            receipt_view_path(run_dir, "agent-c-verifier-view.json"),
            views.agent_c_verifier,
            public_view_digest(run_id),
        ),
        RECEIPT_DIGEST_FIELD,
    )
}

fn shared_fields(run_id: &str, evidence: &DevnetSettlementEvidence) -> String {
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

fn receipt_view_path(run_dir: &Path, file_name: &str) -> String {
    run_dir.join("proof").join(file_name).display().to_string()
}
