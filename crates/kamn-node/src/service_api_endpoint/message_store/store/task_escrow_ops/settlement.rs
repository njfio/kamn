use super::*;

pub(super) fn next_escrow_id(store: &ServiceApiMessageStore, payload: &str) -> String {
    super::next_local_task_escrow_id("escrow-local", payload, |candidate| {
        store.snapshot.escrows.contains_key(candidate)
    })
}

pub(super) fn build_escrow_record(escrow_id: &str) -> ServiceApiPersistedEscrowRecord {
    ServiceApiPersistedEscrowRecord {
        escrow_id: escrow_id.to_owned(),
        state: "funded".to_owned(),
        settlement_receipt_hash: None,
    }
}

pub(super) fn release_escrow_record(
    record: &mut ServiceApiPersistedEscrowRecord,
    settlement_receipt_hash: Option<&str>,
) {
    record.state = "released".to_owned();
    record.settlement_receipt_hash = settlement_receipt_hash.map(str::to_owned);
}

pub(super) fn released_escrow_response(
    escrow_id: &str,
    settlement_receipt_hash: Option<&str>,
) -> ServiceApiEscrowStatusBody {
    ServiceApiEscrowStatusBody {
        escrow_id: escrow_id.to_owned(),
        state: "released".to_owned(),
        settlement_receipt_hash: settlement_receipt_hash.map(str::to_owned),
    }
}
