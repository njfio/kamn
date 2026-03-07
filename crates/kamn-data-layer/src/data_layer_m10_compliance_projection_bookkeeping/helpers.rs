use std::collections::BTreeSet;

use crate::{
    data_layer_m10_validate_partition_month_id, DataLayerM10ComplianceProjectionPort,
    DataLayerM10ComplianceProjectionPortError, DataLayerM10PartitionRegistryStateMachineError,
};

use super::{
    DataLayerM10ComplianceProjectionBookkeepingError,
    DATA_LAYER_M10_COMPLIANCE_INPUT_INVALID_REASON_CODE,
    DATA_LAYER_M10_COMPLIANCE_LOOKUP_FAILED_REASON_CODE,
    DATA_LAYER_M10_COMPLIANCE_OWNER_SCOPE_DENIED_REASON_CODE,
};

pub(super) fn validate_partition_month_id(
    partition_month_id: u32,
) -> Result<(), DataLayerM10ComplianceProjectionBookkeepingError> {
    data_layer_m10_validate_partition_month_id(partition_month_id).map_err(|_| {
        DataLayerM10ComplianceProjectionBookkeepingError::InvalidPartitionMonthId(
            partition_month_id,
        )
    })
}

pub(super) fn collect_partition_message_ids(
    partition_message_ids: Vec<String>,
) -> Result<BTreeSet<String>, DataLayerM10ComplianceProjectionBookkeepingError> {
    if partition_message_ids.is_empty() {
        return Err(
            DataLayerM10ComplianceProjectionBookkeepingError::EmptyField("partition_message_ids"),
        );
    }
    let mut message_ids = BTreeSet::new();
    for message_id in partition_message_ids {
        if message_id.trim().is_empty() {
            return Err(
                DataLayerM10ComplianceProjectionBookkeepingError::EmptyField(
                    "partition_message_ids",
                ),
            );
        }
        message_ids.insert(message_id);
    }
    Ok(message_ids)
}

pub(super) fn evaluate_partition_message_shred_completeness(
    compliance_port: &impl DataLayerM10ComplianceProjectionPort,
    owner_did: &str,
    message_ids: &BTreeSet<String>,
) -> Result<(usize, usize), DataLayerM10ComplianceProjectionBookkeepingError> {
    let mut shredded_partition_messages = 0usize;
    let mut legal_hold_active_messages = 0usize;
    for message_id in message_ids {
        let message = compliance_port
            .message_for_owner(owner_did, message_id.as_str())
            .map_err(map_projection_port_error)?;
        if message.legal_hold_active {
            legal_hold_active_messages += 1;
        }
        if message.shredded_at_epoch_seconds.is_some() {
            shredded_partition_messages += 1;
        }
    }
    Ok((shredded_partition_messages, legal_hold_active_messages))
}

pub(super) fn map_projection_port_error(
    error: DataLayerM10ComplianceProjectionPortError,
) -> DataLayerM10ComplianceProjectionBookkeepingError {
    match error {
        DataLayerM10ComplianceProjectionPortError::OwnerScopeViolation => {
            DataLayerM10ComplianceProjectionBookkeepingError::OwnerScopeViolation {
                reason_code: DATA_LAYER_M10_COMPLIANCE_OWNER_SCOPE_DENIED_REASON_CODE,
            }
        }
        DataLayerM10ComplianceProjectionPortError::LookupFailed(detail) => {
            DataLayerM10ComplianceProjectionBookkeepingError::PortLookupFailed {
                reason_code: DATA_LAYER_M10_COMPLIANCE_LOOKUP_FAILED_REASON_CODE,
                detail,
            }
        }
        DataLayerM10ComplianceProjectionPortError::InvalidInput(detail) => {
            DataLayerM10ComplianceProjectionBookkeepingError::PortInvalidInput {
                reason_code: DATA_LAYER_M10_COMPLIANCE_INPUT_INVALID_REASON_CODE,
                detail,
            }
        }
    }
}

pub(super) fn map_state_machine_error(
    error: DataLayerM10PartitionRegistryStateMachineError,
) -> DataLayerM10ComplianceProjectionBookkeepingError {
    match error {
        DataLayerM10PartitionRegistryStateMachineError::EmptyField(field) => {
            DataLayerM10ComplianceProjectionBookkeepingError::EmptyField(field)
        }
        DataLayerM10PartitionRegistryStateMachineError::InvalidPartitionMonthId(value) => {
            DataLayerM10ComplianceProjectionBookkeepingError::InvalidPartitionMonthId(value)
        }
        DataLayerM10PartitionRegistryStateMachineError::PartitionNotFound(name) => {
            DataLayerM10ComplianceProjectionBookkeepingError::PartitionNotFound(name)
        }
        other => DataLayerM10ComplianceProjectionBookkeepingError::RegistryMutationFailed(
            other.to_string(),
        ),
    }
}
