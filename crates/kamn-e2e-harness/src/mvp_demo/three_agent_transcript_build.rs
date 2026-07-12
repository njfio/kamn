use std::path::Path;

use super::artifact_digest::{attach_json_digest, ArtifactJson, ThreeAgentViewDigests};
use super::devnet_settlement::DevnetSettlementEvidence;
use super::live_task_binding::LiveTaskBinding;
use super::report::escape_json;

pub(crate) fn transcript_json(
    _run_id: &str,
    evidence: &DevnetSettlementEvidence,
    binding: &LiveTaskBinding,
    run_dir: &Path,
    view_digests: &ThreeAgentViewDigests,
) -> Result<ArtifactJson, String> {
    attach_json_digest(
        format!(
            "{{\"schema_version\":\"kamn.mvp.three-agent-transcript.v1\",\"proof_label\":\"local-only\",\"devnet_settlement_linked\":true,\"transaction_id\":\"{}\",\"escrow_id\":\"{}\",\"task_binding_digest\":\"{}\",{},\"views\":{},\"agent_a_private_field_count\":3,\"agent_b_private_field_count\":3,\"verifier_private_field_count\":0,\"private_payload_redacted\":true,{},\"transcript_digest\":\"\",{}}}",
            escape_json(binding.transaction_id.as_str()),
            escape_json(evidence.escrow_id.as_str()),
            escape_json(binding.digest.as_str()),
            steps_json(),
            views_json(),
            settlement_json(evidence),
            transcript_view_fields(run_dir, view_digests)
        ),
        "transcript_digest",
    )
}

fn steps_json() -> &'static str {
    "\"steps\":[\"agent_a_registered\",\"agent_b_registered\",\"agent_a_invoked_transaction\",\"agent_b_accepted_task\",\"escrow_funded\",\"escrow_released\",\"agent_c_verifier_verified\"]"
}

fn views_json() -> &'static str {
    "{\"agent_a\":\"participant-private\",\"agent_b\":\"participant-private\",\"agent_c_verifier\":\"restricted-public\"}"
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

fn transcript_view_fields(run_dir: &Path, view_digests: &ThreeAgentViewDigests) -> String {
    format!(
        "\"agent_a_view_artifact\":\"{}\",\"agent_b_view_artifact\":\"{}\",\"agent_c_verifier_view_artifact\":\"{}\",\"agent_a_view_digest\":\"{}\",\"agent_b_view_digest\":\"{}\",\"agent_c_verifier_view_digest\":\"{}\"",
        proof_artifact_path(run_dir, "agent-a-view.json"),
        proof_artifact_path(run_dir, "agent-b-view.json"),
        proof_artifact_path(run_dir, "agent-c-verifier-view.json"),
        escape_json(view_digests.agent_a.as_str()),
        escape_json(view_digests.agent_b.as_str()),
        escape_json(view_digests.agent_c_verifier.as_str())
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
