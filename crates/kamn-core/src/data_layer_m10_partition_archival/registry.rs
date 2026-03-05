use std::collections::BTreeSet;

use kamn_data_layer::{
    DataLayerM10ComplianceProjectionMessageState, DataLayerM10ComplianceProjectionPort,
    DataLayerM10ComplianceProjectionPortError,
};

use crate::{DataLayerM8ComplianceError, DataLayerM8ComplianceRegistry, KamnDid};
use super::shared::{
    add_months, deterministic_checksum_marker, month_distance, split_month_id, validate_non_empty,
    validate_partition_month_id,
};
use super::*;

struct M8ComplianceProjectionPortAdapter<'a> {
    compliance_registry: &'a DataLayerM8ComplianceRegistry,
}

impl<'a> M8ComplianceProjectionPortAdapter<'a> {
    fn new(compliance_registry: &'a DataLayerM8ComplianceRegistry) -> Self {
        Self {
            compliance_registry,
        }
    }
}

impl DataLayerM10ComplianceProjectionPort for M8ComplianceProjectionPortAdapter<'_> {
    fn authorize_owner_scope(
        &self,
        requester_owner_did: &str,
        owner_did: &str,
    ) -> Result<String, DataLayerM10ComplianceProjectionPortError> {
        let requester_owner_did = normalize_did(requester_owner_did)?;
        let owner_did = normalize_did(owner_did)?;
        if requester_owner_did.as_str() != owner_did.as_str() {
            return Err(DataLayerM10ComplianceProjectionPortError::OwnerScopeViolation);
        }
        Ok(owner_did.as_str().to_owned())
    }

    fn message_for_owner(
        &self,
        owner_did: &str,
        message_id: &str,
    ) -> Result<DataLayerM10ComplianceProjectionMessageState, DataLayerM10ComplianceProjectionPortError>
    {
        let message = self
            .compliance_registry
            .message_for_owner(owner_did, message_id)
            .map_err(map_m8_projection_error_to_port)?;
        Ok(DataLayerM10ComplianceProjectionMessageState {
            message_id: message.message_id.clone(),
            legal_hold_active: message.legal_hold_active,
            shredded_at_epoch_seconds: message.shredded_at_epoch_seconds,
        })
    }
}

fn normalize_did(value: &str) -> Result<KamnDid, DataLayerM10ComplianceProjectionPortError> {
    KamnDid::parse(value).map_err(|_| {
        DataLayerM10ComplianceProjectionPortError::InvalidInput(format!("invalid did: {value}"))
    })
}

fn map_m8_projection_error_to_port(
    error: DataLayerM8ComplianceError,
) -> DataLayerM10ComplianceProjectionPortError {
    match error {
        DataLayerM8ComplianceError::OwnerScopeViolation { .. } => {
            DataLayerM10ComplianceProjectionPortError::OwnerScopeViolation
        }
        DataLayerM8ComplianceError::OwnerNotFound { .. }
        | DataLayerM8ComplianceError::MessageNotFound { .. } => {
            DataLayerM10ComplianceProjectionPortError::LookupFailed(error.to_string())
        }
        DataLayerM8ComplianceError::InvalidDid(_)
        | DataLayerM8ComplianceError::EmptyField(_)
        | DataLayerM8ComplianceError::EmptyWrappedKeys
        | DataLayerM8ComplianceError::InvalidWrappedKey(_)
        | DataLayerM8ComplianceError::DuplicateWrappedKeyRecipient { .. }
        | DataLayerM8ComplianceError::DuplicateMessageId { .. }
        | DataLayerM8ComplianceError::LegalHoldActive { .. }
        | DataLayerM8ComplianceError::AlreadyShredded { .. } => {
            DataLayerM10ComplianceProjectionPortError::InvalidInput(error.to_string())
        }
    }
}

fn map_projection_port_error_to_m10(
    error: DataLayerM10ComplianceProjectionPortError,
) -> DataLayerM10PartitionLifecycleError {
    match error {
        DataLayerM10ComplianceProjectionPortError::OwnerScopeViolation => {
            DataLayerM10PartitionLifecycleError::OwnerScopeViolation {
                reason_code: DATA_LAYER_M10_COMPLIANCE_OWNER_SCOPE_DENIED_REASON_CODE,
            }
        }
        DataLayerM10ComplianceProjectionPortError::LookupFailed(detail) => {
            DataLayerM10PartitionLifecycleError::ComplianceProjectionFailed {
                reason_code: DATA_LAYER_M10_COMPLIANCE_LOOKUP_FAILED_REASON_CODE,
                detail,
            }
        }
        DataLayerM10ComplianceProjectionPortError::InvalidInput(detail) => {
            DataLayerM10PartitionLifecycleError::ComplianceProjectionFailed {
                reason_code: DATA_LAYER_M10_COMPLIANCE_INPUT_INVALID_REASON_CODE,
                detail,
            }
        }
    }
}

