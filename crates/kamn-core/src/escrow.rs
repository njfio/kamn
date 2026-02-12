//! Escrow lifecycle contracts for release, refund, dispute, and receipt-finality reconciliation.

use std::fmt;

/// Escrow lifecycle status projection.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EscrowStatus {
    /// Funds are locked and no settlement has happened yet.
    Funded,
    /// Partial release has occurred and a remainder is still locked.
    PartiallyReleased {
        /// Total amount released to the payee.
        released: u128,
        /// Remaining amount still held in escrow.
        remaining: u128,
    },
    /// Full amount has been released to the payee.
    Released,
    /// Remaining amount has been fully refunded to the payer.
    Refunded,
    /// Escrow is under dispute.
    Disputed,
    /// Dispute resolution split has been finalized.
    Resolved {
        /// Total amount released to the payee after resolution.
        released_total: u128,
        /// Total amount refunded to the payer after resolution.
        refunded_total: u128,
    },
}

/// Receipt finality classification used to decide settlement outcomes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EscrowReceiptFinality {
    /// Receipt finality is confirmed.
    Final,
    /// Receipt is observed but not yet final.
    Pending,
    /// Receipt finality indicates failure.
    Failed,
}

impl EscrowReceiptFinality {
    /// Parses receipt finality from a case-insensitive string.
    pub fn parse(value: &str) -> Result<Self, EscrowLifecycleError> {
        let normalized = value.trim().to_ascii_uppercase();
        match normalized.as_str() {
            "FINAL" => Ok(Self::Final),
            "PENDING" => Ok(Self::Pending),
            "FAILED" => Ok(Self::Failed),
            _ => Err(EscrowLifecycleError::InvalidReceiptFinality {
                found: value.to_owned(),
            }),
        }
    }
}

/// Settlement actions gated by receipt finality.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EscrowSettlementAction {
    /// Release a specific amount to the payee.
    Release {
        /// Amount to release.
        amount: u128,
    },
    /// Refund all remaining escrow balance.
    RefundRemaining,
    /// Refund after timeout threshold is reached.
    TimeoutRefund {
        /// Current Unix timestamp.
        current_unix: u64,
        /// Timeout Unix timestamp.
        timeout_unix: u64,
    },
}

/// State transition actions for evidence-driven escrow mutation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EscrowTransitionAction {
    /// Release a specific amount to the payee.
    Release {
        /// Amount to release.
        amount: u128,
    },
    /// Refund all remaining escrow balance.
    RefundRemaining,
    /// Move escrow into disputed state.
    Dispute,
    /// Resolve a dispute by splitting the remaining balance.
    Resolve {
        /// Amount released to the payee.
        release_to_payee: u128,
        /// Amount refunded to the payer.
        refund_to_payer: u128,
    },
}

/// Transition evidence record emitted for successful state mutations.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EscrowTransitionEvidence {
    /// Previous status.
    pub from: EscrowStatus,
    /// Action that was applied.
    pub action: EscrowTransitionAction,
    /// Resulting status.
    pub to: EscrowStatus,
    /// Stable reason code for policy/audit matching.
    pub reason_code: &'static str,
}

/// Settlement outcome projection after receipt-finality reconciliation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EscrowSettlementOutcome {
    /// Settlement succeeded and status is finalized.
    Settled {
        /// Finalized escrow status after successful settlement.
        status: EscrowStatus,
    },
    /// Settlement is deferred while receipt finality remains pending.
    Pending {
        /// Reason the settlement remains pending.
        reason: &'static str,
    },
    /// Settlement was rejected due to failed finality.
    Rejected {
        /// Reason settlement was rejected.
        reason: &'static str,
    },
}

impl EscrowSettlementOutcome {
    /// Returns a stable reason code for the projected settlement outcome.
    pub fn reason_code(&self) -> &'static str {
        match self {
            Self::Settled { .. } => "escrow_settlement_finalized",
            Self::Pending { .. } => "escrow_settlement_pending_receipt_finality",
            Self::Rejected { .. } => "escrow_settlement_rejected_receipt_finality",
        }
    }
}

