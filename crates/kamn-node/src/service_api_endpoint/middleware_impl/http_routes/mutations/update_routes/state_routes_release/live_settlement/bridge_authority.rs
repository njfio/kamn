use super::errors::{
    bridge_settlement_authority_mismatch_error, bridge_settlement_authority_missing_error,
    bridge_settlement_receipt_replay_error, invalid_release_key, settlement_intent_conflict_error,
};
use super::*;
use crate::service_api_endpoint::message_store::{
    bridge_receipt_digest, BridgeSettlementIntentInput, ServiceApiBridgeReceiptRecord,
    ServiceApiBridgeSettlementTermsRecord,
};

pub(super) struct ReleaseRequest {
    pub(super) idempotency_key: String,
    pub(super) bridge_id: Option<String>,
}

pub(super) struct ValidatedBridgeSettlementAuthority {
    pub(super) bridge_id: String,
    pub(super) recipient_pubkey: String,
    pub(super) receipt: ServiceApiBridgeReceiptRecord,
    pub(super) receipt_terms: ServiceApiBridgeSettlementTermsRecord,
}

pub(super) fn release_request(
    context: &ServiceApiRequestContext,
) -> Result<ReleaseRequest, Box<Response>> {
    let value: serde_json::Value = serde_json::from_str(context.parsed_request.body.as_str())
        .map_err(|error| Box::new(invalid_release_key(error.to_string().as_str())))?;
    let idempotency_key = value
        .get("idempotency_key")
        .and_then(serde_json::Value::as_str)
        .map(str::trim)
        .filter(|key| !key.is_empty())
        .map(str::to_owned)
        .ok_or_else(|| Box::new(invalid_release_key("release idempotency key is required")))?;
    let bridge_id = match value
        .get("authority_mode")
        .and_then(serde_json::Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        Some("bridge-receipt") => Some(
            value
                .get("bridge_id")
                .and_then(serde_json::Value::as_str)
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .map(str::to_owned)
                .ok_or_else(|| Box::new(bridge_settlement_authority_missing_error()))?,
        ),
        Some(_) | None => None,
    };
    Ok(ReleaseRequest {
        idempotency_key,
        bridge_id,
    })
}

pub(super) fn persist_bridge_intent(
    store: &mut ServiceApiMessageStore,
    actor: &str,
    escrow_id: &str,
    idempotency_key: &str,
    authority: &ValidatedBridgeSettlementAuthority,
) -> Result<(), Box<Response>> {
    let input = BridgeSettlementIntentInput {
        bridge_id: authority.bridge_id.clone(),
        bridge_receipt_id: authority.receipt.receipt_id.clone(),
        bridge_receipt_digest: authority.receipt.receipt_digest.clone(),
        bridge_transaction_signature: authority.receipt.transaction_signature.clone(),
        recipient_pubkey: authority.recipient_pubkey.clone(),
        amount_lamports: authority.receipt_terms.amount_lamports,
        network: authority.receipt_terms.network.clone(),
        asset: authority.receipt_terms.asset.clone(),
        terms_digest: authority.receipt_terms.terms_digest.clone(),
        task_id: authority.receipt_terms.task_id.clone(),
    };
    match store.prepare_bridge_settlement_intent(actor, escrow_id, idempotency_key, &input) {
        Ok(_) => Ok(()),
        Err(error) if error == "SETTLEMENT_INTENT_CONFLICT" => {
            Err(Box::new(settlement_intent_conflict_error()))
        }
        Err(error) if error == "BRIDGE_SETTLEMENT_RECEIPT_REPLAY" => {
            Err(Box::new(bridge_settlement_receipt_replay_error()))
        }
        Err(error) => Err(Box::new(live_settlement_evidence_error(error.as_str()))),
    }
}

