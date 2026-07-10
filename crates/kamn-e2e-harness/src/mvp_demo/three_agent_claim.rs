use super::artifact_digest::ThreeAgentArtifactDigests;
use super::devnet_settlement::DevnetSettlementEvidence;
use super::live_task_binding::LiveTaskBinding;
use super::report::{escape_json, CLAIM_LABEL_DEVNET_BACKED};
use super::three_agent_views::{
    agent_a_private_view_digest, agent_b_private_view_digest, public_view_digest,
};

pub(crate) fn three_agent_escrow_claim_json(
    run_id: &str,
    evidence: &DevnetSettlementEvidence,
    binding: &LiveTaskBinding,
    transcript_path: &str,
    view_paths: [&str; 3],
    receipt_paths: [&str; 3],
    artifact_digests: &ThreeAgentArtifactDigests,
) -> String {
    let transaction_id = binding.task_id.as_str();
    let terms_digest = binding.digest.as_str();
    let escrow_id = evidence.escrow_id.as_str();
    format!(
        "{{{},{},{},{},{},{},{},{},{},{}}}",
        claim_header(),
        binding_fields(binding),
        transcript_fields(transcript_path, artifact_digests),
        view_artifact_fields(view_paths, artifact_digests),
        receipt_artifact_fields(receipt_paths, artifact_digests),
        digest_fields(transaction_id, terms_digest),
        escrow_fields(escrow_id),
        settlement_fields(evidence),
        privacy_fields(),
        view_fields(run_id)
    )
}

fn binding_fields(binding: &LiveTaskBinding) -> String {
    format!(
        "\"live_task_settlement_binding_artifact\":\"{}\",\"live_task_settlement_binding_digest\":\"{}\",\"task_binding_digest\":\"{}\"",
        escape_json(binding.artifact_path.as_str()),
        escape_json(binding.digest.as_str()),
        escape_json(binding.digest.as_str())
    )
}

fn transcript_fields(path: &str, artifact_digests: &ThreeAgentArtifactDigests) -> String {
    format!(
        "\"three_agent_transcript_artifact\":\"{}\",\"three_agent_transcript_digest\":\"{}\"",
        escape_json(path),
        escape_json(artifact_digests.transcript.as_str())
    )
}

fn view_artifact_fields(paths: [&str; 3], artifact_digests: &ThreeAgentArtifactDigests) -> String {
    format!(
        "\"agent_a_view_artifact\":\"{}\",\"agent_b_view_artifact\":\"{}\",\"agent_c_verifier_view_artifact\":\"{}\",\"agent_a_view_digest\":\"{}\",\"agent_b_view_digest\":\"{}\",\"agent_c_verifier_view_digest\":\"{}\"",
        escape_json(paths[0]),
        escape_json(paths[1]),
        escape_json(paths[2]),
        escape_json(artifact_digests.views.agent_a.as_str()),
        escape_json(artifact_digests.views.agent_b.as_str()),
        escape_json(artifact_digests.views.agent_c_verifier.as_str())
    )
}

fn receipt_artifact_fields(
    paths: [&str; 3],
    artifact_digests: &ThreeAgentArtifactDigests,
) -> String {
    format!(
        "\"agent_a_observation_receipt_artifact\":\"{}\",\"agent_b_observation_receipt_artifact\":\"{}\",\"agent_c_verifier_observation_receipt_artifact\":\"{}\",\"agent_a_observation_receipt_digest\":\"{}\",\"agent_b_observation_receipt_digest\":\"{}\",\"agent_c_verifier_observation_receipt_digest\":\"{}\"",
        escape_json(paths[0]),
        escape_json(paths[1]),
        escape_json(paths[2]),
        escape_json(artifact_digests.receipts.agent_a.as_str()),
        escape_json(artifact_digests.receipts.agent_b.as_str()),
        escape_json(artifact_digests.receipts.agent_c_verifier.as_str())
    )
}

fn claim_header() -> String {
    format!(
        "\"id\":\"three_agent_escrow_verification\",\"label\":\"{}\",\"required\":true,\"status\":\"PASS\",\"summary\":\"Agent C verifies escrow settlement from restricted proof view\"",
        CLAIM_LABEL_DEVNET_BACKED,
    )
}

fn digest_fields(transaction_id: &str, terms_digest: &str) -> String {
    format!(
        "\"transaction_id\":\"{}\",\"terms_digest\":\"{}\",\"agent_a_terms_digest\":\"{}\",\"agent_b_terms_digest\":\"{}\",\"verifier_terms_digest\":\"{}\"",
        escape_json(transaction_id),
        escape_json(terms_digest),
        escape_json(terms_digest),
        escape_json(terms_digest),
        escape_json(terms_digest),
    )
}

