use std::path::Path;

use super::devnet_settlement::DevnetSettlementEvidence;
use super::report::escape_json;
use super::three_agent_views::{
    agent_a_view_digest, agent_b_view_digest, agent_c_verifier_view_digest,
};

pub(crate) fn transcript_json(
    run_id: &str,
    evidence: &DevnetSettlementEvidence,
    run_dir: &Path,
) -> String {
    format!(
        "{{\"schema_version\":\"kamn.mvp.three-agent-transcript.v1\",\"proof_label\":\"local-only\",\"devnet_settlement_linked\":true,\"transaction_id\":\"mvp-three-agent-{}\",\"escrow_id\":\"escrow-three-agent-{}\",{},\"views\":{},\"agent_a_private_field_count\":3,\"agent_b_private_field_count\":3,\"verifier_private_field_count\":0,\"private_payload_redacted\":true,{},\"transcript_digest\":\"{}\",{}}}",
        escape_json(run_id),
        escape_json(run_id),
        steps_json(),
        views_json(),
        settlement_json(evidence),
        escape_json(format!("three-agent-transcript-digest-{run_id}").as_str()),
        transcript_view_fields(run_id, run_dir)
    )
}

fn steps_json() -> &'static str {
    "\"steps\":[\"agent_a_registered\",\"agent_b_registered\",\"agent_a_invoked_transaction\",\"agent_b_accepted_task\",\"escrow_funded\",\"escrow_released\",\"agent_c_verified\"]"
}

fn views_json() -> &'static str {
    "{\"agent_a\":\"participant-private\",\"agent_b\":\"participant-private\",\"agent_c\":\"restricted-public\"}"
}

fn settlement_json(evidence: &DevnetSettlementEvidence) -> String {
    format!(
        "\"settlement_tx_signature\":\"{}\",\"amount_lamports\":{},\"payer_pubkey\":\"{}\",\"recipient_pubkey\":\"{}\",\"settlement_commitment\":\"{}\"",
        escape_json(evidence.settlement_tx_signature.as_str()),
        evidence.lamports,
        escape_json(evidence.payer_pubkey.as_str()),
        escape_json(evidence.recipient_pubkey.as_str()),
        escape_json(evidence.settlement_commitment.as_str())
    )
}

fn transcript_view_fields(run_id: &str, run_dir: &Path) -> String {
    format!(
        "\"agent_a_view_artifact\":\"{}\",\"agent_b_view_artifact\":\"{}\",\"agent_c_verifier_view_artifact\":\"{}\",\"agent_a_view_digest\":\"{}\",\"agent_b_view_digest\":\"{}\",\"agent_c_verifier_view_digest\":\"{}\"",
        proof_artifact_path(run_dir, "agent-a-view.json"),
        proof_artifact_path(run_dir, "agent-b-view.json"),
        proof_artifact_path(run_dir, "agent-c-verifier-view.json"),
        escape_json(agent_a_view_digest(run_id).as_str()),
        escape_json(agent_b_view_digest(run_id).as_str()),
        escape_json(agent_c_verifier_view_digest(run_id).as_str())
    )
}

fn proof_artifact_path(run_dir: &Path, file_name: &str) -> String {
    escape_json(
        run_dir
            .join("proof")
            .join(file_name)
            .display()
            .to_string()
            .as_str(),
    )
}
