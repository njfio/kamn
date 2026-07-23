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
    if !settlement_signature_is_available(
        &store.snapshot,
        escrow_id,
        prepared.expected_signature.as_str(),
    ) {
        return Err("SETTLEMENT_SIGNATURE_REUSE".to_owned());
    }
    let record = build_record(actor, escrow_id, idempotency_key, prepared);
    store
        .snapshot
        .settlement_intents
        .insert(escrow_id.to_owned(), record.clone());
    store.persist()?;
    Ok(record)
}

pub(crate) fn settlement_signature_is_available(
    snapshot: &ServiceApiPersistedMessageStoreSnapshot,
    escrow_id: &str,
    signature: &str,
) -> bool {
    snapshot
        .settlement_intents
        .values()
        .all(|intent| intent.escrow_id == escrow_id || intent.expected_signature != signature)
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
    record_submission(intent);
    intent.last_error_code = None;
    let response = release_without_refresh(store, escrow_id, settlement)?;
    store.persist()?;
    Ok(response)
}

pub(super) fn mark_ambiguous(
    store: &mut ServiceApiMessageStore,
    escrow_id: &str,
) -> Result<(), String> {
    store.refresh_from_disk()?;
    let intent = store
        .snapshot
        .settlement_intents
        .get_mut(escrow_id)
        .ok_or_else(|| "settlement intent missing during ambiguous outcome".to_owned())?;
    intent.state = "ambiguous".to_owned();
    record_submission(intent);
    intent.last_error_code = Some("SETTLEMENT_OUTCOME_AMBIGUOUS".to_owned());
    store.persist()
}

pub(super) fn mark_failed(
    store: &mut ServiceApiMessageStore,
    escrow_id: &str,
    error_code: &str,
) -> Result<(), String> {
    store.refresh_from_disk()?;
    let intent = store
        .snapshot
        .settlement_intents
        .get_mut(escrow_id)
        .ok_or_else(|| "settlement intent missing during failed outcome".to_owned())?;
    intent.state = "failed".to_owned();
    record_submission(intent);
    intent.last_error_code = Some(error_code.to_owned());
    store.persist()
}

pub(super) fn mark_submitted(
    store: &mut ServiceApiMessageStore,
    escrow_id: &str,
) -> Result<(), String> {
    store.refresh_from_disk()?;
    let intent = store
        .snapshot
        .settlement_intents
        .get_mut(escrow_id)
        .ok_or_else(|| "settlement intent missing during submit".to_owned())?;
    intent.state = "submitted".to_owned();
    record_submission(intent);
    store.persist()
}

fn record_submission(intent: &mut ServiceApiSettlementIntentRecord) {
    intent.submission_attempt_count = intent.submission_attempt_count.max(1);
}

pub(super) fn mark_expired(
    store: &mut ServiceApiMessageStore,
    escrow_id: &str,
) -> Result<(), String> {
    store.refresh_from_disk()?;
    let intent = store
        .snapshot
        .settlement_intents
        .get_mut(escrow_id)
        .ok_or_else(|| "settlement intent missing during expiration".to_owned())?;
    intent.state = "failed".to_owned();
    intent.last_error_code = Some("SETTLEMENT_TRANSACTION_EXPIRED".to_owned());
    store.persist()
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
    super::settlement::release_with_metadata(&mut store.snapshot, escrow_id, settlement)
}
