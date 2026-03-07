use crate::{
    data_layer_m10_month_distance, data_layer_m10_validate_partition_month_id,
    DataLayerM10PartitionMonthPolicyError,
};

use super::{
    DataLayerM10PartitionRecord, DataLayerM10PartitionRegistryStateMachineError,
    DataLayerM10PartitionStatus, DataLayerM10RecoveryDecision, DataLayerM10RecoveryReadinessReport,
    DATA_LAYER_M10_ARCHIVE_FORMAT_PARQUET_ZSTD,
    DATA_LAYER_M10_RECOVERY_METADATA_INCOMPLETE_REASON_CODE,
    DATA_LAYER_M10_RECOVERY_READY_REASON_CODE,
    DATA_LAYER_M10_RECOVERY_STATUS_INELIGIBLE_REASON_CODE,
};

pub(super) fn validate_partition_month_id(
    partition_month_id: u32,
) -> Result<(), DataLayerM10PartitionRegistryStateMachineError> {
    data_layer_m10_validate_partition_month_id(partition_month_id)
        .map_err(map_partition_month_error)
}

pub(super) fn validate_non_empty(
    value: &str,
    field: &'static str,
) -> Result<(), DataLayerM10PartitionRegistryStateMachineError> {
    if value.trim().is_empty() {
        return Err(DataLayerM10PartitionRegistryStateMachineError::EmptyField(
            field,
        ));
    }
    Ok(())
}

pub(super) fn record_is_due_for_archive(
    record: &DataLayerM10PartitionRecord,
    now_month_id: u32,
    active_retention_months: u16,
) -> Result<bool, DataLayerM10PartitionRegistryStateMachineError> {
    if record.lifecycle_status != DataLayerM10PartitionStatus::Active
        || !record.all_messages_shredded
        || record.partition_month_id > now_month_id
    {
        return Ok(false);
    }
    let age_months = data_layer_m10_month_distance(record.partition_month_id, now_month_id)
        .map_err(map_partition_month_error)?;
    Ok(age_months > u32::from(active_retention_months))
}

pub(super) fn project_partition_recovery_readiness(
    record: &DataLayerM10PartitionRecord,
) -> DataLayerM10RecoveryReadinessReport {
    let metadata_complete = record
        .archived_object_uri
        .as_deref()
        .is_some_and(|value| !value.trim().is_empty())
        && record.archive_format_marker == Some(DATA_LAYER_M10_ARCHIVE_FORMAT_PARQUET_ZSTD)
        && record
            .checksum_marker
            .as_deref()
            .is_some_and(|value| !value.trim().is_empty());
    let (decision, reason_code) = match record.lifecycle_status {
        DataLayerM10PartitionStatus::Active => (
            DataLayerM10RecoveryDecision::Blocked,
            DATA_LAYER_M10_RECOVERY_STATUS_INELIGIBLE_REASON_CODE,
        ),
        DataLayerM10PartitionStatus::Archived | DataLayerM10PartitionStatus::Reattached => {
            if metadata_complete {
                (
                    DataLayerM10RecoveryDecision::Ready,
                    DATA_LAYER_M10_RECOVERY_READY_REASON_CODE,
                )
            } else {
                (
                    DataLayerM10RecoveryDecision::Blocked,
                    DATA_LAYER_M10_RECOVERY_METADATA_INCOMPLETE_REASON_CODE,
                )
            }
        }
    };

    DataLayerM10RecoveryReadinessReport {
        partition_month_id: record.partition_month_id,
        partition_name: record.partition_name.clone(),
        decision,
        reason_code,
        lifecycle_status: record.lifecycle_status,
        archived_object_uri: record.archived_object_uri.clone(),
        archive_format_marker: record.archive_format_marker,
        checksum_marker: record.checksum_marker.clone(),
    }
}

pub(super) fn map_partition_month_error(
    error: DataLayerM10PartitionMonthPolicyError,
) -> DataLayerM10PartitionRegistryStateMachineError {
    match error {
        DataLayerM10PartitionMonthPolicyError::InvalidPartitionMonthId(value) => {
            DataLayerM10PartitionRegistryStateMachineError::InvalidPartitionMonthId(value)
        }
    }
}

pub(super) fn missing_partition(
    partition_name: &str,
) -> DataLayerM10PartitionRegistryStateMachineError {
    DataLayerM10PartitionRegistryStateMachineError::PartitionNotFound(partition_name.to_owned())
}

pub(super) fn duplicate_partition_error(
    partition_month_id: u32,
) -> DataLayerM10PartitionRegistryStateMachineError {
    DataLayerM10PartitionRegistryStateMachineError::DuplicatePartitionMonthId(partition_month_id)
}
