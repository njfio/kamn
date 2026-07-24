use super::*;

pub(super) fn resolve_prepared(
    store: &mut ServiceApiMessageStore,
    config: &LiveSolanaSettlementConfig,
    escrow_id: &str,
) -> Result<PreparedLiveSettlement, Box<Response>> {
    let existing = store
        .get_settlement_intent(escrow_id)
        .map_err(super::persistence_error)?;
    if let Some(intent) = existing {
        return Ok(prepared_from_intent(intent));
    }
    live_settlement_dispatch::prepare_live_settlement(config, escrow_id)
        .map_err(|error| Box::new(live_settlement_evidence_error(error.as_str())))
}

fn prepared_from_intent(intent: ServiceApiSettlementIntentRecord) -> PreparedLiveSettlement {
    PreparedLiveSettlement {
        expected_signature: intent.expected_signature,
        signed_transaction_digest: intent.signed_transaction_digest,
        signed_transaction_json: intent.signed_transaction_json,
        recipient_pubkey: intent.recipient_pubkey,
        amount_lamports: intent.amount_lamports,
        network: intent.network,
    }
}
