use super::agent_transaction_persisted_settlement::ExpectedSettlement;
use super::agent_transaction_receipt_chain_digest as digest;
use super::agent_transaction_receipt_chain_entries::{
    durable_receipts, mutation_entries, require_unique,
};
use super::agent_transaction_receipt_chain_model::*;
use super::agent_transaction_receipt_chain_settlement::settlement_entry;

const ERROR: &str = "SERVICE_RECEIPT_CHAIN_INVALID";
const SCHEMA: &str = "kamn.runtime.service-api-message-store.v4";

pub(super) fn recompute(
    raw: &str,
    expected: &ExpectedSettlement<'_>,
) -> Result<ReceiptChainEvidence, String> {
    let state: State = serde_json::from_str(raw).map_err(|_| invalid())?;
    if state.schema_version != SCHEMA {
        return Err(invalid());
    }
    let task = state.tasks.get(expected.task_id).ok_or_else(invalid)?;
    let escrow = state.escrows.get(expected.escrow_id).ok_or_else(invalid)?;
    validate_records(task, escrow, expected)?;
    let tasks = task_receipts(&state, expected.task_id)?;
    let escrows = escrow_receipts(&state, expected.escrow_id)?;
    let mut entries = mutation_entries(&state, task, escrow, &tasks, &escrows)?;
    require_unique(&entries)?;
    entries.push(settlement_entry(&state, escrow, expected)?);
    let receipts = durable_receipts(&entries);
    Ok(ReceiptChainEvidence {
        commitment: digest::chain(&entries),
        receipts,
    })
}

fn validate_records(
    task: &Task,
    escrow: &Escrow,
    expected: &ExpectedSettlement<'_>,
) -> Result<(), String> {
    let valid = task.task_id == expected.task_id
        && task.state == "completed"
        && task.transaction_id.as_deref() == Some(expected.transaction_id)
        && escrow.escrow_id == expected.escrow_id
        && escrow.state == "released"
        && escrow.task_id.as_deref() == Some(expected.task_id)
        && escrow.transaction_id.as_deref() == Some(expected.transaction_id);
    valid.then_some(()).ok_or_else(invalid)
}

fn task_receipts<'a>(state: &'a State, task_id: &str) -> Result<Vec<&'a TaskReceipt>, String> {
    let receipts = state
        .task_transition_receipts
        .iter()
        .filter(|receipt| receipt.task_id == task_id)
        .collect::<Vec<_>>();
    let phases = receipts
        .iter()
        .map(|receipt| {
            (
                receipt.action.as_str(),
                receipt.prior_state.as_str(),
                receipt.resulting_state.as_str(),
            )
        })
        .collect::<Vec<_>>();
    let expected = [
        ("task:create", "none", "submitted"),
        ("task:accept", "submitted", "accepted"),
        ("task:complete", "accepted", "completed"),
    ];
    (phases == expected).then_some(receipts).ok_or_else(invalid)
}

fn escrow_receipts<'a>(
    state: &'a State,
    escrow_id: &str,
) -> Result<Vec<&'a EscrowReceipt>, String> {
    let receipts = state
        .escrow_transition_receipts
        .iter()
        .filter(|receipt| receipt.escrow_id == escrow_id)
        .collect::<Vec<_>>();
    let phases = receipts
        .iter()
        .map(|receipt| {
            (
                receipt.action.as_str(),
                receipt.prior_state.as_str(),
                receipt.resulting_state.as_str(),
            )
        })
        .collect::<Vec<_>>();
    let expected = [
        ("escrow:fund", "unfunded", "funded"),
        ("escrow:release-authorize", "funded", "release-authorized"),
    ];
    (phases == expected).then_some(receipts).ok_or_else(invalid)
}

pub(super) fn invalid() -> String {
    ERROR.to_owned()
}
