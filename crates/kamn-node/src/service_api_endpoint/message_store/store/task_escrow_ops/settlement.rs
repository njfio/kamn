use super::*;

pub(super) fn next_escrow_id(store: &ServiceApiMessageStore, payload: &str) -> String {
    super::tasks::next_local_task_escrow_id("escrow-local", payload, |candidate| {
        store.snapshot.escrows.contains_key(candidate)
    })
}

pub(crate) fn escrow_fund_task_id(payload: &str) -> Result<String, String> {
    let body = serde_json::from_str::<serde_json::Value>(payload)
        .map_err(|error| format!("escrow fund payload must be json: {error}"))?;
    body.get("task_id")
        .and_then(serde_json::Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_owned)
        .ok_or_else(|| "escrow fund task_id is required".to_owned())
}

pub(super) fn release_escrow_record(
    record: &mut ServiceApiPersistedEscrowRecord,
    settlement: Option<&ServiceApiSettlementMetadata>,
) {
    record.state = "released".to_owned();
    record.settlement = settlement.cloned().unwrap_or_default();
}

pub(super) fn escrow_status_response(
    record: &ServiceApiPersistedEscrowRecord,
) -> ServiceApiEscrowStatusBody {
    ServiceApiEscrowStatusBody {
        escrow_id: record.escrow_id.clone(),
        state: record.state.clone(),
        task_id: record.task_id.clone(),
        transaction_id: record.transaction_id.clone(),
        funder_did: record.funder_did.clone(),
        beneficiary_did: record.beneficiary_did.clone(),
        amount_lamports: record.amount_lamports,
        network: record.network.clone(),
        terms_digest: record.terms_digest.clone(),
        release_authority_did: record.release_authority_did.clone(),
        release_policy: record.release_policy.clone(),
        claim_scope: escrow_claim_scope(record).to_owned(),
        receipt_id: None,
        receipt_digest: None,
        action: None,
        settlement: record.settlement.clone(),
        ..ServiceApiEscrowStatusBody::default()
    }
}

pub(super) fn released_response(
    snapshot: &ServiceApiPersistedMessageStoreSnapshot,
    escrow_id: &str,
) -> Result<Option<ServiceApiEscrowStatusBody>, String> {
    let Some(record) = snapshot.escrows.get(escrow_id) else {
        return Ok(None);
    };
    if record.state != "released" {
        return Ok(None);
    }
    let receipt = release_authority_receipt(snapshot, escrow_id)?;
    let intent = confirmed_settlement_intent(snapshot, escrow_id)?;
    Ok(Some(settled_status_response(
        snapshot, record, receipt, intent,
    )?))
}

pub(super) fn release_authority_receipt<'a>(
    snapshot: &'a ServiceApiPersistedMessageStoreSnapshot,
    escrow_id: &str,
) -> Result<&'a ServiceApiEscrowTransitionReceiptRecord, String> {
    snapshot
        .escrow_transition_receipts
        .iter()
        .find(|receipt| {
            receipt.escrow_id == escrow_id && receipt.action == "escrow:release-authorize"
        })
        .ok_or_else(|| "ESCROW_RECEIPT_MISSING: release authorization receipt missing".to_owned())
}

pub(super) fn status_response_with_receipt(
    record: &ServiceApiPersistedEscrowRecord,
    receipt: &ServiceApiEscrowTransitionReceiptRecord,
) -> ServiceApiEscrowStatusBody {
    let mut response = escrow_status_response(record);
    response.receipt_id = Some(receipt.receipt_id.clone());
    response.receipt_digest = Some(authority_digest::escrow(receipt));
    response.action = Some(receipt.action.clone());
    response
}

pub(super) fn receipt_status_response(
    record: &ServiceApiPersistedEscrowRecord,
    receipt: &ServiceApiEscrowTransitionReceiptRecord,
) -> ServiceApiEscrowStatusBody {
    let mut response = status_response_with_receipt(record, receipt);
    response.state = receipt.resulting_state.clone();
    response
}

