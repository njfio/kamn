use std::collections::BTreeSet;

use crate::DataLayerM8ComplianceRegistry;

use super::error::map_m8_projection_error_to_m10;
use super::shared::{
    add_months, authorize_owner_scope, deterministic_checksum_marker, month_distance,
    split_month_id, validate_non_empty, validate_partition_month_id,
};
use super::*;

impl DataLayerM10PartitionLifecycleRegistry {
    /// Creates an empty partition lifecycle registry.
    pub fn new() -> Self {
        Self::default()
    }

    /// Registers one monthly partition lifecycle record.
    pub fn register_partition(
        &mut self,
        input: DataLayerM10PartitionRecordInput,
    ) -> Result<DataLayerM10PartitionRecord, DataLayerM10PartitionLifecycleError> {
        validate_partition_month_id(input.partition_month_id)?;
        if self.partitions.contains_key(&input.partition_month_id) {
            return Err(
                DataLayerM10PartitionLifecycleError::DuplicatePartitionMonthId(
                    input.partition_month_id,
                ),
            );
        }

        let record = DataLayerM10PartitionRecord {
            partition_month_id: input.partition_month_id,
            partition_name: data_layer_m10_format_partition_name(input.partition_month_id)?,
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

    /// Derives partition shred completeness from M8 lifecycle records and updates partition state.
    pub fn project_partition_shred_completeness_from_m8(
        &mut self,
        compliance_registry: &DataLayerM8ComplianceRegistry,
        request: DataLayerM10ComplianceShredProjectionRequest,
    ) -> Result<DataLayerM10ComplianceShredProjectionReport, DataLayerM10PartitionLifecycleError>
    {
        let owner_did = authorize_owner_scope(
            request.requester_owner_did.as_str(),
            request.owner_did.as_str(),
        )?;
        validate_partition_month_id(request.partition_month_id)?;
        if request.partition_message_ids.is_empty() {
            return Err(DataLayerM10PartitionLifecycleError::EmptyField(
                "partition_message_ids",
            ));
        }

        let mut message_ids = BTreeSet::new();
        for message_id in request.partition_message_ids {
            validate_non_empty(message_id.as_str(), "partition_message_ids")?;
            message_ids.insert(message_id);
        }

        let total_partition_messages = message_ids.len();
        let mut shredded_partition_messages = 0usize;
        let mut legal_hold_active_messages = 0usize;
        for message_id in &message_ids {
            let message = compliance_registry
                .message_for_owner(owner_did.as_str(), message_id.as_str())
                .map_err(map_m8_projection_error_to_m10)?;
            if message.legal_hold_active {
                legal_hold_active_messages += 1;
            }
            if message.shredded_at_epoch_seconds.is_some() {
                shredded_partition_messages += 1;
            }
        }
        let all_messages_shredded = shredded_partition_messages == total_partition_messages;
        let reason_code = if legal_hold_active_messages > 0 {
            DATA_LAYER_M10_COMPLIANCE_LEGAL_HOLD_ACTIVE_REASON_CODE
        } else if all_messages_shredded {
            DATA_LAYER_M10_COMPLIANCE_SHRED_COMPLETENESS_TRUE_REASON_CODE
        } else {
            DATA_LAYER_M10_COMPLIANCE_SHRED_COMPLETENESS_FALSE_REASON_CODE
        };

        let partition_name = data_layer_m10_format_partition_name(request.partition_month_id)?;
        let record = self
            .partitions
            .get_mut(&request.partition_month_id)
            .ok_or_else(|| {
                DataLayerM10PartitionLifecycleError::PartitionNotFound(partition_name.clone())
            })?;
        record.all_messages_shredded = all_messages_shredded;
        record.last_reason_code = Some(DATA_LAYER_M10_COMPLIANCE_PROJECTION_APPLIED_REASON_CODE);

        Ok(DataLayerM10ComplianceShredProjectionReport {
            partition_month_id: record.partition_month_id,
            partition_name: record.partition_name.clone(),
            total_partition_messages,
            shredded_partition_messages,
            all_messages_shredded,
            reason_code,
            projection_reason_code: DATA_LAYER_M10_COMPLIANCE_PROJECTION_APPLIED_REASON_CODE,
        })
    }

    /// Plans future partition names for `months_ahead` months after `reference_month_id`.
    pub fn plan_future_partition_names(
        &self,
        reference_month_id: u32,
        months_ahead: u8,
    ) -> Result<Vec<String>, DataLayerM10PartitionLifecycleError> {
        validate_partition_month_id(reference_month_id)?;
        let mut result = Vec::with_capacity(months_ahead as usize);
        for offset in 1..=u32::from(months_ahead) {
            let month_id = add_months(reference_month_id, offset)?;
            result.push(data_layer_m10_format_partition_name(month_id)?);
        }
        Ok(result)
    }

    /// Archives all due partitions and returns archival-index projections.
    pub fn archive_due_partitions(
        &mut self,
        request: DataLayerM10ArchiveDueRequest,
    ) -> Result<Vec<DataLayerM10ArchivalIndexEntry>, DataLayerM10PartitionLifecycleError> {
        validate_partition_month_id(request.now_month_id)?;
        validate_non_empty(
            request.object_storage_prefix.as_str(),
            "object_storage_prefix",
        )?;

        let mut entries = Vec::new();
        for record in self.partitions.values_mut() {
            if record.lifecycle_status != DataLayerM10PartitionStatus::Active {
                continue;
            }
            if !record.all_messages_shredded {
                continue;
            }
            if record.partition_month_id > request.now_month_id {
                continue;
            }

            let age_months = month_distance(record.partition_month_id, request.now_month_id)?;
            if age_months <= u32::from(request.active_retention_months) {
                continue;
            }

            let archived_object_uri = format!(
                "{}/{}.parquet.zst",
                request.object_storage_prefix.trim_end_matches('/'),
                record.partition_name
            );
            let checksum_marker = deterministic_checksum_marker(
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
    ) -> Result<DataLayerM10PartitionRecord, DataLayerM10PartitionLifecycleError> {
        validate_non_empty(partition_name, "partition_name")?;
        let record = self
            .partitions
            .values_mut()
            .find(|entry| entry.partition_name == partition_name)
            .ok_or_else(|| {
                DataLayerM10PartitionLifecycleError::PartitionNotFound(partition_name.to_owned())
            })?;

        if record.lifecycle_status != DataLayerM10PartitionStatus::Archived {
            return Err(
                DataLayerM10PartitionLifecycleError::InvalidLifecycleTransition {
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
    ) -> Result<DataLayerM10RecoveryReadinessReport, DataLayerM10PartitionLifecycleError> {
        validate_non_empty(partition_name, "partition_name")?;
        let record = self
            .partitions
            .values()
            .find(|entry| entry.partition_name == partition_name)
            .ok_or_else(|| {
                DataLayerM10PartitionLifecycleError::PartitionNotFound(partition_name.to_owned())
            })?;
        Ok(project_partition_recovery_readiness(record))
    }

    /// Lists recoverability readiness for historical partitions in deterministic order.
    pub fn list_historical_recovery_readiness(&self) -> Vec<DataLayerM10RecoveryReadinessReport> {
        let mut reports: Vec<DataLayerM10RecoveryReadinessReport> = self
            .partitions
            .values()
            .filter(|record| record.lifecycle_status != DataLayerM10PartitionStatus::Active)
            .map(project_partition_recovery_readiness)
            .collect();
        reports.sort_by(|left, right| {
            left.partition_month_id
                .cmp(&right.partition_month_id)
                .then(left.partition_name.cmp(&right.partition_name))
        });
        reports
    }
}

/// Formats partition month id (`YYYYMM`) as `messages_YYYY_MM`.
pub fn data_layer_m10_format_partition_name(
    partition_month_id: u32,
) -> Result<String, DataLayerM10PartitionLifecycleError> {
    let (year, month) = split_month_id(partition_month_id)?;
    Ok(format!(
        "{DATA_LAYER_M10_PARTITION_PREFIX}{year:04}_{month:02}"
    ))
}

fn project_partition_recovery_readiness(
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