fn collect_partition_message_ids(
    partition_message_ids: Vec<String>,
) -> Result<BTreeSet<String>, DataLayerM10PartitionLifecycleError> {
    if partition_message_ids.is_empty() {
        return Err(DataLayerM10PartitionLifecycleError::EmptyField(
            "partition_message_ids",
        ));
    }
    let mut message_ids = BTreeSet::new();
    for message_id in partition_message_ids {
        validate_non_empty(message_id.as_str(), "partition_message_ids")?;
        message_ids.insert(message_id);
    }
    Ok(message_ids)
}

fn evaluate_partition_message_shred_completeness(
    compliance_port: &impl DataLayerM10ComplianceProjectionPort,
    owner_did: &str,
    message_ids: &BTreeSet<String>,
) -> Result<(usize, usize), DataLayerM10PartitionLifecycleError> {
    let mut shredded_partition_messages = 0usize;
    let mut legal_hold_active_messages = 0usize;
    for message_id in message_ids {
        let message = compliance_port
            .message_for_owner(owner_did, message_id.as_str())
            .map_err(map_projection_port_error_to_m10)?;
        if message.legal_hold_active {
            legal_hold_active_messages += 1;
        }
        if message.shredded_at_epoch_seconds.is_some() {
            shredded_partition_messages += 1;
        }
    }
    Ok((shredded_partition_messages, legal_hold_active_messages))
}

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
        let compliance_port = M8ComplianceProjectionPortAdapter::new(compliance_registry);
        self.project_partition_shred_completeness_with_port(&compliance_port, request)
    }

    /// Derives partition shred completeness from a core-agnostic compliance projection port.
    pub fn project_partition_shred_completeness_with_port(
        &mut self,
        compliance_port: &impl DataLayerM10ComplianceProjectionPort,
        request: DataLayerM10ComplianceShredProjectionRequest,
    ) -> Result<DataLayerM10ComplianceShredProjectionReport, DataLayerM10PartitionLifecycleError>
    {
        let owner_did = compliance_port
            .authorize_owner_scope(
                request.requester_owner_did.as_str(),
                request.owner_did.as_str(),
            )
            .map_err(map_projection_port_error_to_m10)?;
        validate_partition_month_id(request.partition_month_id)?;
        let message_ids = collect_partition_message_ids(request.partition_message_ids)?;
        let total_partition_messages = message_ids.len();
        let (shredded_partition_messages, legal_hold_active_messages) =
            evaluate_partition_message_shred_completeness(
                compliance_port,
                owner_did.as_str(),
                &message_ids,
            )?;
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

#[cfg(test)]
mod tests {
    use super::{
        data_layer_m10_format_partition_name, DataLayerM10ArchiveDueRequest,
        DataLayerM10PartitionLifecycleError, DataLayerM10PartitionLifecycleRegistry,
        DataLayerM10PartitionRecordInput, DataLayerM10PartitionStatus,
        DataLayerM10RecoveryDecision, DATA_LAYER_M10_ARCHIVE_FORMAT_PARQUET_ZSTD,
        DATA_LAYER_M10_INVALID_TRANSITION_REASON_CODE, DATA_LAYER_M10_REATTACH_REASON_CODE,
        DATA_LAYER_M10_RECOVERY_READY_REASON_CODE,
        DATA_LAYER_M10_RECOVERY_STATUS_INELIGIBLE_REASON_CODE,
    };

    fn register_partition(
        registry: &mut DataLayerM10PartitionLifecycleRegistry,
        partition_month_id: u32,
        all_messages_shredded: bool,
    ) {
        registry
            .register_partition(DataLayerM10PartitionRecordInput {
                partition_month_id,
                all_messages_shredded,
            })
            .expect("fixture partition registration must succeed");
    }

    #[test]
    fn unit_m10_registry_registration_and_future_plan_are_deterministic() {
        let mut registry = DataLayerM10PartitionLifecycleRegistry::new();

        let jan = registry
            .register_partition(DataLayerM10PartitionRecordInput {
                partition_month_id: 202401,
                all_messages_shredded: false,
            })
            .expect("first partition should register");
        let feb = registry
            .register_partition(DataLayerM10PartitionRecordInput {
                partition_month_id: 202402,
                all_messages_shredded: true,
            })
            .expect("second partition should register");

        assert_eq!(jan.partition_name, "messages_2024_01");
        assert_eq!(feb.partition_name, "messages_2024_02");
        assert_eq!(jan.lifecycle_status, DataLayerM10PartitionStatus::Active);
        assert_eq!(feb.lifecycle_status, DataLayerM10PartitionStatus::Active);

        assert_eq!(
            registry.register_partition(DataLayerM10PartitionRecordInput {
                partition_month_id: 202401,
                all_messages_shredded: true,
            }),
            Err(DataLayerM10PartitionLifecycleError::DuplicatePartitionMonthId(202401))
        );

        assert_eq!(
            registry
                .plan_future_partition_names(202412, 3)
                .expect("planning must succeed"),
            vec![
                "messages_2025_01".to_owned(),
                "messages_2025_02".to_owned(),
                "messages_2025_03".to_owned(),
            ]
        );
    }

    #[test]
    fn unit_m10_registry_archives_only_due_shredded_partitions_in_sorted_order() {
        let mut registry = DataLayerM10PartitionLifecycleRegistry::new();
        register_partition(&mut registry, 202310, true);
        register_partition(&mut registry, 202311, true);
        register_partition(&mut registry, 202312, false);
        register_partition(&mut registry, 202401, true);

        let entries = registry
            .archive_due_partitions(DataLayerM10ArchiveDueRequest {
                now_month_id: 202501,
                active_retention_months: 12,
                object_storage_prefix: "s3://kamn-archive/".to_owned(),
            })
            .expect("archive evaluation should succeed");

        assert_eq!(
            entries
                .iter()
                .map(|entry| entry.partition_month_id)
                .collect::<Vec<_>>(),
            vec![202310, 202311]
        );
        assert_eq!(
            entries
                .iter()
                .map(|entry| entry.partition_name.as_str())
                .collect::<Vec<_>>(),
            vec!["messages_2023_10", "messages_2023_11"]
        );
        assert_eq!(
            entries
                .iter()
                .map(|entry| entry.archived_object_uri.as_str())
                .collect::<Vec<_>>(),
            vec![
                "s3://kamn-archive/messages_2023_10.parquet.zst",
                "s3://kamn-archive/messages_2023_11.parquet.zst"
            ]
        );
        assert!(
            entries
                .iter()
                .all(|entry| entry.archive_format_marker
                    == DATA_LAYER_M10_ARCHIVE_FORMAT_PARQUET_ZSTD)
        );
        assert!(entries
            .iter()
            .all(|entry| entry.lifecycle_status == DataLayerM10PartitionStatus::Archived));
    }

    #[test]
    fn regression_m10_registry_reattach_rejects_active_partition_then_allows_archived() {
        let mut registry = DataLayerM10PartitionLifecycleRegistry::new();
        register_partition(&mut registry, 202401, true);

        assert!(matches!(
            registry.reattach_partition("messages_2024_01"),
            Err(DataLayerM10PartitionLifecycleError::InvalidLifecycleTransition {
                partition_name,
                from_status: DataLayerM10PartitionStatus::Active,
                to_status: DataLayerM10PartitionStatus::Reattached,
                reason_code: DATA_LAYER_M10_INVALID_TRANSITION_REASON_CODE,
            }) if partition_name == "messages_2024_01"
        ));

        registry
            .archive_due_partitions(DataLayerM10ArchiveDueRequest {
                now_month_id: 202601,
                active_retention_months: 12,
                object_storage_prefix: "s3://kamn-archive".to_owned(),
            })
            .expect("archival should move partition to archived state");

        let reattached = registry
            .reattach_partition("messages_2024_01")
            .expect("archived partition should reattach");
        assert_eq!(
            reattached.lifecycle_status,
            DataLayerM10PartitionStatus::Reattached
        );
        assert_eq!(
            reattached.last_reason_code,
            Some(DATA_LAYER_M10_REATTACH_REASON_CODE)
        );
    }

    #[test]
    fn unit_m10_registry_recovery_readiness_blocks_active_and_accepts_archived_metadata() {
        let mut registry = DataLayerM10PartitionLifecycleRegistry::new();
        register_partition(&mut registry, 202402, true);
        register_partition(&mut registry, 202501, true);

        let active_report = registry
            .evaluate_partition_recovery_readiness("messages_2025_01")
            .expect("active partition readiness should evaluate");
        assert_eq!(
            active_report.decision,
            DataLayerM10RecoveryDecision::Blocked
        );
        assert_eq!(
            active_report.reason_code,
            DATA_LAYER_M10_RECOVERY_STATUS_INELIGIBLE_REASON_CODE
        );

        registry
            .archive_due_partitions(DataLayerM10ArchiveDueRequest {
                now_month_id: 202601,
                active_retention_months: 12,
                object_storage_prefix: "s3://kamn-archive".to_owned(),
            })
            .expect("archival should produce metadata for old partition");

        let archived_partition_name =
            data_layer_m10_format_partition_name(202402).expect("partition name should format");
        let archived_report = registry
            .evaluate_partition_recovery_readiness(archived_partition_name.as_str())
            .expect("archived partition readiness should evaluate");
        assert_eq!(
            archived_report.decision,
            DataLayerM10RecoveryDecision::Ready
        );
        assert_eq!(
            archived_report.reason_code,
            DATA_LAYER_M10_RECOVERY_READY_REASON_CODE
        );
        assert_eq!(
            archived_report.archive_format_marker,
            Some(DATA_LAYER_M10_ARCHIVE_FORMAT_PARQUET_ZSTD)
        );
        assert!(archived_report.archived_object_uri.is_some());
        assert!(archived_report.checksum_marker.is_some());
    }
}