fn settled_status_response(
    snapshot: &ServiceApiPersistedMessageStoreSnapshot,
    record: &ServiceApiPersistedEscrowRecord,
    receipt: &ServiceApiEscrowTransitionReceiptRecord,
    intent: &ServiceApiSettlementIntentRecord,
) -> Result<ServiceApiEscrowStatusBody, String> {
    validate_settlement_receipt_binding(record, intent)?;
    let mut response = receipt_status_response(record, receipt);
    response.settlement_receipt_id = Some(intent.settlement_intent_id.clone());
    response.settlement_receipt_digest = Some(authority_digest::settlement(intent));
    response.settlement_receipt_action = Some("settlement:confirmed".to_owned());
    response.settlement_receipt_resource_id = Some(intent.escrow_id.clone());
    response.settlement_receipt_state = Some(intent.state.clone());
    response.authoritative_settlement =
        super::settlement_authority::build(snapshot, receipt, intent)?;
    Ok(response)
}

fn validate_settlement_receipt_binding(
    escrow: &ServiceApiPersistedEscrowRecord,
    intent: &ServiceApiSettlementIntentRecord,
) -> Result<(), String> {
    let valid = escrow.release_authority_did.as_deref() == Some(intent.actor_did.as_str())
        && escrow.settlement.settlement_tx_signature.as_deref()
            == Some(intent.expected_signature.as_str())
        && escrow.settlement.settlement_receipt_hash.as_deref()
            == Some(intent.expected_signature.as_str())
        && escrow.settlement.settlement_network.as_deref() == Some(intent.network.as_str())
        && escrow.settlement.settlement_commitment.as_deref() == Some("finalized")
        && escrow.amount_lamports == Some(intent.amount_lamports)
        && intent.network == "solana:devnet";
    let bridge_valid = match &intent.bridge_receipt_digest {
        Some(digest) => {
            escrow.settlement.bridge_receipt_digest.as_deref() == Some(digest.as_str())
                && escrow.settlement.bridge_transaction_signature.as_deref()
                    == intent.bridge_transaction_signature.as_deref()
                && escrow.settlement.bridge_receipt_id.as_deref()
                    == intent.bridge_receipt_id.as_deref()
                && escrow.settlement.bridge_id.as_deref() == intent.bridge_id.as_deref()
        }
        None => escrow.settlement.bridge_receipt_digest.is_none(),
    };
    (valid && bridge_valid)
        .then_some(())
        .ok_or_else(|| "SETTLEMENT_RECEIPT_INVALID: settlement receipt binding mismatch".to_owned())
}

fn confirmed_settlement_intent<'a>(
    snapshot: &'a ServiceApiPersistedMessageStoreSnapshot,
    escrow_id: &str,
) -> Result<&'a ServiceApiSettlementIntentRecord, String> {
    snapshot
        .settlement_intents
        .get(escrow_id)
        .filter(|intent| intent.state == "confirmed" && intent.escrow_id == escrow_id)
        .ok_or_else(|| {
            "SETTLEMENT_RECEIPT_MISSING: confirmed settlement receipt missing".to_owned()
        })
}

pub(super) fn release_with_metadata(
    snapshot: &mut ServiceApiPersistedMessageStoreSnapshot,
    escrow_id: &str,
    settlement: &ServiceApiSettlementMetadata,
) -> Result<Option<ServiceApiEscrowStatusBody>, String> {
    if !snapshot.escrows.contains_key(escrow_id) {
        return Ok(None);
    }
    let receipt = release_authority_receipt(snapshot, escrow_id)?.clone();
    let intent = confirmed_settlement_intent(snapshot, escrow_id)?.clone();
    let record = {
        let record = snapshot
            .escrows
            .get_mut(escrow_id)
            .ok_or_else(|| "settlement escrow missing during release".to_owned())?;
        release_escrow_record(record, Some(settlement));
        record.clone()
    };
    Ok(Some(settled_status_response(
        snapshot, &record, &receipt, &intent,
    )?))
}

fn escrow_claim_scope(record: &ServiceApiPersistedEscrowRecord) -> &'static str {
    if record.settlement.settlement_tx_signature.is_some()
        && record.settlement.settlement_network.as_deref() == Some("solana:devnet")
    {
        return "devnet-backed";
    }
    "local-only"
}