pub(super) fn validate_bridge_settlement_authority(
    store: &mut ServiceApiMessageStore,
    config: &LiveSolanaSettlementConfig,
    actor: &str,
    escrow_id: &str,
    bridge_id: &str,
) -> Result<ValidatedBridgeSettlementAuthority, Box<Response>> {
    let escrow = store
        .snapshot
        .escrows
        .get(escrow_id)
        .cloned()
        .ok_or_else(|| Box::new(bridge_settlement_authority_missing_error()))?;
    let receipt = store
        .snapshot
        .bridges
        .get(bridge_id)
        .and_then(|bridge| bridge.bridge_receipt.clone())
        .ok_or_else(|| Box::new(bridge_settlement_authority_missing_error()))?;
    let receipt_terms = receipt
        .settlement_authority
        .clone()
        .ok_or_else(|| Box::new(bridge_settlement_authority_missing_error()))?;
    if bridge_receipt_digest(&receipt) != receipt.receipt_digest {
        return Err(Box::new(bridge_settlement_authority_mismatch_error()));
    }
    if replayed_bridge_receipt(store, escrow_id, &receipt) {
        return Err(Box::new(bridge_settlement_receipt_replay_error()));
    }
    let valid = receipt.bridge_id == bridge_id
        && receipt.resource_id == bridge_id
        && receipt.state == "finalized"
        && receipt.network == "solana:devnet"
        && receipt.commitment == config.commitment_label()
        && receipt_terms.escrow_id == escrow_id
        && escrow.task_id.as_deref() == Some(receipt_terms.task_id.as_str())
        && receipt_terms.actor_did == actor
        && escrow.amount_lamports == Some(receipt_terms.amount_lamports)
        && receipt_terms.asset == "lamports"
        && receipt_terms.network == "solana:devnet"
        && escrow.terms_digest.as_deref() == Some(receipt_terms.terms_digest.as_str())
        && receipt_terms.recipient_pubkey == config.recipient_pubkey();
    valid
        .then_some(ValidatedBridgeSettlementAuthority {
            bridge_id: bridge_id.to_owned(),
            recipient_pubkey: config.recipient_pubkey(),
            receipt,
            receipt_terms,
        })
        .ok_or_else(|| Box::new(bridge_settlement_authority_mismatch_error()))
}

pub(super) fn consume_finalized_bridge_receipt(
    store: &mut ServiceApiMessageStore,
    escrow_id: &str,
    authority: &ValidatedBridgeSettlementAuthority,
) -> Result<Option<ServiceApiEscrowStatusBody>, String> {
    let without_resubmission = settlement_metadata_from_bridge_receipt(authority);
    store.finalize_settlement_intent(escrow_id, &without_resubmission)
}

fn settlement_metadata_from_bridge_receipt(
    authority: &ValidatedBridgeSettlementAuthority,
) -> ServiceApiSettlementMetadata {
    ServiceApiSettlementMetadata {
        settlement_receipt_hash: Some(authority.receipt.transaction_signature.clone()),
        settlement_tx_signature: Some(authority.receipt.transaction_signature.clone()),
        settlement_network: Some(authority.receipt.network.clone()),
        settlement_commitment: Some(authority.receipt.commitment.clone()),
        bridge_id: Some(authority.bridge_id.clone()),
        bridge_receipt_id: Some(authority.receipt.receipt_id.clone()),
        bridge_receipt_digest: Some(authority.receipt.receipt_digest.clone()),
        bridge_transaction_signature: Some(authority.receipt.transaction_signature.clone()),
    }
}

fn replayed_bridge_receipt(
    store: &ServiceApiMessageStore,
    escrow_id: &str,
    receipt: &ServiceApiBridgeReceiptRecord,
) -> bool {
    store.snapshot.settlement_intents.values().any(|intent| {
        intent.escrow_id != escrow_id
            && (intent.bridge_receipt_id.as_deref() == Some(receipt.receipt_id.as_str())
                || intent.bridge_receipt_digest.as_deref() == Some(receipt.receipt_digest.as_str()))
    })
}
