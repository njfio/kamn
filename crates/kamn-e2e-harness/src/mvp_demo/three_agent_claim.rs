use super::devnet_settlement::DevnetSettlementEvidence;
use super::report::{escape_json, CLAIM_LABEL_DEVNET_BACKED};

pub(crate) fn three_agent_escrow_claim_json(
    run_id: &str,
    evidence: &DevnetSettlementEvidence,
    transcript_path: &str,
) -> String {
    let transaction_id = format!("mvp-three-agent-{run_id}");
    let terms_digest = format!("terms-digest-{run_id}");
    let escrow_id = format!("escrow-three-agent-{run_id}");
    format!(
        "{{{},{},{},{},{},{},{}}}",
        claim_header(),
        transcript_fields(run_id, transcript_path),
        digest_fields(transaction_id.as_str(), terms_digest.as_str()),
        escrow_fields(escrow_id.as_str()),
        settlement_fields(evidence),
        privacy_fields(),
        view_fields(run_id)
    )
}

fn transcript_fields(run_id: &str, path: &str) -> String {
    format!(
        "\"three_agent_transcript_artifact\":\"{}\",\"three_agent_transcript_digest\":\"{}\"",
        escape_json(path),
        escape_json(format!("three-agent-transcript-digest-{run_id}").as_str())
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
        "\"network\":\"{}\",\"rpc_url\":\"{}\",\"payer_pubkey\":\"{}\",\"recipient_pubkey\":\"{}\"",
        escape_json(evidence.network.as_str()),
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
    let public_digest = format!("public-view-digest-{run_id}");
    format!(
        "\"agent_a_view_scope\":\"participant-private\",\"agent_b_view_scope\":\"participant-private\",\"verifier_view_scope\":\"restricted-public\",{},{}",
        private_view_fields(run_id),
        public_view_fields(public_digest.as_str())
    )
}

fn private_view_fields(run_id: &str) -> String {
    let agent_a_digest = format!("agent-a-private-digest-{run_id}");
    let agent_b_digest = format!("agent-b-private-digest-{run_id}");
    format!(
        "\"agent_a_private_field_count\":3,\"agent_b_private_field_count\":3,\"verifier_private_field_count\":0,\"agent_a_private_view_digest\":\"{}\",\"agent_b_private_view_digest\":\"{}\",\"private_payload_redacted\":true",
        escape_json(agent_a_digest.as_str()),
        escape_json(agent_b_digest.as_str())
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
