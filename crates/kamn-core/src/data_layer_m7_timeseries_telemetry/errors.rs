use std::fmt;

/// Error taxonomy for M7 time-series contracts.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DataLayerM7TimeseriesError {
    EmptyField(&'static str),
    InvalidDid(String),
    OwnerNotFound { owner_did: String },
    OwnerScopeViolation { reason_code: &'static str },
    InvalidBucketDayEpochSeconds(u64),
    ObservabilitySampleInvalid { reason_code: &'static str },
}

impl fmt::Display for DataLayerM7TimeseriesError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyField(field) => write!(f, "{field} must not be empty"),
            Self::InvalidDid(value) => write!(f, "invalid did: {value}"),
            Self::OwnerNotFound { owner_did } => write!(f, "owner not found: {owner_did}"),
            Self::OwnerScopeViolation { reason_code } => {
                write!(f, "owner scope violation: {reason_code}")
            }
            Self::InvalidBucketDayEpochSeconds(value) => {
                write!(f, "invalid billing day bucket epoch: {value}")
            }
            Self::ObservabilitySampleInvalid { reason_code } => {
                write!(f, "invalid observability sample projection: {reason_code}")
            }
        }
    }
}

impl std::error::Error for DataLayerM7TimeseriesError {}
