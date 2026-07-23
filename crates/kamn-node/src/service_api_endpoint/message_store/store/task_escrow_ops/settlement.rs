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
    }
}

pub(super) fn status_response(
    snapshot: &ServiceApiPersistedMessageStoreSnapshot,
    escrow_id: &str,
) -> Result<Option<ServiceApiEscrowStatusBody>, String> {
    let Some(record) = snapshot.escrows.get(escrow_id) else {
        return Ok(None);
    };
    if record.state != "released" {
        return Ok(Some(escrow_status_response(record)));
    }
    let receipt = release_authority_receipt(snapshot, escrow_id)?;
    Ok(Some(released_status_response(record, receipt)))
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

pub(super) fn released_status_response(
    record: &ServiceApiPersistedEscrowRecord,
    receipt: &ServiceApiEscrowTransitionReceiptRecord,
) -> ServiceApiEscrowStatusBody {
    let mut response = escrow_status_response(record);
    response.receipt_id = Some(receipt.receipt_id.clone());
    response.receipt_digest = Some(authority_digest::escrow(receipt));
    response.action = Some(receipt.action.clone());
    response
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
    let record = snapshot
        .escrows
        .get_mut(escrow_id)
        .ok_or_else(|| "settlement escrow missing during release".to_owned())?;
    release_escrow_record(record, Some(settlement));
    Ok(Some(released_status_response(record, &receipt)))
}

fn escrow_claim_scope(record: &ServiceApiPersistedEscrowRecord) -> &'static str {
    if record.settlement.settlement_tx_signature.is_some()
        && record.settlement.settlement_network.as_deref() == Some("solana:devnet")
    {
        return "devnet-backed";
    }
    "local-only"
}
