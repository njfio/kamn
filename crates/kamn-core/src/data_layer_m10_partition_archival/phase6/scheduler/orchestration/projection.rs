use std::collections::BTreeSet;

use kamn_data_layer::{
    DataLayerM10ComplianceShredProjectionRequest, DataLayerM10Phase6CompliancePort,
    DataLayerM10Phase6CryptoShredInput,
};

use crate::data_layer_m10_partition_archival::error::{
    map_phase6_projection_error_to_m10, phase6_execution_failed,
};
use crate::data_layer_m10_partition_archival::shared::validate_non_empty;
use crate::data_layer_m10_partition_archival::{
    DataLayerM10PartitionLifecycleError, DataLayerM10PartitionLifecycleRegistry,
    DataLayerM10Phase6ExecutionTickRequest,
    DATA_LAYER_M10_PHASE6_EXECUTION_LEGAL_HOLD_ACTIVE_REASON_CODE,
    DATA_LAYER_M10_PHASE6_EXECUTION_PROJECTION_INPUT_INVALID_REASON_CODE,
};

use super::super::super::adapters::bridge::Phase6ProjectionPortBridge;
use super::super::super::adapters::error_mapping::map_phase6_port_error_to_m10;

pub(super) fn shred_due_candidates(
    compliance_port: &mut impl DataLayerM10Phase6CompliancePort,
    owner_did: &str,
    shredded_at_epoch_seconds: u64,
    due_candidates: Vec<kamn_data_layer::DataLayerM10Phase6RetentionDueCandidate>,
) -> Result<Vec<String>, DataLayerM10PartitionLifecycleError> {
    let mut shredded_message_ids = Vec::with_capacity(due_candidates.len());
    for candidate in due_candidates {
        compliance_port
            .crypto_shred(DataLayerM10Phase6CryptoShredInput {
                requester_owner_did: owner_did.to_owned(),
                owner_did: owner_did.to_owned(),
                message_id: candidate.message_id.clone(),
                shredded_at_epoch_seconds,
            })
            .map_err(map_phase6_port_error_to_m10)?;
        shredded_message_ids.push(candidate.message_id);
    }
    shredded_message_ids.sort();
    Ok(shredded_message_ids)
}

pub(super) fn project_partition_reports(
    compliance_port: &mut impl DataLayerM10Phase6CompliancePort,
    partition_registry: &mut DataLayerM10PartitionLifecycleRegistry,
    request: &DataLayerM10Phase6ExecutionTickRequest,
    owner_did: &str,
) -> Result<
    Vec<crate::DataLayerM10ComplianceShredProjectionReport>,
    DataLayerM10PartitionLifecycleError,
> {
    let mut projection_reports = Vec::with_capacity(request.partition_message_ids_by_month.len());
    for (partition_month_id, partition_message_ids) in &request.partition_message_ids_by_month {
        projection_reports.push(project_single_partition_report(
            compliance_port,
            partition_registry,
            owner_did,
            *partition_month_id,
            partition_message_ids,
        )?);
    }
    Ok(projection_reports)
}

fn project_single_partition_report(
    compliance_port: &mut impl DataLayerM10Phase6CompliancePort,
    partition_registry: &mut DataLayerM10PartitionLifecycleRegistry,
    owner_did: &str,
    partition_month_id: u32,
    partition_message_ids: &[String],
) -> Result<crate::DataLayerM10ComplianceShredProjectionReport, DataLayerM10PartitionLifecycleError>
{
    validate_partition_message_ids(partition_month_id, partition_message_ids)?;
    let partition_message_ids =
        build_partition_message_ids(compliance_port, owner_did, partition_message_ids)?;
    let projection_port = Phase6ProjectionPortBridge::new(&*compliance_port);
    partition_registry
        .project_partition_shred_completeness_with_port(
            &projection_port,
            build_projection_request(owner_did, partition_month_id, partition_message_ids),
        )
        .map_err(map_phase6_projection_error_to_m10)
}

fn validate_partition_message_ids(
    partition_month_id: u32,
    partition_message_ids: &[String],
) -> Result<(), DataLayerM10PartitionLifecycleError> {
    if partition_message_ids.is_empty() {
        return Err(phase6_execution_failed(
            DATA_LAYER_M10_PHASE6_EXECUTION_PROJECTION_INPUT_INVALID_REASON_CODE,
            format!("partition {partition_month_id} message set is empty"),
        ));
    }
    Ok(())
}

fn build_projection_request(
    owner_did: &str,
    partition_month_id: u32,
    partition_message_ids: Vec<String>,
) -> DataLayerM10ComplianceShredProjectionRequest {
    DataLayerM10ComplianceShredProjectionRequest {
        requester_owner_did: owner_did.to_owned(),
        owner_did: owner_did.to_owned(),
        partition_month_id,
        partition_message_ids,
    }
}

fn build_partition_message_ids(
    compliance_port: &mut impl DataLayerM10Phase6CompliancePort,
    owner_did: &str,
    partition_message_ids: &[String],
) -> Result<Vec<String>, DataLayerM10PartitionLifecycleError> {
    let mut deduped_partition_message_ids = BTreeSet::new();
    for message_id in partition_message_ids {
        validate_non_empty(message_id.as_str(), "partition_message_ids")
            .map_err(map_phase6_projection_error_to_m10)?;
        let message = compliance_port
            .message_for_owner(owner_did, message_id.as_str())
            .map_err(map_phase6_port_error_to_m10)?;
        if message.legal_hold_active {
            return Err(phase6_execution_failed(
                DATA_LAYER_M10_PHASE6_EXECUTION_LEGAL_HOLD_ACTIVE_REASON_CODE,
                format!("message {} is under legal hold", message.message_id),
            ));
        }
        deduped_partition_message_ids.insert(message_id.clone());
    }
    Ok(deduped_partition_message_ids.into_iter().collect())
}

pub(super) fn archive_due_partitions(
    partition_registry: &mut DataLayerM10PartitionLifecycleRegistry,
    request: DataLayerM10Phase6ExecutionTickRequest,
) -> Result<Vec<crate::DataLayerM10ArchivalIndexEntry>, DataLayerM10PartitionLifecycleError> {
    partition_registry
        .archive_due_partitions(crate::DataLayerM10ArchiveDueRequest {
            now_month_id: request.now_month_id,
            active_retention_months: request.active_retention_months,
            object_storage_prefix: request.object_storage_prefix,
        })
        .map_err(map_phase6_projection_error_to_m10)
}
