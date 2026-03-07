use std::collections::BTreeMap;

use crate::{
    data_layer_m10_add_months, data_layer_m10_deterministic_checksum_marker,
    data_layer_m10_format_partition_name,
};

use super::helpers::{
    duplicate_partition_error, map_partition_month_error, missing_partition,
    project_partition_recovery_readiness, record_is_due_for_archive, validate_non_empty,
    validate_partition_month_id,
};
use super::{
    DataLayerM10ArchivalIndexEntry, DataLayerM10ArchiveDueRequest, DataLayerM10PartitionRecord,
    DataLayerM10PartitionRecordInput, DataLayerM10PartitionRegistryStateMachineError,
    DataLayerM10PartitionStatus, DataLayerM10RecoveryReadinessReport,
    DATA_LAYER_M10_ARCHIVE_FORMAT_PARQUET_ZSTD, DATA_LAYER_M10_ARCHIVE_REASON_CODE,
    DATA_LAYER_M10_INVALID_TRANSITION_REASON_CODE, DATA_LAYER_M10_REATTACH_REASON_CODE,
};

/// Deterministic registry state machine for M10 partition lifecycle behavior.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct DataLayerM10PartitionRegistryStateMachine {
    pub(super) partitions: BTreeMap<u32, DataLayerM10PartitionRecord>,
}

impl DataLayerM10PartitionRegistryStateMachine {
    /// Registers one monthly partition lifecycle record.
    pub fn register_partition(
        &mut self,
        input: DataLayerM10PartitionRecordInput,
    ) -> Result<DataLayerM10PartitionRecord, DataLayerM10PartitionRegistryStateMachineError> {
        validate_partition_month_id(input.partition_month_id)?;
        if self.partitions.contains_key(&input.partition_month_id) {
            return Err(duplicate_partition_error(input.partition_month_id));
        }
        let record = DataLayerM10PartitionRecord {
            partition_month_id: input.partition_month_id,
            partition_name: data_layer_m10_format_partition_name(input.partition_month_id)
                .map_err(map_partition_month_error)?,
            all_messages_shredded: input.all_messages_shredded,
            lifecycle_status: DataLayerM10PartitionStatus::Active,
            archived_object_uri: None,
            archive_format_marker: None,
            checksum_marker: None,
            last_reason_code: None,
        };
        self.partitions
            .insert(input.partition_month_id, record.clone());
        Ok(record)
    }

    /// Applies shred-completeness state to one existing partition.
    pub fn apply_partition_shred_completeness(
        &mut self,
        partition_month_id: u32,
        all_messages_shredded: bool,
        last_reason_code: &'static str,
    ) -> Result<DataLayerM10PartitionRecord, DataLayerM10PartitionRegistryStateMachineError> {
        validate_partition_month_id(partition_month_id)?;
        let partition_name = data_layer_m10_format_partition_name(partition_month_id)
            .map_err(map_partition_month_error)?;
        let record = self.partitions.get_mut(&partition_month_id).ok_or(
            DataLayerM10PartitionRegistryStateMachineError::PartitionNotFound(partition_name),
        )?;
        record.all_messages_shredded = all_messages_shredded;
        record.last_reason_code = Some(last_reason_code);
        Ok(record.clone())
    }

    /// Plans future partition names for `months_ahead` months after `reference_month_id`.
    pub fn plan_future_partition_names(
        &self,
        reference_month_id: u32,
        months_ahead: u8,
    ) -> Result<Vec<String>, DataLayerM10PartitionRegistryStateMachineError> {
        validate_partition_month_id(reference_month_id)?;
        let mut result = Vec::with_capacity(months_ahead as usize);
        for offset in 1..=u32::from(months_ahead) {
            let month_id = data_layer_m10_add_months(reference_month_id, offset)
                .map_err(map_partition_month_error)?;
            result.push(
                data_layer_m10_format_partition_name(month_id)
                    .map_err(map_partition_month_error)?,
            );
        }
        Ok(result)
    }

