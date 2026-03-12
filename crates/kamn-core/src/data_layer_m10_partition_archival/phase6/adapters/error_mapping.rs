use kamn_data_layer::{
    DataLayerM10ComplianceProjectionPortError, DataLayerM10Phase6CompliancePortError,
};

use crate::{DataLayerM8ComplianceError, data_layer_m10_partition_archival::error::phase6_execution_failed};
use crate::data_layer_m10_partition_archival::{
    DataLayerM10PartitionLifecycleError,
    DATA_LAYER_M10_PHASE6_EXECUTION_INPUT_INVALID_REASON_CODE,
    DATA_LAYER_M10_PHASE6_EXECUTION_OWNER_SCOPE_DENIED_REASON_CODE,
};

pub(crate) fn map_phase6_port_error_to_projection_port_error(
    error: DataLayerM10Phase6CompliancePortError,
) -> DataLayerM10ComplianceProjectionPortError {
    match error {
        DataLayerM10Phase6CompliancePortError::OwnerScopeViolation => {
            DataLayerM10ComplianceProjectionPortError::OwnerScopeViolation
        }
        DataLayerM10Phase6CompliancePortError::LookupFailed(detail) => {
            DataLayerM10ComplianceProjectionPortError::LookupFailed(detail)
        }
        DataLayerM10Phase6CompliancePortError::InvalidInput(detail) => {
            DataLayerM10ComplianceProjectionPortError::InvalidInput(detail)
        }
    }
}

pub(crate) fn map_m8_execution_error_to_phase6_port(
    error: DataLayerM8ComplianceError,
) -> DataLayerM10Phase6CompliancePortError {
    match error {
        DataLayerM8ComplianceError::OwnerScopeViolation { .. } => {
            DataLayerM10Phase6CompliancePortError::OwnerScopeViolation
        }
        DataLayerM8ComplianceError::OwnerNotFound { .. }
        | DataLayerM8ComplianceError::MessageNotFound { .. } => {
            DataLayerM10Phase6CompliancePortError::LookupFailed(error.to_string())
        }
        DataLayerM8ComplianceError::InvalidDid(_)
        | DataLayerM8ComplianceError::EmptyField(_)
        | DataLayerM8ComplianceError::EmptyWrappedKeys
        | DataLayerM8ComplianceError::InvalidWrappedKey(_)
        | DataLayerM8ComplianceError::DuplicateWrappedKeyRecipient { .. }
        | DataLayerM8ComplianceError::DuplicateMessageId { .. }
        | DataLayerM8ComplianceError::LegalHoldActive { .. }
        | DataLayerM8ComplianceError::AlreadyShredded { .. } => {
            DataLayerM10Phase6CompliancePortError::InvalidInput(error.to_string())
        }
    }
}

pub(crate) fn map_phase6_owner_scope_error_to_phase6_port(
    error: DataLayerM10PartitionLifecycleError,
) -> DataLayerM10Phase6CompliancePortError {
    match error {
        DataLayerM10PartitionLifecycleError::OwnerScopeViolation { .. } => {
            DataLayerM10Phase6CompliancePortError::OwnerScopeViolation
        }
        DataLayerM10PartitionLifecycleError::ComplianceProjectionFailed { detail, .. } => {
            DataLayerM10Phase6CompliancePortError::InvalidInput(detail)
        }
        other => DataLayerM10Phase6CompliancePortError::InvalidInput(other.to_string()),
    }
}

pub(crate) fn map_phase6_port_error_to_m10(
    error: DataLayerM10Phase6CompliancePortError,
) -> DataLayerM10PartitionLifecycleError {
    match error {
        DataLayerM10Phase6CompliancePortError::OwnerScopeViolation => phase6_execution_failed(
            DATA_LAYER_M10_PHASE6_EXECUTION_OWNER_SCOPE_DENIED_REASON_CODE,
            "phase6 owner scope authorization failed",
        ),
        DataLayerM10Phase6CompliancePortError::LookupFailed(detail)
        | DataLayerM10Phase6CompliancePortError::InvalidInput(detail) => phase6_execution_failed(
            DATA_LAYER_M10_PHASE6_EXECUTION_INPUT_INVALID_REASON_CODE,
            detail,
        ),
    }
}
