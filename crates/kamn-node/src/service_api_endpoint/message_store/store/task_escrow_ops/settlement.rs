use super::*;

pub(super) fn next_escrow_id(store: &ServiceApiMessageStore, payload: &str) -> String {
    super::tasks::next_local_task_escrow_id("escrow-local", payload, |candidate| {
        store.snapshot.escrows.contains_key(candidate)
    })
}

pub(super) fn build_escrow_record(escrow_id: &str) -> ServiceApiPersistedEscrowRecord {
    ServiceApiPersistedEscrowRecord {
        escrow_id: escrow_id.to_owned(),
        state: "funded".to_owned(),
        settlement: ServiceApiSettlementMetadata::default(),
    }
}

pub(super) fn release_escrow_record(
    record: &mut ServiceApiPersistedEscrowRecord,
    settlement: Option<&ServiceApiSettlementMetadata>,
) {
    record.state = "released".to_owned();
    record.settlement = settlement.cloned().unwrap_or_default();
}

pub(super) fn escrow_status_response(record: &ServiceApiPersistedEscrowRecord) -> ServiceApiEscrowStatusBody {
    ServiceApiEscrowStatusBody {
        escrow_id: record.escrow_id.clone(),
        state: record.state.clone(),
        settlement: record.settlement.clone(),
    }
}
