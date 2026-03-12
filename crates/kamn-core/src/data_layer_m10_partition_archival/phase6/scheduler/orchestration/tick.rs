use kamn_data_layer::DataLayerM10Phase6CompliancePort;

use crate::data_layer_m10_partition_archival::error::phase6_execution_failed;
use crate::data_layer_m10_partition_archival::shared::validate_non_empty;
use crate::data_layer_m10_partition_archival::{
    DataLayerM10PartitionLifecycleError, DataLayerM10PartitionLifecycleRegistry,
    DataLayerM10Phase6ExecutionTickReport, DataLayerM10Phase6ExecutionTickRequest,
    DATA_LAYER_M10_PHASE6_EXECUTION_APPLIED_REASON_CODE,
    DATA_LAYER_M10_PHASE6_EXECUTION_INPUT_INVALID_REASON_CODE,
};
use crate::DataLayerM8ComplianceRegistry;

use super::projection::{
    archive_due_partitions, project_partition_reports, shred_due_candidates,
};
use super::super::super::adapters::bridge::M8Phase6CompliancePortAdapter;
use super::super::super::adapters::error_mapping::map_phase6_port_error_to_m10;

pub fn data_layer_m10_execute_phase6_orchestration_tick(
    compliance_registry: &mut DataLayerM8ComplianceRegistry,
    partition_registry: &mut DataLayerM10PartitionLifecycleRegistry,
    request: DataLayerM10Phase6ExecutionTickRequest,
) -> Result<DataLayerM10Phase6ExecutionTickReport, DataLayerM10PartitionLifecycleError> {
    let mut compliance_port = M8Phase6CompliancePortAdapter::new(compliance_registry);
    data_layer_m10_execute_phase6_orchestration_tick_with_port(
        &mut compliance_port,
        partition_registry,
        request,
    )
}

pub fn data_layer_m10_execute_phase6_orchestration_tick_with_port(
    compliance_port: &mut impl DataLayerM10Phase6CompliancePort,
    partition_registry: &mut DataLayerM10PartitionLifecycleRegistry,
    request: DataLayerM10Phase6ExecutionTickRequest,
) -> Result<DataLayerM10Phase6ExecutionTickReport, DataLayerM10PartitionLifecycleError> {
    let owner_did = authorize_tick_owner(compliance_port, &request)?;
    validate_tick_request(&request)?;
    let due_candidates = retention_due_candidates(compliance_port, &owner_did, &request)?;
    let due_candidate_count = due_candidates.len();
    let shredded_message_ids =
        shred_due_messages(compliance_port, owner_did.as_str(), &request, due_candidates)?;
    let projection_reports =
        project_sorted_partition_reports(compliance_port, partition_registry, &request, &owner_did)?;
    let archived_entries = archive_due_partitions(partition_registry, request)?;
    Ok(build_execution_report(
        owner_did,
        due_candidate_count,
        shredded_message_ids,
        projection_reports,
        archived_entries,
    ))
}

fn shred_due_messages(
    compliance_port: &mut impl DataLayerM10Phase6CompliancePort,
    owner_did: &str,
    request: &DataLayerM10Phase6ExecutionTickRequest,
    due_candidates: Vec<kamn_data_layer::DataLayerM10Phase6RetentionDueCandidate>,
) -> Result<Vec<String>, DataLayerM10PartitionLifecycleError> {
    shred_due_candidates(
        compliance_port,
        owner_did,
        request.shredded_at_epoch_seconds,
        due_candidates,
    )
}

fn project_sorted_partition_reports(
    compliance_port: &mut impl DataLayerM10Phase6CompliancePort,
    partition_registry: &mut DataLayerM10PartitionLifecycleRegistry,
    request: &DataLayerM10Phase6ExecutionTickRequest,
    owner_did: &str,
) -> Result<Vec<crate::DataLayerM10ComplianceShredProjectionReport>, DataLayerM10PartitionLifecycleError> {
    let mut projection_reports =
        project_partition_reports(compliance_port, partition_registry, request, owner_did)?;
    projection_reports.sort_by(compare_partition_reports);
    Ok(projection_reports)
}

fn compare_partition_reports(
    left: &crate::DataLayerM10ComplianceShredProjectionReport,
    right: &crate::DataLayerM10ComplianceShredProjectionReport,
) -> std::cmp::Ordering {
    left.partition_month_id
        .cmp(&right.partition_month_id)
        .then(left.partition_name.cmp(&right.partition_name))
}

fn build_execution_report(
    owner_did: String,
    due_candidate_count: usize,
    shredded_message_ids: Vec<String>,
    projection_reports: Vec<crate::DataLayerM10ComplianceShredProjectionReport>,
    archived_entries: Vec<crate::DataLayerM10ArchivalIndexEntry>,
) -> DataLayerM10Phase6ExecutionTickReport {
    DataLayerM10Phase6ExecutionTickReport {
        owner_did,
        due_candidate_count,
        shredded_message_ids,
        projection_reports,
        archived_entries,
        reason_code: DATA_LAYER_M10_PHASE6_EXECUTION_APPLIED_REASON_CODE,
    }
}

fn authorize_tick_owner(
    compliance_port: &mut impl DataLayerM10Phase6CompliancePort,
    request: &DataLayerM10Phase6ExecutionTickRequest,
) -> Result<String, DataLayerM10PartitionLifecycleError> {
    compliance_port
        .authorize_owner_scope(
            request.requester_owner_did.as_str(),
            request.owner_did.as_str(),
        )
        .map_err(map_phase6_port_error_to_m10)
}

fn retention_due_candidates(
    compliance_port: &mut impl DataLayerM10Phase6CompliancePort,
    owner_did: &str,
    request: &DataLayerM10Phase6ExecutionTickRequest,
) -> Result<Vec<kamn_data_layer::DataLayerM10Phase6RetentionDueCandidate>, DataLayerM10PartitionLifecycleError> {
    compliance_port
        .retention_due_for_owner(owner_did, request.now_epoch_seconds)
        .map_err(map_phase6_port_error_to_m10)
}

fn validate_tick_request(
    request: &DataLayerM10Phase6ExecutionTickRequest,
) -> Result<(), DataLayerM10PartitionLifecycleError> {
    if request.now_epoch_seconds == 0 {
        return Err(phase6_execution_failed(
            DATA_LAYER_M10_PHASE6_EXECUTION_INPUT_INVALID_REASON_CODE,
            "now_epoch_seconds must be > 0",
        ));
    }
    if request.shredded_at_epoch_seconds == 0 {
        return Err(phase6_execution_failed(
            DATA_LAYER_M10_PHASE6_EXECUTION_INPUT_INVALID_REASON_CODE,
            "shredded_at_epoch_seconds must be > 0",
        ));
    }
    validate_non_empty(request.object_storage_prefix.as_str(), "object_storage_prefix")
        .map_err(crate::data_layer_m10_partition_archival::error::map_phase6_projection_error_to_m10)
}