    /// Archives all due partitions and returns archival-index projections.
    pub fn archive_due_partitions(
        &mut self,
        request: DataLayerM10ArchiveDueRequest,
    ) -> Result<Vec<DataLayerM10ArchivalIndexEntry>, DataLayerM10PartitionRegistryStateMachineError>
    {
        validate_partition_month_id(request.now_month_id)?;
        validate_non_empty(
            request.object_storage_prefix.as_str(),
            "object_storage_prefix",
        )?;

        let mut entries = Vec::new();
        for record in self.partitions.values_mut() {
            if !record_is_due_for_archive(
                record,
                request.now_month_id,
                request.active_retention_months,
            )? {
                continue;
            }
            let archived_object_uri = format!(
                "{}/{}.parquet.zst",
                request.object_storage_prefix.trim_end_matches('/'),
                record.partition_name
            );
            let checksum_marker = data_layer_m10_deterministic_checksum_marker(
                record.partition_name.as_str(),
                record.partition_month_id,
            );
            record.lifecycle_status = DataLayerM10PartitionStatus::Archived;
            record.archived_object_uri = Some(archived_object_uri.clone());
            record.archive_format_marker = Some(DATA_LAYER_M10_ARCHIVE_FORMAT_PARQUET_ZSTD);
            record.checksum_marker = Some(checksum_marker.clone());
            record.last_reason_code = Some(DATA_LAYER_M10_ARCHIVE_REASON_CODE);
            entries.push(DataLayerM10ArchivalIndexEntry {
                partition_month_id: record.partition_month_id,
                partition_name: record.partition_name.clone(),
                archived_object_uri,
                archive_format_marker: DATA_LAYER_M10_ARCHIVE_FORMAT_PARQUET_ZSTD,
                checksum_marker,
                lifecycle_status: record.lifecycle_status,
            });
        }

        entries.sort_by(|left, right| {
            left.partition_month_id
                .cmp(&right.partition_month_id)
                .then(left.partition_name.cmp(&right.partition_name))
        });
        Ok(entries)
    }

    /// Re-attaches one archived partition for historical query access.
    pub fn reattach_partition(
        &mut self,
        partition_name: &str,
    ) -> Result<DataLayerM10PartitionRecord, DataLayerM10PartitionRegistryStateMachineError> {
        validate_non_empty(partition_name, "partition_name")?;
        let record = self
            .partitions
            .values_mut()
            .find(|entry| entry.partition_name == partition_name)
            .ok_or_else(|| missing_partition(partition_name))?;
        if record.lifecycle_status != DataLayerM10PartitionStatus::Archived {
            return Err(
                DataLayerM10PartitionRegistryStateMachineError::InvalidLifecycleTransition {
                    partition_name: record.partition_name.clone(),
                    from_status: record.lifecycle_status,
                    to_status: DataLayerM10PartitionStatus::Reattached,
                    reason_code: DATA_LAYER_M10_INVALID_TRANSITION_REASON_CODE,
                },
            );
        }
        record.lifecycle_status = DataLayerM10PartitionStatus::Reattached;
        record.last_reason_code = Some(DATA_LAYER_M10_REATTACH_REASON_CODE);
        Ok(record.clone())
    }

    /// Evaluates recoverability readiness for one partition.
    pub fn evaluate_partition_recovery_readiness(
        &self,
        partition_name: &str,
    ) -> Result<DataLayerM10RecoveryReadinessReport, DataLayerM10PartitionRegistryStateMachineError>
    {
        validate_non_empty(partition_name, "partition_name")?;
        let record = self
            .partitions
            .values()
            .find(|entry| entry.partition_name == partition_name)
            .ok_or_else(|| missing_partition(partition_name))?;
        Ok(project_partition_recovery_readiness(record))
    }
}
