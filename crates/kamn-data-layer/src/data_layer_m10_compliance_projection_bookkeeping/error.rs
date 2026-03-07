use std::fmt;

/// Error taxonomy for extracted M10 compliance-projection bookkeeping.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DataLayerM10ComplianceProjectionBookkeepingError {
    /// Required field was empty.
    EmptyField(&'static str),
    /// Partition month id failed `YYYYMM` validation.
    InvalidPartitionMonthId(u32),
    /// Owner-scope authorization denied.
    OwnerScopeViolation {
        /// Stable reason marker.
        reason_code: &'static str,
    },
    /// Projection lookup failed.
    PortLookupFailed {
        /// Stable reason marker.
        reason_code: &'static str,
        /// Stable detail string.
        detail: String,
    },
    /// Projection input was invalid.
    PortInvalidInput {
        /// Stable reason marker.
        reason_code: &'static str,
        /// Stable detail string.
        detail: String,
    },
    /// Target partition state was missing.
    PartitionNotFound(String),
    /// Registry mutation failed in an unexpected way.
    RegistryMutationFailed(String),
}

impl fmt::Display for DataLayerM10ComplianceProjectionBookkeepingError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyField(field) => write!(formatter, "field must not be empty: {field}"),
            Self::InvalidPartitionMonthId(value) => {
                write!(formatter, "invalid partition month id: {value}")
            }
            Self::OwnerScopeViolation { reason_code } => {
                write!(formatter, "owner scope violation: {reason_code}")
            }
            Self::PortLookupFailed {
                reason_code,
                detail,
            } => write!(formatter, "lookup failed: {reason_code} ({detail})"),
            Self::PortInvalidInput {
                reason_code,
                detail,
            } => write!(formatter, "invalid input: {reason_code} ({detail})"),
            Self::PartitionNotFound(name) => write!(formatter, "partition not found: {name}"),
            Self::RegistryMutationFailed(detail) => {
                write!(formatter, "registry mutation failed: {detail}")
            }
        }
    }
}

impl std::error::Error for DataLayerM10ComplianceProjectionBookkeepingError {}