/// Mutable escrow lifecycle state machine.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EscrowLifecycle {
    total_amount: u128,
    released_total: u128,
    refunded_total: u128,
    status: EscrowStatus,
}

impl EscrowLifecycle {
    /// Creates a new escrow lifecycle with a non-zero total amount.
    pub fn new(total_amount: u128) -> Result<Self, EscrowLifecycleError> {
        if total_amount == 0 {
            return Err(EscrowLifecycleError::ZeroAmount);
        }
        Ok(Self {
            total_amount,
            released_total: 0,
            refunded_total: 0,
            status: EscrowStatus::Funded,
        })
    }

    /// Returns the current escrow status.
    pub fn status(&self) -> EscrowStatus {
        self.status.clone()
    }

    /// Returns the currently unreleased and unrefunded amount.
    pub fn remaining_amount(&self) -> u128 {
        self.total_amount
            .saturating_sub(self.released_total)
            .saturating_sub(self.refunded_total)
    }

    /// Returns the total amount released to the payee.
    pub fn released_amount(&self) -> u128 {
        self.released_total
    }

    /// Returns the total amount refunded to the payer.
    pub fn refunded_amount(&self) -> u128 {
        self.refunded_total
    }

    /// Releases an amount from escrow to the payee.
    pub fn release(&mut self, amount: u128) -> Result<(), EscrowLifecycleError> {
        if amount == 0 {
            return Err(EscrowLifecycleError::ZeroAmount);
        }
        match self.status {
            EscrowStatus::Funded | EscrowStatus::PartiallyReleased { .. } => {}
            _ => return Err(self.invalid_transition("release")),
        }

        let remaining = self.remaining_amount();
        if amount > remaining {
            return Err(EscrowLifecycleError::InvalidAmount {
                action: "release",
                amount,
                remaining,
            });
        }

        self.released_total = self
            .released_total
            .checked_add(amount)
            .ok_or(EscrowLifecycleError::AmountOverflow)?;

        let next_remaining = self.remaining_amount();
        self.status = if next_remaining == 0 {
            EscrowStatus::Released
        } else {
            EscrowStatus::PartiallyReleased {
                released: self.released_total,
                remaining: next_remaining,
            }
        };
        Ok(())
    }

    /// Refunds the remaining escrow balance to the payer.
    pub fn refund_remaining(&mut self) -> Result<(), EscrowLifecycleError> {
        match self.status {
            EscrowStatus::Funded
            | EscrowStatus::PartiallyReleased { .. }
            | EscrowStatus::Disputed => {}
            _ => return Err(self.invalid_transition("refund_remaining")),
        }

        let remaining = self.remaining_amount();
        self.refunded_total = self
            .refunded_total
            .checked_add(remaining)
            .ok_or(EscrowLifecycleError::AmountOverflow)?;
        self.status = EscrowStatus::Refunded;
        Ok(())
    }

    /// Refunds remaining balance only if timeout has elapsed.
    pub fn refund_after_timeout(
        &mut self,
        current_unix: u64,
        timeout_unix: u64,
    ) -> Result<(), EscrowLifecycleError> {
        if current_unix < timeout_unix {
            return Err(EscrowLifecycleError::TimeoutNotElapsed {
                current_unix,
                timeout_unix,
            });
        }
        self.refund_remaining()
    }

    /// Moves escrow into disputed state.
    pub fn dispute(&mut self) -> Result<(), EscrowLifecycleError> {
        match self.status {
            EscrowStatus::Funded | EscrowStatus::PartiallyReleased { .. } => {
                self.status = EscrowStatus::Disputed;
                Ok(())
            }
            _ => Err(self.invalid_transition("dispute")),
        }
    }

    /// Resolves a disputed escrow by applying an exact split of remaining balance.
    pub fn resolve(
        &mut self,
        release_to_payee: u128,
        refund_to_payer: u128,
    ) -> Result<(), EscrowLifecycleError> {
        match self.status {
            EscrowStatus::Disputed => {}
            _ => return Err(self.invalid_transition("resolve")),
        }
        let split = release_to_payee
            .checked_add(refund_to_payer)
            .ok_or(EscrowLifecycleError::AmountOverflow)?;
        let remaining = self.remaining_amount();
        if split != remaining {
            return Err(EscrowLifecycleError::ResolutionMismatch {
                expected_remaining: remaining,
                actual_split: split,
            });
        }

        self.released_total = self
            .released_total
            .checked_add(release_to_payee)
            .ok_or(EscrowLifecycleError::AmountOverflow)?;
        self.refunded_total = self
            .refunded_total
            .checked_add(refund_to_payer)
            .ok_or(EscrowLifecycleError::AmountOverflow)?;
        self.status = EscrowStatus::Resolved {
            released_total: self.released_total,
            refunded_total: self.refunded_total,
        };
        Ok(())
    }

