//! Finality contracts for Kolme runtime-commit settlement.

/// Finality status for a runtime-commit submission.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FinalityState {
    /// Transaction has not yet reached the configured confirmation threshold.
    Pending,
    /// Transaction reached the configured confirmation threshold.
    Confirmed,
    /// Transaction is explicitly rejected.
    Rejected,
}

/// Deterministic finality resolution snapshot.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FinalityResolution {
    state: FinalityState,
    confirmations: u64,
    threshold: u64,
}

impl FinalityResolution {
    /// Creates a new finality resolution.
    pub fn new(state: FinalityState, confirmations: u64, threshold: u64) -> Self {
        Self {
            state,
            confirmations,
            threshold: threshold.max(1),
        }
    }

    /// Returns the resolved finality state.
    pub fn state(&self) -> FinalityState {
        self.state
    }

    /// Returns observed confirmation count.
    pub fn confirmations(&self) -> u64 {
        self.confirmations
    }

    /// Returns configured confirmation threshold.
    pub fn threshold(&self) -> u64 {
        self.threshold
    }

    /// True when the state reached a terminal outcome.
    pub fn is_final(&self) -> bool {
        matches!(
            self.state,
            FinalityState::Confirmed | FinalityState::Rejected
        )
    }
}

/// Resolves finality from confirmations, threshold, and explicit rejection.
pub fn resolve_finality(confirmations: u64, threshold: u64, rejected: bool) -> FinalityResolution {
    let normalized_threshold = threshold.max(1);
    let state = if rejected {
        FinalityState::Rejected
    } else if confirmations >= normalized_threshold {
        FinalityState::Confirmed
    } else {
        FinalityState::Pending
    };

    FinalityResolution::new(state, confirmations, normalized_threshold)
}

#[cfg(test)]
mod tests {
    use super::{resolve_finality, FinalityState};

    #[test]
    fn unit_resolve_finality_marks_pending_below_threshold() {
        let resolution = resolve_finality(1, 2, false);
        assert_eq!(resolution.state(), FinalityState::Pending);
        assert!(!resolution.is_final());
    }

    #[test]
    fn unit_resolve_finality_normalizes_zero_threshold() {
        let resolution = resolve_finality(1, 0, false);
        assert_eq!(resolution.threshold(), 1);
        assert_eq!(resolution.state(), FinalityState::Confirmed);
        assert!(resolution.is_final());
    }
}
