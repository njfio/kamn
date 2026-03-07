use std::fmt;

use super::DataLayerM10PartitionStatus;

/// Error taxonomy for deterministic M10 partition registry lifecycle behavior.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DataLayerM10PartitionRegistryStateMachineError {
    /// Required field was empty.
    EmptyField(&'static str),
    /// Partition month id failed `YYYYMM` validation.
    InvalidPartitionMonthId(u32),
    /// Duplicate partition month id registration.
    DuplicatePartitionMonthId(u32),
    /// Named partition does not exist in registry.
    PartitionNotFound(String),
    /// Lifecycle transition was not allowed from current state.
    InvalidLifecycleTransition {
        /// Partition name.
        partition_name: String,
        /// Current lifecycle status.
        from_status: DataLayerM10PartitionStatus,
        /// Requested lifecycle status.
        to_status: DataLayerM10PartitionStatus,
        /// Stable reason marker.
        reason_code: &'static str,
    },
}

impl fmt::Display for DataLayerM10PartitionRegistryStateMachineError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyField(field) => write!(f, "field must not be empty: {field}"),
            Self::InvalidPartitionMonthId(value) => {
                write!(f, "invalid partition month id: {value}")
            }
            Self::DuplicatePartitionMonthId(value) => {
                write!(f, "duplicate partition month id: {value}")
            }
            Self::PartitionNotFound(value) => write!(f, "partition not found: {value}"),
            Self::InvalidLifecycleTransition {
                partition_name,
                from_status,
                to_status,
                reason_code,
            } => write!(
                f,
                "invalid lifecycle transition for {partition_name}: {from_status:?} -> {to_status:?} ({reason_code})"
            ),
        }
    }
}

impl std::error::Error for DataLayerM10PartitionRegistryStateMachineError {}