    /// Applies a transition action and returns evidence for the successful mutation.
    pub fn apply_transition_with_evidence(
        &mut self,
        action: EscrowTransitionAction,
    ) -> Result<EscrowTransitionEvidence, EscrowLifecycleError> {
        let from = self.status();
        match action.clone() {
            EscrowTransitionAction::Release { amount } => self.release(amount)?,
            EscrowTransitionAction::RefundRemaining => self.refund_remaining()?,
            EscrowTransitionAction::Dispute => self.dispute()?,
            EscrowTransitionAction::Resolve {
                release_to_payee,
                refund_to_payer,
            } => self.resolve(release_to_payee, refund_to_payer)?,
        }

        Ok(EscrowTransitionEvidence {
            from,
            action,
            to: self.status(),
            reason_code: "escrow_transition_allowed",
        })
    }

    /// Reconciles settlement action against receipt finality and returns outcome.
    pub fn reconcile_receipt_finality(
        &mut self,
        receipt_id: &str,
        finality: EscrowReceiptFinality,
        action: EscrowSettlementAction,
    ) -> Result<EscrowSettlementOutcome, EscrowLifecycleError> {
        if receipt_id.trim().is_empty() {
            return Err(EscrowLifecycleError::MissingReceiptEvidence);
        }

        match finality {
            EscrowReceiptFinality::Pending => Ok(EscrowSettlementOutcome::Pending {
                reason: "receipt finality pending",
            }),
            EscrowReceiptFinality::Failed => Ok(EscrowSettlementOutcome::Rejected {
                reason: "receipt finality failed",
            }),
            EscrowReceiptFinality::Final => {
                match action {
                    EscrowSettlementAction::Release { amount } => self.release(amount)?,
                    EscrowSettlementAction::RefundRemaining => self.refund_remaining()?,
                    EscrowSettlementAction::TimeoutRefund {
                        current_unix,
                        timeout_unix,
                    } => self.refund_after_timeout(current_unix, timeout_unix)?,
                }

                Ok(EscrowSettlementOutcome::Settled {
                    status: self.status(),
                })
            }
        }
    }

    fn invalid_transition(&self, action: &'static str) -> EscrowLifecycleError {
        EscrowLifecycleError::InvalidTransition {
            from: self.status(),
            action,
        }
    }
}

/// Error taxonomy for escrow lifecycle validation and transition failures.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EscrowLifecycleError {
    /// Amount is zero where positive value is required.
    ZeroAmount,
    /// Required receipt evidence is missing.
    MissingReceiptEvidence,
    /// Receipt finality string is invalid.
    InvalidReceiptFinality {
        /// Observed finality string.
        found: String,
    },
    /// Requested amount is invalid for current remaining balance.
    InvalidAmount {
        /// Action name that failed validation.
        action: &'static str,
        /// Requested amount.
        amount: u128,
        /// Remaining amount available for action.
        remaining: u128,
    },
    /// Requested transition is not allowed from current status.
    InvalidTransition {
        /// Current status.
        from: EscrowStatus,
        /// Action name.
        action: &'static str,
    },
    /// Dispute resolution split does not match remaining balance.
    ResolutionMismatch {
        /// Remaining amount expected to be split.
        expected_remaining: u128,
        /// Actual split amount provided.
        actual_split: u128,
    },
    /// Timeout refund attempted before timeout elapsed.
    TimeoutNotElapsed {
        /// Current Unix timestamp.
        current_unix: u64,
        /// Required timeout Unix timestamp.
        timeout_unix: u64,
    },
    /// Arithmetic overflow occurred while accumulating totals.
    AmountOverflow,
}

