use super::*;
use crate::service_api_endpoint::message_store::store::task_escrow_ops::settlement;

pub(super) struct SettlementTerms {
    pub(super) task_id: String,
    pub(super) terms_digest: String,
    pub(super) asset: String,
}

pub(super) struct BuildRecordInput<'a> {
    pub(super) actor: &'a str,
    pub(super) escrow_id: &'a str,
    pub(super) key: &'a str,
    pub(super) expected_signature: &'a str,
    pub(super) signed_transaction_digest: &'a str,
    pub(super) signed_transaction_json: &'a str,
    pub(super) recipient_pubkey: &'a str,
    pub(super) amount_lamports: u64,
    pub(super) network: &'a str,
    pub(super) asset: &'a str,
    pub(super) terms_digest: &'a str,
    pub(super) task_id: &'a str,
    pub(super) bridge: Option<&'a BridgeSettlementIntentInput>,
}

pub(super) fn validate_agreement(
    store: &ServiceApiMessageStore,
    escrow_id: &str,
    prepared: &PreparedLiveSettlement,
) -> Result<SettlementTerms, String> {
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
    settlement_terms(escrow)
}

pub(super) fn validate_bridge_agreement(
    store: &ServiceApiMessageStore,
    escrow_id: &str,
    authority: &BridgeSettlementIntentInput,
) -> Result<(), String> {
    let escrow = store
        .snapshot
        .escrows
        .get(escrow_id)
        .ok_or_else(|| "settlement escrow missing".to_owned())?;
    let terms = settlement_terms(escrow)?;
    let valid = authority.task_id == terms.task_id
        && authority.terms_digest == terms.terms_digest
        && authority.asset == terms.asset
        && authority.amount_lamports == escrow.amount_lamports.unwrap_or_default()
        && authority.network == "solana:devnet";
    valid
        .then_some(())
        .ok_or_else(|| "BRIDGE_SETTLEMENT_AUTHORITY_MISMATCH".to_owned())
}

pub(super) fn identical_or_conflict(
    existing: &ServiceApiSettlementIntentRecord,
    actor: &str,
    key: &str,
    expected_signature: &str,
    signed_transaction_digest: &str,
    bridge: Option<&BridgeSettlementIntentInput>,
) -> Result<ServiceApiSettlementIntentRecord, String> {
    if existing.actor_did == actor
        && existing.idempotency_key == key
        && existing.expected_signature == expected_signature
        && existing.signed_transaction_digest == signed_transaction_digest
        && bridge_matches(existing, bridge)
    {
        return Ok(existing.clone());
    }
    Err("SETTLEMENT_INTENT_CONFLICT".to_owned())
}

pub(super) fn build_record(input: BuildRecordInput<'_>) -> ServiceApiSettlementIntentRecord {
    ServiceApiSettlementIntentRecord {
        settlement_intent_id: format!("settlement-intent-{}", input.escrow_id),
        escrow_id: input.escrow_id.to_owned(),
        task_id: input.task_id.to_owned(),
        actor_did: input.actor.to_owned(),
        idempotency_key: input.key.to_owned(),
        recipient_pubkey: input.recipient_pubkey.to_owned(),
        amount_lamports: input.amount_lamports,
        asset: input.asset.to_owned(),
        network: input.network.to_owned(),
        terms_digest: input.terms_digest.to_owned(),
        expected_signature: input.expected_signature.to_owned(),
        signed_transaction_digest: input.signed_transaction_digest.to_owned(),
        signed_transaction_json: input.signed_transaction_json.to_owned(),
        bridge_id: input.bridge.map(|value| value.bridge_id.clone()),
        bridge_receipt_id: input.bridge.map(|value| value.bridge_receipt_id.clone()),
        bridge_receipt_digest: input
            .bridge
            .map(|value| value.bridge_receipt_digest.clone()),
        bridge_transaction_signature: input
            .bridge
            .map(|value| value.bridge_transaction_signature.clone()),
        state: "prepared".to_owned(),
        submission_attempt_count: 0,
        last_error_code: None,
    }
}

pub(super) fn persist_record(
    store: &mut ServiceApiMessageStore,
    escrow_id: &str,
    record: ServiceApiSettlementIntentRecord,
) -> Result<ServiceApiSettlementIntentRecord, String> {
    store
        .snapshot
        .settlement_intents
        .insert(escrow_id.to_owned(), record.clone());
    store.persist()?;
    Ok(record)
}

pub(super) fn record_submission(intent: &mut ServiceApiSettlementIntentRecord) {
    if intent.bridge_receipt_id.is_none() {
        intent.submission_attempt_count = intent.submission_attempt_count.max(1);
    }
}

pub(super) fn release_without_refresh(
    store: &mut ServiceApiMessageStore,
    escrow_id: &str,
    settlement: &ServiceApiSettlementMetadata,
) -> Result<Option<ServiceApiEscrowStatusBody>, String> {
    settlement::release_with_metadata(&mut store.snapshot, escrow_id, settlement)
}

fn bridge_matches(
    existing: &ServiceApiSettlementIntentRecord,
    bridge: Option<&BridgeSettlementIntentInput>,
) -> bool {
    match bridge {
        Some(bridge) => {
            existing.bridge_id.as_deref() == Some(bridge.bridge_id.as_str())
                && existing.bridge_receipt_id.as_deref() == Some(bridge.bridge_receipt_id.as_str())
                && existing.bridge_receipt_digest.as_deref()
                    == Some(bridge.bridge_receipt_digest.as_str())
                && existing.bridge_transaction_signature.as_deref()
                    == Some(bridge.bridge_transaction_signature.as_str())
        }
        None => existing.bridge_id.is_none() && existing.bridge_receipt_id.is_none(),
    }
}

fn settlement_terms(escrow: &ServiceApiPersistedEscrowRecord) -> Result<SettlementTerms, String> {
    Ok(SettlementTerms {
        task_id: escrow
            .task_id
            .clone()
            .ok_or_else(|| "settlement task missing".to_owned())?,
        terms_digest: escrow
            .terms_digest
            .clone()
            .ok_or_else(|| "settlement terms missing".to_owned())?,
        asset: "lamports".to_owned(),
    })
}
