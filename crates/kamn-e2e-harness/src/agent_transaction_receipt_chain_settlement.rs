use super::agent_transaction_persisted_settlement::ExpectedSettlement;
use super::agent_transaction_receipt_chain::invalid;
use super::agent_transaction_receipt_chain_digest as digest;
use super::agent_transaction_receipt_chain_model::{ChainEntry, Escrow, State};

pub(super) fn settlement_entry(
    state: &State,
    escrow: &Escrow,
    expected: &ExpectedSettlement<'_>,
) -> Result<ChainEntry, String> {
    let intent = state
        .settlement_intents
        .get(expected.escrow_id)
        .ok_or_else(invalid)?;
    validate_intent(intent, escrow, expected)?;
    Ok(ChainEntry {
        receipt_id: intent.settlement_intent_id.clone(),
        receipt_digest: digest::settlement(intent),
        authorization_digest: String::new(),
        actor_did: intent.actor_did.clone(),
        action: "settlement:confirmed".to_owned(),
        resource_id: intent.escrow_id.clone(),
        correlation_id: String::new(),
        idempotency_key: intent.idempotency_key.clone(),
        prior_state: "submitted".to_owned(),
        resulting_state: intent.state.clone(),
    })
}

fn validate_intent(
    intent: &super::agent_transaction_receipt_chain_model::SettlementIntent,
    escrow: &Escrow,
    expected: &ExpectedSettlement<'_>,
) -> Result<(), String> {
    let valid = intent.state == "confirmed"
        && intent.escrow_id == escrow.escrow_id
        && escrow.release_authority_did.as_deref() == Some(intent.actor_did.as_str())
        && intent.expected_signature == expected.signature
        && escrow.settlement_tx_signature.as_deref() == Some(expected.signature)
        && escrow.amount_lamports == Some(intent.amount_lamports)
        && intent.network == "solana:devnet";
    valid.then_some(()).ok_or_else(invalid)
}
