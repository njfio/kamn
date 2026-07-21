use super::*;
use std::collections::BTreeSet;

const TASK_PHASES: &[(&str, &str, &str)] = &[
    ("task:create", "none", "created"),
    ("task:accept", "created", "accepted"),
    ("task:complete", "accepted", "completed"),
];
const ESCROW_PHASES: &[(&str, &str, &str)] = &[
    ("escrow:fund", "unfunded", "funded"),
    ("escrow:release-authorize", "funded", "release-authorized"),
];

pub(super) fn task_phases(
    task: &ServiceApiPersistedTaskRecord,
    receipts: &[&ServiceApiTaskTransitionReceiptRecord],
) -> Result<(), TaskProjectionError> {
    let expected = if task.state == "completed" {
        TASK_PHASES
    } else {
        &TASK_PHASES[..2]
    };
    validate(receipts, expected)
}

pub(super) fn escrow_phases(
    escrow: &ServiceApiPersistedEscrowRecord,
    receipts: &[&ServiceApiEscrowTransitionReceiptRecord],
) -> Result<(), TaskProjectionError> {
    let expected = if matches!(escrow.state.as_str(), "release-authorized" | "released") {
        ESCROW_PHASES
    } else {
        &ESCROW_PHASES[..1]
    };
    validate(receipts, expected)
}

trait PhaseReceipt {
    fn phase(&self) -> (&str, &str, &str);
}

impl PhaseReceipt for &ServiceApiTaskTransitionReceiptRecord {
    fn phase(&self) -> (&str, &str, &str) {
        (&self.action, &self.prior_state, &self.resulting_state)
    }
}

impl PhaseReceipt for &ServiceApiEscrowTransitionReceiptRecord {
    fn phase(&self) -> (&str, &str, &str) {
        (&self.action, &self.prior_state, &self.resulting_state)
    }
}

fn validate<T: PhaseReceipt>(
    receipts: &[T],
    expected: &[(&str, &str, &str)],
) -> Result<(), TaskProjectionError> {
    let actual: Vec<_> = receipts.iter().map(PhaseReceipt::phase).collect();
    (actual == expected)
        .then_some(())
        .ok_or(TaskProjectionError::ReceiptChainInvalid)
}

pub(super) fn unique_fields(entries: &[ReceiptChainEntry]) -> Result<(), TaskProjectionError> {
    let receipt_ids: BTreeSet<_> = entries.iter().map(|entry| &entry.receipt_id).collect();
    let actor_keys: BTreeSet<_> = entries
        .iter()
        .map(|entry| (&entry.actor_did, &entry.idempotency_key))
        .collect();
    if receipt_ids.len() == entries.len() && actor_keys.len() == entries.len() {
        return Ok(());
    }
    Err(TaskProjectionError::ReceiptChainInvalid)
}
