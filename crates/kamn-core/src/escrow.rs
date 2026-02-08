use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EscrowStatus {
    Funded,
    PartiallyReleased {
        released: u128,
        remaining: u128,
    },
    Released,
    Refunded,
    Disputed,
    Resolved {
        released_total: u128,
        refunded_total: u128,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EscrowLifecycle {
    total_amount: u128,
    released_total: u128,
    refunded_total: u128,
    status: EscrowStatus,
}

impl EscrowLifecycle {
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

    pub fn status(&self) -> EscrowStatus {
        self.status.clone()
    }

    pub fn remaining_amount(&self) -> u128 {
        self.total_amount
            .saturating_sub(self.released_total)
            .saturating_sub(self.refunded_total)
    }

    pub fn released_amount(&self) -> u128 {
        self.released_total
    }

    pub fn refunded_amount(&self) -> u128 {
        self.refunded_total
    }

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

    pub fn dispute(&mut self) -> Result<(), EscrowLifecycleError> {
        match self.status {
            EscrowStatus::Funded | EscrowStatus::PartiallyReleased { .. } => {
                self.status = EscrowStatus::Disputed;
                Ok(())
            }
            _ => Err(self.invalid_transition("dispute")),
        }
    }

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

    fn invalid_transition(&self, action: &'static str) -> EscrowLifecycleError {
        EscrowLifecycleError::InvalidTransition {
            from: self.status(),
            action,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EscrowLifecycleError {
    ZeroAmount,
    InvalidAmount {
        action: &'static str,
        amount: u128,
        remaining: u128,
    },
    InvalidTransition {
        from: EscrowStatus,
        action: &'static str,
    },
    ResolutionMismatch {
        expected_remaining: u128,
        actual_split: u128,
    },
    AmountOverflow,
}

impl fmt::Display for EscrowLifecycleError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ZeroAmount => write!(f, "amount must be greater than zero"),
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
}
