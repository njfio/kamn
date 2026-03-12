mod mapping;

use kamn_data_layer::data_layer_m10_phase6_runtime_evidence::data_layer_m10_project_phase6_runtime_evidence_bundle as data_layer_m10_project_phase6_runtime_evidence_bundle_policy;

use super::super::{
    DataLayerM10PartitionLifecycleError, DataLayerM10Phase6RuntimeEvidenceBundle,
    DataLayerM10Phase6RuntimeEvidenceInput,
};
use mapping::{
    map_data_layer_runtime_evidence_error_to_m10, map_phase6_runtime_evidence_bundle_from_policy,
    map_phase6_runtime_evidence_input_to_policy,
};

/// Projects canonical Phase-6 runtime evidence from one scheduler-cycle report and runtime state.
pub fn data_layer_m10_project_phase6_runtime_evidence_bundle(
    input: DataLayerM10Phase6RuntimeEvidenceInput,
) -> Result<DataLayerM10Phase6RuntimeEvidenceBundle, DataLayerM10PartitionLifecycleError> {
    let policy_input = map_phase6_runtime_evidence_input_to_policy(input)?;
    let policy_bundle = data_layer_m10_project_phase6_runtime_evidence_bundle_policy(policy_input)
        .map_err(map_data_layer_runtime_evidence_error_to_m10)?;
    Ok(map_phase6_runtime_evidence_bundle_from_policy(
        policy_bundle,
    ))
}
