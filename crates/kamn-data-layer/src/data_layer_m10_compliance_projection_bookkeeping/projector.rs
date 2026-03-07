use crate::{DataLayerM10ComplianceProjectionPort, DataLayerM10PartitionRegistryStateMachine};

use super::helpers::{
    collect_partition_message_ids, evaluate_partition_message_shred_completeness,
    map_projection_port_error, map_state_machine_error, validate_partition_month_id,
};
use super::{
    DataLayerM10ComplianceProjectionBookkeepingError, DataLayerM10ComplianceShredProjectionReport,
    DataLayerM10ComplianceShredProjectionRequest,
    DATA_LAYER_M10_COMPLIANCE_LEGAL_HOLD_ACTIVE_REASON_CODE,
    DATA_LAYER_M10_COMPLIANCE_PROJECTION_APPLIED_REASON_CODE,
    DATA_LAYER_M10_COMPLIANCE_SHRED_COMPLETENESS_FALSE_REASON_CODE,
    DATA_LAYER_M10_COMPLIANCE_SHRED_COMPLETENESS_TRUE_REASON_CODE,
};

/// Projects partition shred completeness through the extracted port and state-machine seams.
pub fn data_layer_m10_project_partition_shred_completeness_with_port(
    state_machine: &mut DataLayerM10PartitionRegistryStateMachine,
    compliance_port: &impl DataLayerM10ComplianceProjectionPort,
    request: DataLayerM10ComplianceShredProjectionRequest,
) -> Result<
    DataLayerM10ComplianceShredProjectionReport,
    DataLayerM10ComplianceProjectionBookkeepingError,
> {
    let owner_did = compliance_port
        .authorize_owner_scope(
            request.requester_owner_did.as_str(),
            request.owner_did.as_str(),
        )
        .map_err(map_projection_port_error)?;
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
    let record = state_machine
        .apply_partition_shred_completeness(
            request.partition_month_id,
            all_messages_shredded,
            DATA_LAYER_M10_COMPLIANCE_PROJECTION_APPLIED_REASON_CODE,
        )
        .map_err(map_state_machine_error)?;

    Ok(DataLayerM10ComplianceShredProjectionReport {
        partition_month_id: record.partition_month_id,
        partition_name: record.partition_name,
        total_partition_messages,
        shredded_partition_messages,
        all_messages_shredded,
        reason_code,
        projection_reason_code: DATA_LAYER_M10_COMPLIANCE_PROJECTION_APPLIED_REASON_CODE,
    })
}
