use std::error::Error;
use std::fmt::{Display, Formatter};

#[derive(Debug, Clone, PartialEq, Eq)]
/// Construct lock error.
pub enum ConstructLockError {
    /// Invalid lease ttl.
    InvalidLeaseTtl,
    /// Invalid owner id.
    InvalidOwnerId,
    /// No active lease.
    NoActiveLease,
    /// No lease for execution.
    NoLeaseForExecution,
    /// Lease already held.
    LeaseAlreadyHeld {
        /// Current lock owner id.
        owner: String,
    },
    /// Lease owner mismatch.
    LeaseOwnerMismatch {
        /// Expected owner id.
        expected: String,
        /// Observed owner id.
        found: String,
    },
    /// Stale fencing token.
    StaleFencingToken {
        /// Expected fencing token.
        expected: u64,
        /// Observed fencing token.
        found: u64,
    },
}

impl Display for ConstructLockError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidLeaseTtl => write!(f, "construct lock lease ttl must be positive"),
            Self::InvalidOwnerId => write!(f, "construct lock owner id cannot be empty"),
            Self::NoActiveLease => write!(f, "construct lock has no active lease"),
            Self::NoLeaseForExecution => {
                write!(f, "daemon execution requires an active construct lock lease")
            }
            Self::LeaseAlreadyHeld { owner } => {
                write!(f, "construct lock lease already held by {owner}")
            }
            Self::LeaseOwnerMismatch { expected, found } => {
                write!(f, "construct lock owner mismatch: expected {expected}, found {found}")
            }
            Self::StaleFencingToken { expected, found } => {
                write!(f, "construct lock stale fencing token: expected {expected}, found {found}")
            }
        }
    }
}

impl Error for ConstructLockError {}
