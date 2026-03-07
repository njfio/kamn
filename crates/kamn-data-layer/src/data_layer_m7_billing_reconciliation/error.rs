use std::fmt;

/// Error taxonomy for M7 billing reconciliation policy.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DataLayerM7BillingReconciliationError {
    /// Billing day bucket epoch is zero or not daily-aligned.
    InvalidBucketDayEpochSeconds(u64),
}

impl fmt::Display for DataLayerM7BillingReconciliationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidBucketDayEpochSeconds(value) => {
                write!(formatter, "invalid billing day bucket epoch: {value}")
            }
        }
    }
}

impl std::error::Error for DataLayerM7BillingReconciliationError {}