impl EscrowLifecycleError {
    /// Returns a stable reason code for telemetry and policy contracts.
    pub fn reason_code(&self) -> &'static str {
        match self {
            Self::ZeroAmount => "escrow_amount_zero",
            Self::MissingReceiptEvidence => "escrow_receipt_missing",
            Self::InvalidReceiptFinality { .. } => "escrow_receipt_finality_invalid",
            Self::InvalidAmount { .. } => "escrow_amount_invalid",
            Self::InvalidTransition { .. } => "escrow_transition_invalid",
            Self::ResolutionMismatch { .. } => "escrow_resolution_mismatch",
            Self::TimeoutNotElapsed { .. } => "escrow_timeout_not_elapsed",
            Self::AmountOverflow => "escrow_amount_overflow",
        }
    }
}

impl fmt::Display for EscrowLifecycleError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ZeroAmount => write!(f, "amount must be greater than zero"),
            Self::MissingReceiptEvidence => write!(f, "missing receipt evidence"),
            Self::InvalidReceiptFinality { found } => {
                write!(f, "invalid receipt finality state: {found}")
            }
            Self::InvalidAmount {
                action,
                amount,
                remaining,
            } => {
                write!(
                    f,
                    "invalid amount for {action}, amount {amount}, remaining {remaining}"
                )
            }
            Self::InvalidTransition { from, action } => {
                write!(
                    f,
                    "invalid escrow transition from {:?} with action {}",
                    from, action
                )
            }
            Self::ResolutionMismatch {
                expected_remaining,
                actual_split,
            } => write!(
                f,
                "resolution split must equal remaining, expected {expected_remaining}, got {actual_split}"
            ),
            Self::TimeoutNotElapsed {
                current_unix,
                timeout_unix,
            } => write!(
                f,
                "timeout not elapsed: current_unix {current_unix}, timeout_unix {timeout_unix}"
            ),
            Self::AmountOverflow => write!(f, "escrow amount overflow"),
        }
    }
}

impl std::error::Error for EscrowLifecycleError {}

#[cfg(test)]
mod tests {
    use super::{EscrowLifecycle, EscrowLifecycleError, EscrowStatus};

    #[test]
    fn release_rejects_amount_above_remaining() {
        let mut escrow = match EscrowLifecycle::new(10) {
            Ok(value) => value,
            Err(error) => panic!("init failed: {error}"),
        };
        assert_eq!(
            escrow.release(11),
            Err(EscrowLifecycleError::InvalidAmount {
                action: "release",
                amount: 11,
                remaining: 10,
            })
        );
    }

    #[test]
    fn dispute_then_refund_transitions_to_refunded() {
        let mut escrow = match EscrowLifecycle::new(10) {
            Ok(value) => value,
            Err(error) => panic!("init failed: {error}"),
        };
        if let Err(error) = escrow.dispute() {
            panic!("dispute failed: {error}");
        }
        if let Err(error) = escrow.refund_remaining() {
            panic!("refund failed: {error}");
        }
        assert_eq!(escrow.status(), EscrowStatus::Refunded);
        assert_eq!(escrow.refunded_amount(), 10);
    }

    #[test]
    fn regression_premature_timeout_refund_is_rejected() {
        // Regression: #542
        let mut escrow = EscrowLifecycle::new(50).expect("escrow should initialize");
        assert_eq!(
            escrow.refund_after_timeout(1_716_620_050, 1_716_620_100),
            Err(EscrowLifecycleError::TimeoutNotElapsed {
                current_unix: 1_716_620_050,
                timeout_unix: 1_716_620_100,
            })
        );
    }

    #[test]
    fn timeout_refund_at_deadline_refunds_remaining_balance() {
        let mut escrow = EscrowLifecycle::new(100).expect("escrow should initialize");
        escrow.release(35).expect("release should succeed");
        escrow
            .refund_after_timeout(1_716_620_100, 1_716_620_100)
            .expect("refund at timeout boundary should succeed");

        assert_eq!(escrow.status(), EscrowStatus::Refunded);
        assert_eq!(escrow.released_amount(), 35);
        assert_eq!(escrow.refunded_amount(), 65);
    }
}