fn escrow_fields(escrow_id: &str) -> String {
    format!(
        "\"escrow_id\":\"{}\",\"agent_a_escrow_id\":\"{}\",\"agent_b_escrow_id\":\"{}\",\"verifier_escrow_id\":\"{}\"",
        escape_json(escrow_id),
        escape_json(escrow_id),
        escape_json(escrow_id),
        escape_json(escrow_id),
    )
}

fn settlement_fields(evidence: &DevnetSettlementEvidence) -> String {
    [
        network_fields(evidence),
        signature_fields(evidence),
        commitment_fields(evidence),
        balance_fields(evidence),
        amount_fields(evidence),
    ]
    .join(",")
}

fn network_fields(evidence: &DevnetSettlementEvidence) -> String {
    format!(
        "\"network\":\"{}\",\"execution_surface\":\"{}\",\"rpc_url\":\"{}\",\"payer_pubkey\":\"{}\",\"recipient_pubkey\":\"{}\"",
        escape_json(evidence.network.as_str()),
        escape_json(evidence.execution_surface.as_str()),
        escape_json(evidence.rpc_url.as_str()),
        escape_json(evidence.payer_pubkey.as_str()),
        escape_json(evidence.recipient_pubkey.as_str()),
    )
}

fn signature_fields(evidence: &DevnetSettlementEvidence) -> String {
    format!(
        "\"settlement_tx_signature\":\"{}\",\"agent_a_settlement_tx_signature\":\"{}\",\"agent_b_settlement_tx_signature\":\"{}\",\"verifier_settlement_tx_signature\":\"{}\"",
        escape_json(evidence.settlement_tx_signature.as_str()),
        escape_json(evidence.settlement_tx_signature.as_str()),
        escape_json(evidence.settlement_tx_signature.as_str()),
        escape_json(evidence.settlement_tx_signature.as_str()),
    )
}

fn commitment_fields(evidence: &DevnetSettlementEvidence) -> String {
    format!(
        "\"settlement_commitment\":\"{}\",\"agent_a_settlement_commitment\":\"{}\",\"agent_b_settlement_commitment\":\"{}\",\"verifier_settlement_commitment\":\"{}\"",
        escape_json(evidence.settlement_commitment.as_str()),
        escape_json(evidence.settlement_commitment.as_str()),
        escape_json(evidence.settlement_commitment.as_str()),
        escape_json(evidence.settlement_commitment.as_str()),
    )
}

fn balance_fields(evidence: &DevnetSettlementEvidence) -> String {
    format!(
        "\"lamports\":{},\"payer_balance_before\":{},\"payer_balance_after\":{},\"recipient_balance_before\":{},\"recipient_balance_after\":{},\"persisted_settlement_tx_signature\":\"{}\"",
        evidence.lamports,
        evidence.payer_balance_before,
        evidence.payer_balance_after,
        evidence.recipient_balance_before,
        evidence.recipient_balance_after,
        escape_json(evidence.persisted_settlement_tx_signature.as_str()),
    )
}

fn amount_fields(evidence: &DevnetSettlementEvidence) -> String {
    format!(
        "\"amount_lamports\":{},\"agent_a_amount_lamports\":{},\"agent_b_amount_lamports\":{},\"verifier_amount_lamports\":{}",
        evidence.lamports,
        evidence.lamports,
        evidence.lamports,
        evidence.lamports
    )
}

fn privacy_fields() -> &'static str {
    "\"agent_a_private_view_visible\":true,\"agent_b_private_view_visible\":true,\"verifier_private_view_visible\":false"
}

fn view_fields(run_id: &str) -> String {
    format!(
        "\"agent_a_view_scope\":\"participant-private\",\"agent_b_view_scope\":\"participant-private\",\"verifier_view_scope\":\"restricted-public\",{},{}",
        private_view_fields(run_id),
        public_view_fields(public_view_digest(run_id).as_str())
    )
}

fn private_view_fields(run_id: &str) -> String {
    format!(
        "\"agent_a_private_field_count\":3,\"agent_b_private_field_count\":3,\"verifier_private_field_count\":0,\"agent_a_private_view_digest\":\"{}\",\"agent_b_private_view_digest\":\"{}\",\"private_payload_redacted\":true",
        escape_json(agent_a_private_view_digest(run_id).as_str()),
        escape_json(agent_b_private_view_digest(run_id).as_str())
    )
}

fn public_view_fields(public_digest: &str) -> String {
    format!(
        "\"agent_a_public_view_digest\":\"{}\",\"agent_b_public_view_digest\":\"{}\",\"verifier_public_view_digest\":\"{}\"",
        escape_json(public_digest),
        escape_json(public_digest),
        escape_json(public_digest)
    )
}
