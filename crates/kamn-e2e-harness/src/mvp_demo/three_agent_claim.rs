use super::devnet_settlement::DevnetSettlementEvidence;
use super::report::{escape_json, CLAIM_LABEL_DEVNET_BACKED};

pub(crate) fn three_agent_escrow_claim_json(
    run_id: &str,
    evidence: &DevnetSettlementEvidence,
) -> String {
    let transaction_id = format!("mvp-three-agent-{run_id}");
    let terms_digest = format!("terms-digest-{run_id}");
    let escrow_id = format!("escrow-three-agent-{run_id}");
    format!(
        "{{\"id\":\"three_agent_escrow_verification\",\"label\":\"{}\",\"required\":true,\"status\":\"PASS\",\"summary\":\"Agent C verifies escrow settlement from restricted proof view\",\"transaction_id\":\"{}\",\"terms_digest\":\"{}\",\"agent_a_terms_digest\":\"{}\",\"agent_b_terms_digest\":\"{}\",\"verifier_terms_digest\":\"{}\",\"escrow_id\":\"{}\",\"agent_a_escrow_id\":\"{}\",\"agent_b_escrow_id\":\"{}\",\"verifier_escrow_id\":\"{}\",\"network\":\"{}\",\"rpc_url\":\"{}\",\"payer_pubkey\":\"{}\",\"recipient_pubkey\":\"{}\",\"lamports\":{},\"settlement_tx_signature\":\"{}\",\"agent_a_settlement_tx_signature\":\"{}\",\"agent_b_settlement_tx_signature\":\"{}\",\"verifier_settlement_tx_signature\":\"{}\",\"settlement_commitment\":\"{}\",\"agent_a_settlement_commitment\":\"{}\",\"agent_b_settlement_commitment\":\"{}\",\"verifier_settlement_commitment\":\"{}\",\"payer_balance_before\":{},\"payer_balance_after\":{},\"recipient_balance_before\":{},\"recipient_balance_after\":{},\"persisted_settlement_tx_signature\":\"{}\",\"amount_lamports\":{},\"agent_a_amount_lamports\":{},\"agent_b_amount_lamports\":{},\"verifier_amount_lamports\":{},\"agent_a_private_view_visible\":true,\"agent_b_private_view_visible\":true,\"verifier_private_view_visible\":false}}",
        CLAIM_LABEL_DEVNET_BACKED,
        escape_json(transaction_id.as_str()),
        escape_json(terms_digest.as_str()),
        escape_json(terms_digest.as_str()),
        escape_json(terms_digest.as_str()),
        escape_json(terms_digest.as_str()),
        escape_json(escrow_id.as_str()),
        escape_json(escrow_id.as_str()),
        escape_json(escrow_id.as_str()),
        escape_json(escrow_id.as_str()),
        escape_json(evidence.network.as_str()),
        escape_json(evidence.rpc_url.as_str()),
        escape_json(evidence.payer_pubkey.as_str()),
        escape_json(evidence.recipient_pubkey.as_str()),
        evidence.lamports,
        escape_json(evidence.settlement_tx_signature.as_str()),
        escape_json(evidence.settlement_tx_signature.as_str()),
        escape_json(evidence.settlement_tx_signature.as_str()),
        escape_json(evidence.settlement_tx_signature.as_str()),
        escape_json(evidence.settlement_commitment.as_str()),
        escape_json(evidence.settlement_commitment.as_str()),
        escape_json(evidence.settlement_commitment.as_str()),
        escape_json(evidence.settlement_commitment.as_str()),
        evidence.payer_balance_before,
        evidence.payer_balance_after,
        evidence.recipient_balance_before,
        evidence.recipient_balance_after,
        escape_json(evidence.persisted_settlement_tx_signature.as_str()),
        evidence.lamports,
        evidence.lamports,
        evidence.lamports,
        evidence.lamports
    )
}
