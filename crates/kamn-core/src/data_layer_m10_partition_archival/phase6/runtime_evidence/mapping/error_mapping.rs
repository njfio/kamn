use kamn_data_layer::data_layer_m10_phase6_runtime_evidence::DataLayerM10Phase6PolicyRuntimeEvidenceError;

use crate::data_layer_m10_partition_archival::DataLayerM10PartitionLifecycleError;

pub(crate) fn map_data_layer_runtime_evidence_error_to_m10(
    error: DataLayerM10Phase6PolicyRuntimeEvidenceError,
) -> DataLayerM10PartitionLifecycleError {
    match error {
        DataLayerM10Phase6PolicyRuntimeEvidenceError::InvalidInput { field, reason_code } => {
            DataLayerM10PartitionLifecycleError::InvalidPhase6RuntimeEvidenceInput {
                field,
                reason_code,
            }
        }
    }
}
