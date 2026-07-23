use super::errors::{settlement_outcome_ambiguous_error, settlement_transaction_expired_error};
use super::*;

pub(super) fn handle(
    store: &mut ServiceApiMessageStore,
    escrow_id: &str,
    result: Result<LiveSettlementEvidence, String>,
) -> Result<LiveSettlementEvidence, Box<Response>> {
    match result {
        Ok(evidence) => Ok(evidence),
        Err(error) if error.starts_with("SETTLEMENT_SUBMISSION_PERSISTENCE_FAILED") => {
            Err(persistence_error(error))
        }
        Err(error) if error.starts_with("SETTLEMENT_OUTCOME_AMBIGUOUS") => {
            persist_ambiguous(store, escrow_id)?;
            Err(Box::new(settlement_outcome_ambiguous_error()))
        }
        Err(error) if error == "SETTLEMENT_TRANSACTION_EXPIRED" => {
            persist_expired(store, escrow_id)?;
            Err(Box::new(settlement_transaction_expired_error()))
        }
        Err(error) => Err(Box::new(live_settlement_evidence_error(error.as_str()))),
    }
}

fn persist_ambiguous(
    store: &mut ServiceApiMessageStore,
    escrow_id: &str,
) -> Result<(), Box<Response>> {
    store
        .mark_settlement_outcome_ambiguous(escrow_id)
        .map_err(persistence_error)
}

fn persist_expired(
    store: &mut ServiceApiMessageStore,
    escrow_id: &str,
) -> Result<(), Box<Response>> {
    store
        .mark_settlement_expired(escrow_id)
        .map_err(persistence_error)
}
