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
        settlement: record.settlement.clone(),
    }
}

fn escrow_claim_scope(record: &ServiceApiPersistedEscrowRecord) -> &'static str {
    if record.settlement.settlement_tx_signature.is_some()
        && record.settlement.settlement_network.as_deref() == Some("solana:devnet")
    {
        return "devnet-backed";
    }
    "local-only"
}
