use super::super::super::*;
use crate::service_api_endpoint::live_settlement_dispatch::PreparedLiveSettlement;

mod build;

pub(crate) struct BridgeSettlementIntentInput {
    pub(crate) bridge_id: String,
    pub(crate) bridge_receipt_id: String,
    pub(crate) bridge_receipt_digest: String,
    pub(crate) bridge_transaction_signature: String,
    pub(crate) recipient_pubkey: String,
    pub(crate) amount_lamports: u64,
    pub(crate) network: String,
    pub(crate) asset: String,
    pub(crate) terms_digest: String,
    pub(crate) task_id: String,
}

pub(super) fn prepare(
    store: &mut ServiceApiMessageStore,
    actor: &str,
    escrow_id: &str,
    idempotency_key: &str,
    prepared: &PreparedLiveSettlement,
) -> Result<ServiceApiSettlementIntentRecord, String> {
    store.refresh_from_disk()?;
    let terms = build::validate_agreement(store, escrow_id, prepared)?;
    if let Some(existing) = store.snapshot.settlement_intents.get(escrow_id) {
        return build::identical_or_conflict(
            existing,
            actor,
            idempotency_key,
            prepared.expected_signature.as_str(),
            prepared.signed_transaction_digest.as_str(),
            None,
        );
    }
    if !settlement_signature_is_available(
        &store.snapshot,
        escrow_id,
        prepared.expected_signature.as_str(),
    ) {
        return Err("SETTLEMENT_SIGNATURE_REUSE".to_owned());
    }
    let record = build::build_record(build::BuildRecordInput {
        actor,
        escrow_id,
        key: idempotency_key,
        expected_signature: prepared.expected_signature.as_str(),
        signed_transaction_digest: prepared.signed_transaction_digest.as_str(),
        signed_transaction_json: prepared.signed_transaction_json.as_str(),
        recipient_pubkey: prepared.recipient_pubkey.as_str(),
        amount_lamports: prepared.amount_lamports,
        network: prepared.network.as_str(),
        asset: terms.asset.as_str(),
        terms_digest: terms.terms_digest.as_str(),
        task_id: terms.task_id.as_str(),
        bridge: None,
    });
    build::persist_record(store, escrow_id, record)
}

pub(super) fn prepare_bridge(
    store: &mut ServiceApiMessageStore,
    actor: &str,
    escrow_id: &str,
    idempotency_key: &str,
    authority: &BridgeSettlementIntentInput,
) -> Result<ServiceApiSettlementIntentRecord, String> {
    store.refresh_from_disk()?;
    build::validate_bridge_agreement(store, escrow_id, authority)?;
    if let Some(existing) = store.snapshot.settlement_intents.get(escrow_id) {
        return build::identical_or_conflict(
            existing,
            actor,
            idempotency_key,
            authority.bridge_transaction_signature.as_str(),
            authority.bridge_receipt_digest.as_str(),
            Some(authority),
        );
    }
    if !settlement_signature_is_available(
        &store.snapshot,
        escrow_id,
        authority.bridge_transaction_signature.as_str(),
    ) {
        return Err("BRIDGE_SETTLEMENT_RECEIPT_REPLAY".to_owned());
    }
    let record = build::build_record(build::BuildRecordInput {
        actor,
        escrow_id,
        key: idempotency_key,
        expected_signature: authority.bridge_transaction_signature.as_str(),
        signed_transaction_digest: authority.bridge_receipt_digest.as_str(),
        signed_transaction_json: "",
        recipient_pubkey: authority.recipient_pubkey.as_str(),
        amount_lamports: authority.amount_lamports,
        network: authority.network.as_str(),
        asset: authority.asset.as_str(),
        terms_digest: authority.terms_digest.as_str(),
        task_id: authority.task_id.as_str(),
        bridge: Some(authority),
    });
    build::persist_record(store, escrow_id, record)
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
    build::record_submission(intent);
    intent.last_error_code = None;
    let response = build::release_without_refresh(store, escrow_id, settlement)?;
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
    build::record_submission(intent);
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
    build::record_submission(intent);
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
    build::record_submission(intent);
    store.persist()
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
