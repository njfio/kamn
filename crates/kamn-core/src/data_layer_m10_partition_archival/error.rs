use std::fmt;

use crate::DataLayerM8ComplianceError;

use super::*;

/// Error taxonomy for M10 partition lifecycle contracts.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DataLayerM10PartitionLifecycleError {
    /// Required field was empty.
    EmptyField(&'static str),
    /// Partition month id failed `YYYYMM` validation.
    InvalidPartitionMonthId(u32),
    /// Duplicate partition month id registration.
    DuplicatePartitionMonthId(u32),
    /// Named partition does not exist in registry.
    PartitionNotFound(String),
    /// Owner-scope projection request was denied.
    OwnerScopeViolation {
        /// Stable reason marker.
        reason_code: &'static str,
    },
    /// Compliance projection failed before partition update could be applied.
    ComplianceProjectionFailed {
        /// Stable reason marker.
        reason_code: &'static str,
        /// Stable detail from compliance lookup/projection step.
        detail: String,
    },
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
    /// Archival retry policy configuration is invalid.
    InvalidRetryPolicy {
        /// Invalid field.
        field: &'static str,
        /// Stable reason marker.
        reason_code: &'static str,
    },
    /// Current retry attempt metadata is invalid.
    InvalidRetryAttempt {
        /// Invalid field.
        field: &'static str,
        /// Invalid value.
        value: u8,
        /// Stable reason marker.
        reason_code: &'static str,
    },
    /// Phase-6 orchestration failed before completing execution.
    Phase6ExecutionFailed {
        /// Stable reason marker.
        reason_code: &'static str,
        /// Stable detail marker for diagnostics.
        detail: String,
    },
    /// Phase-6 execution budget configuration is invalid.
    InvalidPhase6ExecutionBudget {
        /// Invalid field.
        field: &'static str,
        /// Stable reason marker.
        reason_code: &'static str,
    },
    /// Phase-6 scheduler policy configuration is invalid.
    InvalidPhase6SchedulerPolicy {
        /// Invalid field.
        field: &'static str,
        /// Stable reason marker.
        reason_code: &'static str,
    },
    /// Phase-6 scheduler signal metadata is invalid.
    InvalidPhase6SchedulerSignal {
        /// Invalid field.
        field: &'static str,
        /// Stable reason marker.
        reason_code: &'static str,
    },
    /// Phase-6 scheduler preflight budget check exceeded limits and failed closed.
    Phase6SchedulerBudgetPreflightExceeded {
        /// Stable reason marker describing exceeded dimension.
        reason_code: &'static str,
        /// Stable detail marker for diagnostics.
        detail: String,
    },
    /// Phase-6 runtime evidence input payload is invalid.
    InvalidPhase6RuntimeEvidenceInput {
        /// Invalid field.
        field: &'static str,
        /// Stable reason marker.
        reason_code: &'static str,
    },
}

impl fmt::Display for DataLayerM10PartitionLifecycleError {
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
            Self::OwnerScopeViolation { reason_code } => {
                write!(f, "owner scope violation: {reason_code}")
            }
            Self::ComplianceProjectionFailed {
                reason_code,
                detail,
            } => {
                write!(f, "compliance projection failed: {reason_code} ({detail})")
            }
            Self::InvalidLifecycleTransition {
                partition_name,
                from_status,
                to_status,
                reason_code,
            } => write!(
                f,
                "invalid lifecycle transition for {partition_name}: {from_status:?} -> {to_status:?} ({reason_code})"
            ),
            Self::InvalidRetryPolicy { field, reason_code } => {
                write!(f, "invalid archival retry policy field {field} ({reason_code})")
            }
            Self::InvalidRetryAttempt {
                field,
                value,
                reason_code,
            } => write!(
                f,
                "invalid archival retry attempt for {field}: {value} ({reason_code})"
            ),
            Self::Phase6ExecutionFailed {
                reason_code,
                detail,
            } => {
                write!(f, "phase6 execution failed: {reason_code} ({detail})")
            }
            Self::InvalidPhase6ExecutionBudget { field, reason_code } => write!(
                f,
                "invalid phase6 execution budget field {field} ({reason_code})"
            ),
            Self::InvalidPhase6SchedulerPolicy { field, reason_code } => write!(
                f,
                "invalid phase6 scheduler policy field {field} ({reason_code})"
            ),
            Self::InvalidPhase6SchedulerSignal { field, reason_code } => write!(
                f,
                "invalid phase6 scheduler signal field {field} ({reason_code})"
            ),
            Self::Phase6SchedulerBudgetPreflightExceeded { reason_code, detail } => {
                write!(
                    f,
                    "phase6 scheduler budget preflight exceeded: {reason_code} ({detail})"
                )
            }
            Self::InvalidPhase6RuntimeEvidenceInput { field, reason_code } => write!(
                f,
                "invalid phase6 runtime evidence input field {field} ({reason_code})"
            ),
        }
    }
}

