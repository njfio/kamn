//! Runtime-commit lifecycle and finality policy contracts.

use crate::ReceiptFinality;

/// Finality classification for a runtime commit receipt.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KolmeCommitReceiptFinality {
    /// Commit has been submitted and is pending confirmation.
    Pending,
    /// Commit is fully finalized.
    Final,
    /// Commit failed validation/finality.
    Failed,
}

/// Runtime lifecycle state projected from commit receipt outcomes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RuntimeCommitLifecycleState {
    /// Commit is pending confirmation and should remain on requeue/watch.
    Pending,
    /// Commit has reached final confirmation.
    Finalized,
    /// Commit failed and should not be retried automatically.
    Failed,
}

/// Maps receipt finality to projected lifecycle state.
pub fn lifecycle_state_for_finality(
    finality: KolmeCommitReceiptFinality,
) -> RuntimeCommitLifecycleState {
    match finality {
        KolmeCommitReceiptFinality::Pending => RuntimeCommitLifecycleState::Pending,
        KolmeCommitReceiptFinality::Final => RuntimeCommitLifecycleState::Finalized,
        KolmeCommitReceiptFinality::Failed => RuntimeCommitLifecycleState::Failed,
    }
}

/// Maps receipt finality aliases to runtime commit finality.
pub fn commit_finality_from_receipt_finality(
    finality: ReceiptFinality,
) -> KolmeCommitReceiptFinality {
    match finality {
        ReceiptFinality::Pending => KolmeCommitReceiptFinality::Pending,
        ReceiptFinality::Final => KolmeCommitReceiptFinality::Final,
        ReceiptFinality::Failed => KolmeCommitReceiptFinality::Failed,
    }
}

/// Renders deterministic lifecycle state labels for diagnostics and errors.
pub fn lifecycle_state_label(state: RuntimeCommitLifecycleState) -> &'static str {
    match state {
        RuntimeCommitLifecycleState::Pending => "pending",
        RuntimeCommitLifecycleState::Finalized => "finalized",
        RuntimeCommitLifecycleState::Failed => "failed",
    }
}

/// Renders deterministic receipt finality labels for diagnostics and errors.
pub fn commit_finality_label(finality: KolmeCommitReceiptFinality) -> &'static str {
    match finality {
        KolmeCommitReceiptFinality::Pending => "pending",
        KolmeCommitReceiptFinality::Final => "final",
        KolmeCommitReceiptFinality::Failed => "failed",
    }
}

#[cfg(test)]
mod tests {
    use super::{
        commit_finality_from_receipt_finality, commit_finality_label, lifecycle_state_for_finality,
        lifecycle_state_label, KolmeCommitReceiptFinality, RuntimeCommitLifecycleState,
    };
    use crate::ReceiptFinality;

    #[test]
    fn unit_maps_finality_to_lifecycle_state() {
        assert_eq!(
            lifecycle_state_for_finality(KolmeCommitReceiptFinality::Pending),
            RuntimeCommitLifecycleState::Pending
        );
        assert_eq!(
            lifecycle_state_for_finality(KolmeCommitReceiptFinality::Final),
            RuntimeCommitLifecycleState::Finalized
        );
        assert_eq!(
            lifecycle_state_for_finality(KolmeCommitReceiptFinality::Failed),
            RuntimeCommitLifecycleState::Failed
        );
    }

    #[test]
    fn regression_labels_remain_stable() {
        // Regression: #1775
        assert_eq!(
            lifecycle_state_label(RuntimeCommitLifecycleState::Finalized),
            "finalized"
        );
        assert_eq!(
            commit_finality_label(KolmeCommitReceiptFinality::Final),
            "final"
        );
    }

    #[test]
    fn regression_receipt_finality_mapping_remains_stable() {
        // Regression: #1779
        assert_eq!(
            commit_finality_from_receipt_finality(ReceiptFinality::Pending),
            KolmeCommitReceiptFinality::Pending
        );
        assert_eq!(
            commit_finality_from_receipt_finality(ReceiptFinality::Final),
            KolmeCommitReceiptFinality::Final
        );
        assert_eq!(
            commit_finality_from_receipt_finality(ReceiptFinality::Failed),
            KolmeCommitReceiptFinality::Failed
        );
    }
}
