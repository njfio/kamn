use super::super::super::*;
use crate::service_api_endpoint::live_settlement_dispatch::PreparedLiveSettlement;

pub(super) fn prepare(
    store: &mut ServiceApiMessageStore,
    actor: &str,
    escrow_id: &str,
    idempotency_key: &str,
    prepared: &PreparedLiveSettlement,
) -> Result<ServiceApiSettlementIntentRecord, String> {
    store.refresh_from_disk()?;
    validate_agreement(store, escrow_id, prepared)?;
    if let Some(existing) = store.snapshot.settlement_intents.get(escrow_id) {
        return identical_or_conflict(existing, actor, idempotency_key, prepared);
    }
    let record = build_record(actor, escrow_id, idempotency_key, prepared);
    store
        .snapshot
        .settlement_intents
        .insert(escrow_id.to_owned(), record.clone());
    store.persist()?;
    Ok(record)
}

pub(super) fn finalize(
    store: &mut ServiceApiMessageStore,
    escrow_id: &str,
    settlement: &ServiceApiSettlementMetadata,
) -> Result<Option<ServiceApiEscrowStatusBody>, String> {
    store.refresh_from_disk()?;
    let Some(intent) = store.snapshot.settlement_intents.get_mut(escrow_id) else {
        return Err("settlement intent missing during finalize".to_owned());
    };
    intent.state = "confirmed".to_owned();
    let response = release_without_refresh(store, escrow_id, settlement)?;
    store.persist()?;
    Ok(response)
}

fn validate_agreement(
    store: &ServiceApiMessageStore,
    escrow_id: &str,
    prepared: &PreparedLiveSettlement,
) -> Result<(), String> {
    let escrow = store
        .snapshot
        .escrows
        .get(escrow_id)
        .ok_or_else(|| "settlement escrow missing".to_owned())?;
    if escrow.amount_lamports != Some(prepared.amount_lamports)
        || escrow.network.as_deref() != Some("solana-devnet")
        || prepared.network != "solana:devnet"
    {
        return Err("SETTLEMENT_AGREEMENT_MISMATCH".to_owned());
    }
    Ok(())
}

fn identical_or_conflict(
    existing: &ServiceApiSettlementIntentRecord,
    actor: &str,
    key: &str,
    prepared: &PreparedLiveSettlement,
) -> Result<ServiceApiSettlementIntentRecord, String> {
    if existing.actor_did == actor
        && existing.idempotency_key == key
        && existing.expected_signature == prepared.expected_signature
        && existing.signed_transaction_digest == prepared.signed_transaction_digest
    {
        return Ok(existing.clone());
    }
    Err("SETTLEMENT_INTENT_CONFLICT".to_owned())
}

fn build_record(
    actor: &str,
    escrow_id: &str,
    key: &str,
    prepared: &PreparedLiveSettlement,
) -> ServiceApiSettlementIntentRecord {
    ServiceApiSettlementIntentRecord {
        settlement_intent_id: format!("settlement-intent-{escrow_id}"),
        escrow_id: escrow_id.to_owned(),
        actor_did: actor.to_owned(),
        idempotency_key: key.to_owned(),
        recipient_pubkey: prepared.recipient_pubkey.clone(),
        amount_lamports: prepared.amount_lamports,
        network: prepared.network.clone(),
        expected_signature: prepared.expected_signature.clone(),
        signed_transaction_digest: prepared.signed_transaction_digest.clone(),
        signed_transaction_json: prepared.signed_transaction_json.clone(),
        state: "prepared".to_owned(),
        submission_attempt_count: 0,
        last_error_code: None,
    }
}

fn release_without_refresh(
    store: &mut ServiceApiMessageStore,
    escrow_id: &str,
    settlement: &ServiceApiSettlementMetadata,
) -> Result<Option<ServiceApiEscrowStatusBody>, String> {
    let Some(record) = store.snapshot.escrows.get_mut(escrow_id) else {
        return Ok(None);
    };
    super::settlement::release_escrow_record(record, Some(settlement));
    Ok(Some(super::settlement::escrow_status_response(record)))
}