impl std::error::Error for DataLayerM10PartitionLifecycleError {}

pub(super) fn map_m8_projection_error_to_m10(
    error: DataLayerM8ComplianceError,
) -> DataLayerM10PartitionLifecycleError {
    let reason_code = match error {
        DataLayerM8ComplianceError::OwnerScopeViolation { .. } => {
            DATA_LAYER_M10_COMPLIANCE_OWNER_SCOPE_DENIED_REASON_CODE
        }
        DataLayerM8ComplianceError::OwnerNotFound { .. }
        | DataLayerM8ComplianceError::MessageNotFound { .. } => {
            DATA_LAYER_M10_COMPLIANCE_LOOKUP_FAILED_REASON_CODE
        }
        DataLayerM8ComplianceError::InvalidDid(_)
        | DataLayerM8ComplianceError::EmptyField(_)
        | DataLayerM8ComplianceError::EmptyWrappedKeys
        | DataLayerM8ComplianceError::InvalidWrappedKey(_)
        | DataLayerM8ComplianceError::DuplicateWrappedKeyRecipient { .. }
        | DataLayerM8ComplianceError::DuplicateMessageId { .. }
        | DataLayerM8ComplianceError::LegalHoldActive { .. }
        | DataLayerM8ComplianceError::AlreadyShredded { .. } => {
            DATA_LAYER_M10_COMPLIANCE_INPUT_INVALID_REASON_CODE
        }
    };

    DataLayerM10PartitionLifecycleError::ComplianceProjectionFailed {
        reason_code,
        detail: error.to_string(),
    }
}

pub(super) fn map_m8_execution_error_to_m10(
    error: DataLayerM8ComplianceError,
) -> DataLayerM10PartitionLifecycleError {
    let reason_code = match error {
        DataLayerM8ComplianceError::OwnerScopeViolation { .. } => {
            DATA_LAYER_M10_PHASE6_EXECUTION_OWNER_SCOPE_DENIED_REASON_CODE
        }
        DataLayerM8ComplianceError::LegalHoldActive { .. } => {
            DATA_LAYER_M10_PHASE6_EXECUTION_LEGAL_HOLD_ACTIVE_REASON_CODE
        }
        DataLayerM8ComplianceError::OwnerNotFound { .. }
        | DataLayerM8ComplianceError::MessageNotFound { .. }
        | DataLayerM8ComplianceError::InvalidDid(_)
        | DataLayerM8ComplianceError::EmptyField(_)
        | DataLayerM8ComplianceError::EmptyWrappedKeys
        | DataLayerM8ComplianceError::InvalidWrappedKey(_)
        | DataLayerM8ComplianceError::DuplicateWrappedKeyRecipient { .. }
        | DataLayerM8ComplianceError::DuplicateMessageId { .. }
        | DataLayerM8ComplianceError::AlreadyShredded { .. } => {
            DATA_LAYER_M10_PHASE6_EXECUTION_INPUT_INVALID_REASON_CODE
        }
    };
    phase6_execution_failed(reason_code, error.to_string())
}

pub(super) fn map_phase6_projection_error_to_m10(
    error: DataLayerM10PartitionLifecycleError,
) -> DataLayerM10PartitionLifecycleError {
    let reason_code = match &error {
        DataLayerM10PartitionLifecycleError::OwnerScopeViolation { .. } => {
            DATA_LAYER_M10_PHASE6_EXECUTION_OWNER_SCOPE_DENIED_REASON_CODE
        }
        DataLayerM10PartitionLifecycleError::EmptyField(field)
            if *field == "partition_message_ids" || *field == "object_storage_prefix" =>
        {
            DATA_LAYER_M10_PHASE6_EXECUTION_PROJECTION_INPUT_INVALID_REASON_CODE
        }
        _ => DATA_LAYER_M10_PHASE6_EXECUTION_PROJECTION_FAILED_REASON_CODE,
    };
    phase6_execution_failed(reason_code, error.to_string())
}

pub(super) fn map_phase6_owner_scope_error_to_m10(
    error: DataLayerM10PartitionLifecycleError,
) -> DataLayerM10PartitionLifecycleError {
    match error {
        DataLayerM10PartitionLifecycleError::OwnerScopeViolation { .. } => phase6_execution_failed(
            DATA_LAYER_M10_PHASE6_EXECUTION_OWNER_SCOPE_DENIED_REASON_CODE,
            "phase6 owner scope authorization failed",
        ),
        other => other,
    }
}

pub(super) fn phase6_execution_failed(
    reason_code: &'static str,
    detail: impl Into<String>,
) -> DataLayerM10PartitionLifecycleError {
    DataLayerM10PartitionLifecycleError::Phase6ExecutionFailed {
        reason_code,
        detail: detail.into(),
    }
}
