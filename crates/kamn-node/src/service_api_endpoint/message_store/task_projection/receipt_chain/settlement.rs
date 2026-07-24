use super::*;

pub(super) fn append(
    snapshot: &ServiceApiPersistedMessageStoreSnapshot,
    escrow: &ServiceApiPersistedEscrowRecord,
    entries: &mut Vec<ReceiptChainEntry>,
) -> Result<(), TaskProjectionError> {
    let Some(signature) = escrow.settlement.settlement_tx_signature.as_deref() else {
        return Ok(());
    };
    let intent = snapshot
        .settlement_intents
        .get(&escrow.escrow_id)
        .ok_or(TaskProjectionError::ReceiptChainInvalid)?;
    require_binding(escrow, intent, signature)?;
    entries.push(entry(intent));
    Ok(())
}

fn require_binding(
    escrow: &ServiceApiPersistedEscrowRecord,
    intent: &ServiceApiSettlementIntentRecord,
    signature: &str,
) -> Result<(), TaskProjectionError> {
    let valid = intent.state == "confirmed"
        && intent.escrow_id == escrow.escrow_id
        && escrow.release_authority_did.as_deref() == Some(intent.actor_did.as_str())
        && intent.expected_signature == signature
        && escrow.amount_lamports == Some(intent.amount_lamports)
        && intent.network == "solana:devnet"
        && bridge_binding_matches(escrow, intent);
    valid
        .then_some(())
        .ok_or(TaskProjectionError::ReceiptChainInvalid)
}

fn bridge_binding_matches(
    escrow: &ServiceApiPersistedEscrowRecord,
    intent: &ServiceApiSettlementIntentRecord,
) -> bool {
    match intent.bridge_receipt_digest.as_deref() {
        Some(bridge_receipt_digest) => {
            escrow.settlement.bridge_receipt_digest.as_deref() == Some(bridge_receipt_digest)
                && escrow.settlement.bridge_transaction_signature.as_deref()
                    == intent.bridge_transaction_signature.as_deref()
        }
        None => escrow.settlement.bridge_receipt_digest.is_none(),
    }
}

fn entry(intent: &ServiceApiSettlementIntentRecord) -> ReceiptChainEntry {
    ReceiptChainEntry {
        receipt_id: intent.settlement_intent_id.clone(),
        receipt_digest: authority_digest::settlement(intent),
        authorization_digest: String::new(),
        actor_did: intent.actor_did.clone(),
        action: "settlement:confirmed".to_owned(),
        resource_id: intent.escrow_id.clone(),
        correlation_id: String::new(),
        idempotency_key: intent.idempotency_key.clone(),
        prior_state: "submitted".to_owned(),
        resulting_state: intent.state.clone(),
    }
}
