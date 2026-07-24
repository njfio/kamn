use super::*;
use crate::service_api_endpoint::escrow_models::ServiceApiAuthoritativeSettlement;

pub(super) fn build(
    snapshot: &ServiceApiPersistedMessageStoreSnapshot,
    receipt: &ServiceApiEscrowTransitionReceiptRecord,
    intent: &ServiceApiSettlementIntentRecord,
) -> Result<Option<ServiceApiAuthoritativeSettlement>, String> {
    let Some(bridge_id) = intent.bridge_id.as_deref() else {
        return Ok(None);
    };
    let bridge = finalized_bridge_receipt(snapshot, bridge_id)?;
    let chain = super::super::super::task_projection::receipt_chain_commitment(
        snapshot,
        intent.task_id.as_str(),
    )
    .map_err(|_| "SETTLEMENT_RECEIPT_INVALID: receipt chain mismatch".to_owned())?;
    Ok(Some(fields(receipt, intent, bridge, chain)))
}

fn finalized_bridge_receipt<'a>(
    snapshot: &'a ServiceApiPersistedMessageStoreSnapshot,
    bridge_id: &str,
) -> Result<&'a ServiceApiBridgeReceiptRecord, String> {
    snapshot
        .bridges
        .get(bridge_id)
        .and_then(|record| record.bridge_receipt.as_ref())
        .filter(|receipt| {
            receipt.state == "finalized"
                && authority_digest::bridge(receipt) == receipt.receipt_digest
        })
        .ok_or_else(|| "SETTLEMENT_RECEIPT_INVALID: bridge receipt mismatch".to_owned())
}

fn fields(
    receipt: &ServiceApiEscrowTransitionReceiptRecord,
    intent: &ServiceApiSettlementIntentRecord,
    bridge: &ServiceApiBridgeReceiptRecord,
    receipt_chain_commitment: String,
) -> ServiceApiAuthoritativeSettlement {
    ServiceApiAuthoritativeSettlement {
        bridge_id: bridge.bridge_id.clone(),
        bridge_receipt_id: bridge.receipt_id.clone(),
        bridge_receipt_digest: bridge.receipt_digest.clone(),
        settlement_receipt_id: intent.settlement_intent_id.clone(),
        settlement_receipt_digest: authority_digest::settlement(intent),
        action: "settlement:confirmed".to_owned(),
        resource_id: intent.escrow_id.clone(),
        actor_did: intent.actor_did.clone(),
        resulting_state: intent.state.clone(),
        task_id: intent.task_id.clone(),
        escrow_id: intent.escrow_id.clone(),
        recipient: intent.recipient_pubkey.clone(),
        amount_lamports: intent.amount_lamports,
        asset: intent.asset.clone(),
        network: intent.network.clone(),
        transaction_signature: intent.expected_signature.clone(),
        commitment: bridge.commitment.clone(),
        finalized_slot: bridge.finalized_slot,
        receipt_chain_commitment,
        terms_digest: receipt.terms_digest.clone(),
        idempotency_key: intent.idempotency_key.clone(),
    }
}
